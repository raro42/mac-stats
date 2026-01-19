//! Ollama integration module
//! 
//! Local LLM chat interface using Ollama API

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use url::Url;
use crate::security;

/// Ollama configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub endpoint: String, // e.g., "http://localhost:11434"
    pub model: String,
    pub api_key: Option<String>, // For remote instances
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama2".to_string(),
            api_key: None,
        }
    }
}

impl OllamaConfig {
    pub fn validate(&self) -> Result<()> {
        Url::parse(&self.endpoint)
            .context("Invalid Ollama endpoint URL")?;
        Ok(())
    }

    /// Get API key from Keychain if configured
    #[allow(dead_code)] // Used in OllamaClient methods
    pub fn get_api_key(&self) -> Result<Option<String>> {
        if let Some(ref keychain_account) = self.api_key {
            security::get_credential(keychain_account)
                .context("Failed to retrieve Ollama API key from Keychain")
        } else {
            Ok(None)
        }
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
}

/// Chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
}

/// Ollama client
pub struct OllamaClient {
    pub config: OllamaConfig,
    #[allow(dead_code)] // Used in methods that may not be called directly
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Result<Self> {
        use tracing::info;
        
        config.validate()?;
        info!("Ollama: Initializing client with endpoint: {}", config.endpoint);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // Long timeout for LLM
            .build()?;

        Ok(Self {
            config,
            client,
        })
    }

    /// Check if Ollama is available
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn check_connection(&self) -> Result<bool> {
        use tracing::{debug, info};
        
        let url = format!("{}/api/tags", self.config.endpoint);
        debug!("Ollama: Checking connection to {}", url);
        
        let mut request = self.client.get(&url);
        
        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            request = request.header("Authorization", format!("Bearer {}", api_key));
            debug!("Ollama: Using API key for authentication (masked: {})", masked);
        }

        match request.send().await {
            Ok(response) => {
                let success = response.status().is_success();
                if success {
                    info!("Ollama: Connection successful to {}", self.config.endpoint);
                } else {
                    debug!("Ollama: Connection failed - HTTP status: {}", response.status());
                }
                Ok(success)
            },
            Err(e) => {
                debug!("Ollama: Connection error: {}", e);
                Ok(false)
            }
        }
    }

    /// Send chat message (async, non-blocking)
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse> {
        use tracing::{debug, info};
        use serde_json;
        
        let url = format!("{}/api/chat", self.config.endpoint);
        info!("Ollama: Using endpoint: {}", url);
        info!("Ollama: Streaming is disabled (stream: false)");
        
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.clone(),
            stream: false,
        };

        // Log raw request JSON before sending
        let request_json = serde_json::to_string_pretty(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string());
        info!("Ollama: Sending HTTP POST to {} with request JSON:\n{}", url, request_json);

        let mut http_request = self.client.post(&url).json(&request);
        
        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
            debug!("Ollama: Using API key for chat request (masked: {})", masked);
        }

        let start_time = std::time::Instant::now();
        let response = http_request
            .send()
            .await
            .context("Failed to send chat request to Ollama")?
            .json::<ChatResponse>()
            .await
            .context("Failed to parse Ollama response")?;
        let duration = start_time.elapsed();

        // Log raw response JSON
        let response_json = serde_json::to_string_pretty(&response)
            .unwrap_or_else(|_| "Failed to serialize response".to_string());
        info!("Ollama: Received HTTP response in {:?} with response JSON:\n{}", duration, response_json);

        Ok(response)
    }

    /// List available models (async, non-blocking)
    #[allow(dead_code)] // May be used in future or via direct client access
    pub async fn list_models(&self) -> Result<Vec<String>> {
        use tracing::{debug, info};
        
        let url = format!("{}/api/tags", self.config.endpoint);
        info!("Ollama: Using endpoint: {}", url);
        debug!("Ollama: Listing models from {}", url);
        
        let mut request = self.client.get(&url);
        
        // Add API key if configured
        if let Ok(Some(api_key)) = self.config.get_api_key() {
            let masked = security::mask_credential(&api_key);
            request = request.header("Authorization", format!("Bearer {}", api_key));
            debug!("Ollama: Using API key for model listing (masked: {})", masked);
        }

        let response: serde_json::Value = request
            .send()
            .await
            .context("Failed to request models from Ollama")?
            .json()
            .await
            .context("Failed to parse models response")?;
        
        // Log raw response JSON
        let response_json = serde_json::to_string_pretty(&response)
            .unwrap_or_else(|_| "Failed to serialize response".to_string());
        info!("Ollama: Received models list HTTP response JSON:\n{}", response_json);
        
        let models: Vec<String> = response
            .get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        info!("Ollama: Extracted {} models from response", models.len());
        Ok(models)
    }
}
