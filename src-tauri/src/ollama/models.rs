//! Model classification and role-based resolution.
//!
//! At startup, build a `ModelCatalog` from `/api/tags` data. Agents declare a
//! `model_role` in their agent.json (e.g. "general", "code", "small", "vision",
//! "thinking", "cheap", "expensive"); the catalog resolves that to an actual
//! model name. **Only local (non-cloud) models are chosen** for role resolution;
//! cloud models are used only when the user explicitly configures them (e.g. `model`
//! in agent.json or default Ollama config).

use crate::ollama::ModelSummary;
use std::sync::{Mutex, OnceLock};
use tracing::{debug, info, warn};

/// Global cached catalog, populated at startup after querying Ollama.
/// load_agents() checks this to auto-resolve model_role for each agent.
fn global_catalog() -> &'static Mutex<Option<ModelCatalog>> {
    static CATALOG: OnceLock<Mutex<Option<ModelCatalog>>> = OnceLock::new();
    CATALOG.get_or_init(|| Mutex::new(None))
}

/// Store a catalog after startup detection. Called once from ensure_ollama_agent_ready_at_startup.
pub fn set_global_catalog(catalog: ModelCatalog) {
    if let Ok(mut guard) = global_catalog().lock() {
        *guard = Some(catalog);
    }
}

/// Get a clone of the cached catalog, if available.
pub fn get_global_catalog() -> Option<ModelCatalog> {
    global_catalog().lock().ok().and_then(|g| g.clone())
}

/// First local (non-cloud) model name from the global catalog, if any.
/// Used to prefer local over cloud when the configured default is a cloud model.
pub fn get_first_local_model_name() -> Option<String> {
    let local: Vec<_> = get_global_catalog()
        .map(|catalog| catalog.eligible_local().map(|m| m.name.clone()).collect())
        .unwrap_or_default();
    local.into_iter().next()
}

/// True if the given model name is vision-capable (per catalog). Used for verification with screenshot.
pub fn is_vision_capable(model_name: &str) -> bool {
    get_global_catalog()
        .and_then(|c| c.model_by_name(model_name).map(|m| m.capability == ModelCapability::Vision))
        .unwrap_or(false)
}

/// First available local vision model name for verification (screenshot tasks). None if no vision model.
pub fn get_vision_model_for_verification() -> Option<String> {
    let catalog = get_global_catalog()?;
    let local: Vec<_> = catalog.eligible_local().collect();
    let vision: Vec<_> = local
        .iter()
        .filter(|m| m.capability == ModelCapability::Vision)
        .collect();
    vision.last().map(|m| m.name.clone())
}

/// Maximum parameter count (in billions) for auto-selected models.
/// Models above this are never chosen automatically.
const MAX_AUTO_PARAMS_B: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCapability {
    /// Vision/multimodal (llava, pixtral, etc.)
    Vision,
    /// Reasoning/thinking (deepseek-r1, qwq, etc.)
    Reasoning,
    /// Code-oriented (coder, code in name/family)
    Code,
    /// General-purpose (default)
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSizeTier {
    Small,  // < 4B
    Medium, // 4B–15B
    Large,  // > 15B
}

#[derive(Debug, Clone)]
pub struct ClassifiedModel {
    pub name: String,
    pub capability: ModelCapability,
    pub size_tier: ModelSizeTier,
    pub param_billions: f64,
    /// True if model is a cloud/remote model (e.g. qwen3.5:cloud). Used to prefer local models.
    pub is_cloud: bool,
}

/// Catalog of available models, classified by capability and size.
#[derive(Debug, Clone)]
pub struct ModelCatalog {
    pub models: Vec<ClassifiedModel>,
}

impl ModelCatalog {
    /// Build a catalog from the `/api/tags` response.
    pub fn from_model_list(summaries: &[ModelSummary]) -> Self {
        let mut models: Vec<ClassifiedModel> = summaries
            .iter()
            .filter_map(classify_model)
            .collect();

        models.sort_by(|a, b| a.param_billions.partial_cmp(&b.param_billions).unwrap_or(std::cmp::Ordering::Equal));

        if models.is_empty() {
            warn!("ModelCatalog: no models could be classified from {} summaries", summaries.len());
        } else {
            info!(
                "ModelCatalog: classified {} models: {}",
                models.len(),
                models
                    .iter()
                    .map(|m| format!("{}({:?}/{:.1}B)", m.name, m.capability, m.param_billions))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        Self { models }
    }

    /// Resolve a `model_role` string to the best available model name.
    /// Returns None if no suitable model is found.
    /// Roles: code, general, small, vision, thinking, reasoning, cheap (= small), expensive.
    pub fn resolve_role(&self, role: &str) -> Option<&ClassifiedModel> {
        match role.to_lowercase().as_str() {
            "code" => self.pick_code(),
            "general" => self.pick_general(),
            "small" | "cheap" => self.pick_small(),
            "vision" => self.pick_vision(),
            "thinking" | "reasoning" => self.pick_reasoning(),
            "expensive" => self.pick_expensive(),
            other => {
                warn!("ModelCatalog: unknown model_role '{}', treating as 'general'", other);
                self.pick_general()
            }
        }
    }

    /// Check whether a specific model name is available.
    pub fn has_model(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.models.iter().any(|m| {
            let mn = m.name.to_lowercase();
            mn == lower || mn.starts_with(&format!("{}:", lower))
        })
    }

    /// Look up a classified model by name (exact or base:tag). Used for capability checks.
    pub fn model_by_name(&self, name: &str) -> Option<&ClassifiedModel> {
        let lower = name.to_lowercase();
        self.models.iter().find(|m| {
            let mn = m.name.to_lowercase();
            mn == lower || mn.starts_with(&format!("{}:", lower))
        })
    }

    /// Eligible local models only (no cloud). Role resolution uses this only; cloud is used only when the user explicitly configures it (e.g. model in agent.json or default).
    fn eligible_local(&self) -> impl Iterator<Item = &ClassifiedModel> {
        self.models
            .iter()
            .filter(|m| m.param_billions <= MAX_AUTO_PARAMS_B && !m.is_cloud)
    }

    fn pick_vision(&self) -> Option<&ClassifiedModel> {
        let local: Vec<_> = self.eligible_local().collect();
        let vision: Vec<_> = local
            .iter()
            .filter(|m| m.capability == ModelCapability::Vision)
            .collect();
        if let Some(best) = vision.last() {
            debug!("ModelCatalog: role=vision -> {} ({:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        debug!("ModelCatalog: role=vision -> no local vision models, falling back to general");
        self.pick_general()
    }

    fn pick_reasoning(&self) -> Option<&ClassifiedModel> {
        let local: Vec<_> = self.eligible_local().collect();
        let reasoning_medium: Vec<_> = local
            .iter()
            .filter(|m| m.capability == ModelCapability::Reasoning && m.size_tier == ModelSizeTier::Medium)
            .collect();
        if let Some(best) = reasoning_medium.last() {
            debug!("ModelCatalog: role=thinking -> {} (reasoning/medium, {:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        let reasoning: Vec<_> = local
            .iter()
            .filter(|m| m.capability == ModelCapability::Reasoning)
            .collect();
        if let Some(best) = reasoning.last() {
            debug!("ModelCatalog: role=thinking -> {} (reasoning/any, {:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        debug!("ModelCatalog: role=thinking -> no local reasoning models, falling back to general");
        self.pick_general()
    }

    fn pick_expensive(&self) -> Option<&ClassifiedModel> {
        let local: Vec<_> = self.eligible_local().collect();
        if let Some(best) = local.last() {
            debug!("ModelCatalog: role=expensive -> {} ({:.1}B, largest local)", best.name, best.param_billions);
            return Some(best);
        }
        if let Some(best) = self.models.iter().filter(|m| !m.is_cloud).next_back() {
            warn!("ModelCatalog: role=expensive -> {} (above cap, local only)", best.name);
            return Some(best);
        }
        None
    }

    fn pick_code(&self) -> Option<&ClassifiedModel> {
        let local: Vec<_> = self.eligible_local().collect();
        let code_medium: Vec<_> = local
            .iter()
            .filter(|m| m.capability == ModelCapability::Code && m.size_tier == ModelSizeTier::Medium)
            .collect();
        if let Some(best) = code_medium.last() {
            debug!("ModelCatalog: role=code -> {} (code/medium, {:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        if let Some(best) = local.iter().filter(|m| m.capability == ModelCapability::Code).next_back() {
            debug!("ModelCatalog: role=code -> {} (code/any, {:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        debug!("ModelCatalog: role=code -> no local code models, falling back to general");
        self.pick_general()
    }

    fn pick_general(&self) -> Option<&ClassifiedModel> {
        let local: Vec<_> = self.eligible_local().collect();
        let general_medium: Vec<_> = local
            .iter()
            .filter(|m| m.capability == ModelCapability::General && m.size_tier == ModelSizeTier::Medium)
            .collect();
        if let Some(best) = general_medium.last() {
            debug!("ModelCatalog: role=general -> {} (general/medium, {:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        if let Some(best) = local.iter().filter(|m| m.capability == ModelCapability::General).next_back() {
            debug!("ModelCatalog: role=general -> {} (general/any, {:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        if let Some(best) = local.last() {
            debug!("ModelCatalog: role=general -> {} (any local fallback, {:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        if let Some(best) = self.models.iter().filter(|m| !m.is_cloud).min_by(|a, b| a.param_billions.partial_cmp(&b.param_billions).unwrap_or(std::cmp::Ordering::Equal)) {
            warn!(
                "ModelCatalog: no local within cap, using smallest local: {} ({:.1}B)",
                best.name, best.param_billions
            );
            return Some(best);
        }
        None
    }

    fn pick_small(&self) -> Option<&ClassifiedModel> {
        let local: Vec<_> = self.eligible_local().collect();
        if let Some(best) = local.first() {
            debug!("ModelCatalog: role=small -> {} ({:.1}B)", best.name, best.param_billions);
            return Some(best);
        }
        if let Some(best) = self.models.iter().filter(|m| !m.is_cloud).min_by(|a, b| a.param_billions.partial_cmp(&b.param_billions).unwrap_or(std::cmp::Ordering::Equal)) {
            warn!("ModelCatalog: role=small -> {} ({:.1}B, above cap, local only)", best.name, best.param_billions);
            return Some(best);
        }
        None
    }
}

/// True if the model name indicates a cloud/remote model (e.g. qwen3.5:cloud).
/// Used to prefer local models in default selection and catalog resolution.
pub fn is_cloud_model(name: &str) -> bool {
    name.to_lowercase().contains("cloud")
}

fn classify_model(summary: &ModelSummary) -> Option<ClassifiedModel> {
    let param_billions = parse_param_size(summary)?;
    let capability = detect_capability(summary);
    let size_tier = if param_billions < 4.0 {
        ModelSizeTier::Small
    } else if param_billions <= 15.0 {
        ModelSizeTier::Medium
    } else {
        ModelSizeTier::Large
    };
    let is_cloud = is_cloud_model(&summary.name);

    Some(ClassifiedModel {
        name: summary.name.clone(),
        capability,
        size_tier,
        param_billions,
        is_cloud,
    })
}

/// Parse parameter count in billions from `details.parameter_size` (e.g., "7.6B", "3.2B")
/// or estimate from file size if details are missing.
fn parse_param_size(summary: &ModelSummary) -> Option<f64> {
    if let Some(ref details) = summary.details {
        if let Some(ref ps) = details.parameter_size {
            let cleaned = ps.trim().to_uppercase();
            let cleaned = cleaned.trim_end_matches('B');
            if let Ok(n) = cleaned.parse::<f64>() {
                return Some(n);
            }
        }
    }
    // Estimate from file size: very rough heuristic (Q4 quantized ~ 0.5 bytes/param)
    if let Some(size_bytes) = summary.size {
        let estimated_b = (size_bytes as f64) / 0.5e9;
        debug!(
            "ModelCatalog: estimating {} at {:.1}B from file size {}",
            summary.name, estimated_b, size_bytes
        );
        return Some(estimated_b);
    }
    warn!("ModelCatalog: cannot determine size for '{}', skipping", summary.name);
    None
}

/// Detect capability from name and family. Order: Vision → Reasoning → Code → General.
fn detect_capability(summary: &ModelSummary) -> ModelCapability {
    let name_lower = summary.name.to_lowercase();
    let check_family = |s: &str| {
        let sl = s.to_lowercase();
        name_lower.contains(&sl)
            || summary
                .details
                .as_ref()
                .and_then(|d| d.family.as_ref())
                .map(|f| f.to_lowercase().contains(&sl))
                .unwrap_or(false)
            || summary
                .details
                .as_ref()
                .and_then(|d| d.families.as_ref())
                .map(|fs| fs.iter().any(|f| f.to_lowercase().contains(&sl)))
                .unwrap_or(false)
    };
    // Vision: llava, bakllava, vision, pixtral, minicpm-v
    if check_family("llava")
        || check_family("bakllava")
        || check_family("vision")
        || check_family("pixtral")
        || check_family("minicpm-v")
    {
        return ModelCapability::Vision;
    }
    // Reasoning/thinking: deepseek-r1, thinking, reason, qwq, openreason
    if check_family("deepseek-r1")
        || check_family("thinking")
        || check_family("reason")
        || check_family("qwq")
        || check_family("openreason")
    {
        return ModelCapability::Reasoning;
    }
    // Code
    if name_lower.contains("coder") || name_lower.contains("code") {
        return ModelCapability::Code;
    }
    if let Some(ref details) = summary.details {
        if let Some(ref family) = details.family {
            let f = family.to_lowercase();
            if f.contains("coder") || f.contains("code") {
                return ModelCapability::Code;
            }
        }
        if let Some(ref families) = details.families {
            for f in families {
                let fl = f.to_lowercase();
                if fl.contains("coder") || fl.contains("code") {
                    return ModelCapability::Code;
                }
            }
        }
    }
    ModelCapability::General
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ollama::{ModelDetails, ModelSummary};

    fn make_model(name: &str, param_size: &str, family: &str) -> ModelSummary {
        ModelSummary {
            name: name.to_string(),
            modified_at: None,
            size: None,
            digest: None,
            details: Some(ModelDetails {
                format: None,
                family: Some(family.to_string()),
                families: None,
                parameter_size: Some(param_size.to_string()),
                quantization_level: None,
            }),
        }
    }

    #[test]
    fn classify_code_model() {
        let models = vec![
            make_model("qwen2.5-coder:latest", "7.6B", "qwen2"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
            make_model("llama3.2:latest", "3.2B", "llama"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        assert_eq!(catalog.models.len(), 3);

        let resolved = catalog.resolve_role("code").unwrap();
        assert_eq!(resolved.name, "qwen2.5-coder:latest");
        assert_eq!(resolved.capability, ModelCapability::Code);
    }

    #[test]
    fn classify_general_prefers_medium() {
        let models = vec![
            make_model("llama3.2:latest", "3.2B", "llama"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
            make_model("gemma3:12b", "12.2B", "gemma3"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("general").unwrap();
        assert_eq!(resolved.size_tier, ModelSizeTier::Medium);
    }

    #[test]
    fn classify_small_picks_smallest() {
        let models = vec![
            make_model("deepscaler:latest", "1.8B", "qwen2"),
            make_model("llama3.2:latest", "3.2B", "llama"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("small").unwrap();
        assert_eq!(resolved.name, "deepscaler:latest");
    }

    #[test]
    fn excludes_large_models() {
        let models = vec![
            make_model("openthinker:32b", "32.8B", "qwen2"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("general").unwrap();
        assert_eq!(resolved.name, "qwen3:latest");
    }

    #[test]
    fn falls_back_to_smallest_when_all_large() {
        let models = vec![
            make_model("openthinker:32b", "32.8B", "qwen2"),
            make_model("devstral:latest", "23.6B", "mistral"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("general").unwrap();
        assert_eq!(resolved.name, "devstral:latest");
    }

    #[test]
    fn has_model_case_insensitive() {
        let models = vec![make_model("Qwen3:latest", "8.2B", "qwen3")];
        let catalog = ModelCatalog::from_model_list(&models);
        assert!(catalog.has_model("qwen3:latest"));
        assert!(catalog.has_model("Qwen3:latest"));
    }

    #[test]
    fn code_fallback_to_general_when_no_code_models() {
        let models = vec![
            make_model("qwen3:latest", "8.2B", "qwen3"),
            make_model("llama3.2:latest", "3.2B", "llama"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("code").unwrap();
        // Should fall back to general since no code models
        assert_eq!(resolved.capability, ModelCapability::General);
    }

    #[test]
    fn resolve_vision_picks_vision_model() {
        let models = vec![
            make_model("llama3.2:latest", "3.2B", "llama"),
            make_model("llava:latest", "7.5B", "llava"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("vision").unwrap();
        assert_eq!(resolved.capability, ModelCapability::Vision);
        assert_eq!(resolved.name, "llava:latest");
    }

    #[test]
    fn resolve_thinking_picks_reasoning_model() {
        let models = vec![
            make_model("llama3.2:latest", "3.2B", "llama"),
            make_model("deepseek-r1:latest", "8.2B", "deepseek-r1"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("thinking").unwrap();
        assert_eq!(resolved.capability, ModelCapability::Reasoning);
        assert_eq!(resolved.name, "deepseek-r1:latest");
    }

    #[test]
    fn resolve_cheap_equals_small() {
        let models = vec![
            make_model("qwen2.5:1.5b", "1.5B", "qwen2"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let small = catalog.resolve_role("small").unwrap();
        let cheap = catalog.resolve_role("cheap").unwrap();
        assert_eq!(small.name, cheap.name);
    }

    #[test]
    fn resolve_expensive_picks_largest_eligible() {
        let models = vec![
            make_model("llama3.2:latest", "3.2B", "llama"),
            make_model("qwen3:latest", "8.2B", "qwen3"),
            make_model("gemma3:12b", "12.2B", "gemma3"),
        ];
        let catalog = ModelCatalog::from_model_list(&models);
        let resolved = catalog.resolve_role("expensive").unwrap();
        assert_eq!(resolved.name, "gemma3:12b");
    }
}
