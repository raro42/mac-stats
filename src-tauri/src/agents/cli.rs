//! Agent-test CLI: run an agent with prompts from testing.md. Invoked as `mac_stats agent test <selector> [path]`.

use crate::agents::{find_agent_by_id_or_name, load_agents};
use crate::config::Config;
use std::path::Path;
use tracing::info;

/// Parse testing.md content into a list of test prompts. Splits on `## `; each section body (after first newline) is one prompt.
/// If no `## ` headers, treat the whole file as one prompt.
pub fn parse_testing_md(content: &str) -> Vec<String> {
    let content = content.trim();
    if content.is_empty() {
        return Vec::new();
    }
    if !content.contains("## ") {
        return vec![content.to_string()];
    }
    let mut prompts = Vec::new();
    for block in content.split("## ") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let body = match block.find('\n') {
            Some(i) => block[i + 1..].trim(),
            None => continue,
        };
        if !body.is_empty() {
            prompts.push(body.to_string());
        }
    }
    if prompts.is_empty() {
        prompts.push(content.to_string());
    }
    prompts
}

/// Run agent tests: resolve agent, read testing.md, run each prompt via Ollama. Returns exit code (0 = ok).
pub async fn run_agent_test(selector: &str, path: Option<&Path>) -> Result<(), i32> {
    crate::commands::ollama::ensure_ollama_agent_ready_at_startup().await;

    let agents = load_agents();
    let agent = match find_agent_by_id_or_name(&agents, selector) {
        Some(a) => a,
        None => {
            let list: String = agents
                .iter()
                .map(|a| a.slug.as_deref().unwrap_or(a.name.as_str()).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            eprintln!("Agent not found: {:?}. Available: {}", selector, list);
            return Err(1);
        }
    };

    let test_path = match path {
        Some(p) => p.to_path_buf(),
        None => {
            let dir = Config::agents_dir().join(format!("agent-{}", agent.id));
            let p = dir.join("testing.md");
            if !p.exists() {
                eprintln!("testing.md not found at {}. Required for each agent.", p.display());
                return Err(1);
            }
            p
        }
    };

    let content = match std::fs::read_to_string(&test_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {}", test_path.display(), e);
            return Err(1);
        }
    };

    let prompts = parse_testing_md(&content);
    if prompts.is_empty() {
        eprintln!("No test prompts in {}", test_path.display());
        return Err(1);
    }

    info!(
        "Agent test: {} ({}) — {} prompt(s) from {}",
        agent.name,
        agent.id,
        prompts.len(),
        test_path.display()
    );

    for (i, prompt) in prompts.iter().enumerate() {
        info!("Agent test {}/{}: running ({} chars)", i + 1, prompts.len(), prompt.chars().count());
        match crate::commands::ollama::run_agent_ollama_session(agent, prompt, None).await {
            Ok(response) => {
                info!("Agent test {}/{}: response {} chars", i + 1, prompts.len(), response.chars().count());
                println!("Test {}: {} chars", i + 1, response.chars().count());
            }
            Err(e) => {
                eprintln!("Agent test {}/{} failed: {}", i + 1, prompts.len(), e);
                return Err(1);
            }
        }
    }

    println!("Agent {} ({}): {} tests run, ok", agent.name, agent.id, prompts.len());
    Ok(())
}
