//! Model classification and role-based resolution.
//!
//! At startup, build a `ModelCatalog` from `/api/tags` data. Agents declare a
//! `model_role` ("general", "code", "small") in their agent.json; the catalog
//! resolves that to an actual model name from whatever is currently installed.

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

/// Maximum parameter count (in billions) for auto-selected models.
/// Models above this are never chosen automatically.
const MAX_AUTO_PARAMS_B: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCapability {
    Code,
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
    pub fn resolve_role(&self, role: &str) -> Option<&ClassifiedModel> {
        match role.to_lowercase().as_str() {
            "code" => self.pick_code(),
            "general" => self.pick_general(),
            "small" => self.pick_small(),
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

    fn eligible(&self) -> impl Iterator<Item = &ClassifiedModel> {
        self.models
            .iter()
            .filter(|m| m.param_billions <= MAX_AUTO_PARAMS_B)
    }

    fn pick_code(&self) -> Option<&ClassifiedModel> {
        // Prefer code-capability models in medium tier
        let code_medium: Vec<_> = self
            .eligible()
            .filter(|m| m.capability == ModelCapability::Code && m.size_tier == ModelSizeTier::Medium)
            .collect();
        if let Some(best) = code_medium.last() {
            debug!("ModelCatalog: role=code -> {} (code/medium, {:.1}B)", best.name, best.param_billions);
            return Some(best);
        }
        // Fall back to any code model under cap
        if let Some(best) = self.eligible().filter(|m| m.capability == ModelCapability::Code).last() {
            debug!("ModelCatalog: role=code -> {} (code/any, {:.1}B)", best.name, best.param_billions);
            return Some(best);
        }
        // Fall back to largest general under cap
        debug!("ModelCatalog: role=code -> no code models, falling back to general");
        self.pick_general()
    }

    fn pick_general(&self) -> Option<&ClassifiedModel> {
        // Prefer general-capability models in medium tier
        let general_medium: Vec<_> = self
            .eligible()
            .filter(|m| m.capability == ModelCapability::General && m.size_tier == ModelSizeTier::Medium)
            .collect();
        if let Some(best) = general_medium.last() {
            debug!("ModelCatalog: role=general -> {} (general/medium, {:.1}B)", best.name, best.param_billions);
            return Some(best);
        }
        // Fall back to any general model under cap (prefer larger)
        if let Some(best) = self.eligible().filter(|m| m.capability == ModelCapability::General).last() {
            debug!("ModelCatalog: role=general -> {} (general/any, {:.1}B)", best.name, best.param_billions);
            return Some(best);
        }
        // Last resort: any model under cap
        if let Some(best) = self.eligible().last() {
            debug!("ModelCatalog: role=general -> {} (any/fallback, {:.1}B)", best.name, best.param_billions);
            return Some(best);
        }
        // If all models are above cap, pick the smallest available
        if let Some(best) = self.models.first() {
            warn!(
                "ModelCatalog: all models above {:.0}B cap, using smallest: {} ({:.1}B)",
                MAX_AUTO_PARAMS_B, best.name, best.param_billions
            );
            return Some(best);
        }
        None
    }

    fn pick_small(&self) -> Option<&ClassifiedModel> {
        // Smallest model overall (sorted ascending by param_billions)
        let eligible: Vec<_> = self.eligible().collect();
        if let Some(best) = eligible.first() {
            debug!("ModelCatalog: role=small -> {} ({:.1}B)", best.name, best.param_billions);
            return Some(*best);
        }
        // Fallback: smallest of any size
        if let Some(best) = self.models.first() {
            warn!("ModelCatalog: role=small -> {} ({:.1}B, above cap)", best.name, best.param_billions);
            return Some(best);
        }
        None
    }
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

    Some(ClassifiedModel {
        name: summary.name.clone(),
        capability,
        size_tier,
        param_billions,
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

/// Detect whether a model is code-oriented based on name and family.
fn detect_capability(summary: &ModelSummary) -> ModelCapability {
    let name_lower = summary.name.to_lowercase();
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
}
