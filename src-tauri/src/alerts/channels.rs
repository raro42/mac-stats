//! Alert channel implementations

use super::AlertContext;
use anyhow::{Result, Context};
use crate::security;

/// Trait for alert channels
pub trait AlertChannel: Send + Sync {
    fn send(&mut self, message: &str, context: &AlertContext) -> Result<()>;
    #[allow(dead_code)] // Part of trait API, may be used in future
    fn get_id(&self) -> &str;
    #[allow(dead_code)] // Part of trait API, may be used in future
    fn get_name(&self) -> &str;
}

/// Telegram alert channel
#[allow(dead_code)] // Part of API, may be used in future
pub struct TelegramChannel {
    id: String,
    bot_token_keychain_account: String,
    chat_id: String,
}

impl TelegramChannel {
    #[allow(dead_code)] // Part of API, may be used in future
    pub fn new(id: String, chat_id: String) -> Self {
        let bot_token_keychain_account = format!("telegram_bot_{}", id);
        Self {
            id,
            bot_token_keychain_account,
            chat_id,
        }
    }

    #[allow(dead_code)] // Used internally, may be called in future
    fn get_bot_token(&self) -> Result<String> {
        security::get_credential(&self.bot_token_keychain_account)?
            .context("Telegram bot token not found in Keychain")
    }
}

impl AlertChannel for TelegramChannel {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        "Telegram"
    }

    fn send(&mut self, message: &str, _context: &AlertContext) -> Result<()> {
        let token = self.get_bot_token()?;
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        
        let client = reqwest::blocking::Client::new();
        let payload = serde_json::json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "Markdown"
        });

        client
            .post(&url)
            .json(&payload)
            .send()?;

        Ok(())
    }
}

/// Slack alert channel
#[allow(dead_code)] // Part of API, may be used in future
pub struct SlackChannel {
    id: String,
    webhook_url_keychain_account: String,
}

impl SlackChannel {
    #[allow(dead_code)] // Part of API, may be used in future
    pub fn new(id: String) -> Self {
        let webhook_url_keychain_account = format!("slack_webhook_{}", id);
        Self {
            id,
            webhook_url_keychain_account,
        }
    }

    #[allow(dead_code)] // Used internally, may be called in future
    fn get_webhook_url(&self) -> Result<String> {
        security::get_credential(&self.webhook_url_keychain_account)?
            .context("Slack webhook URL not found in Keychain")
    }
}

impl AlertChannel for SlackChannel {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        "Slack"
    }

    fn send(&mut self, message: &str, _context: &AlertContext) -> Result<()> {
        let webhook_url = self.get_webhook_url()?;
        
        let client = reqwest::blocking::Client::new();
        let payload = serde_json::json!({
            "text": message
        });

        client
            .post(&webhook_url)
            .json(&payload)
            .send()?;

        Ok(())
    }
}

/// Mastodon alert channel
#[allow(dead_code)] // Part of API, may be used in future
pub struct MastodonChannel {
    id: String,
    instance_url: String,
    api_token_keychain_account: String,
}

impl MastodonChannel {
    #[allow(dead_code)] // Part of API, may be used in future
    pub fn new(id: String, instance_url: String) -> Self {
        let api_token_keychain_account = format!("mastodon_alert_{}", id);
        Self {
            id,
            instance_url,
            api_token_keychain_account,
        }
    }

    #[allow(dead_code)] // Used internally, may be called in future
    fn get_api_token(&self) -> Result<String> {
        security::get_credential(&self.api_token_keychain_account)?
            .context("Mastodon API token not found in Keychain")
    }
}

impl AlertChannel for MastodonChannel {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        "Mastodon"
    }

    fn send(&mut self, message: &str, _context: &AlertContext) -> Result<()> {
        let token = self.get_api_token()?;
        let url = format!("{}/api/v1/statuses", self.instance_url);
        
        let client = reqwest::blocking::Client::new();
        let payload = serde_json::json!({
            "status": message,
            "visibility": "direct" // Private toot
        });

        client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()?;

        Ok(())
    }
}

/// Signal alert channel (placeholder - requires Signal API setup)
#[allow(dead_code)] // Part of API, may be used in future
pub struct SignalChannel {
    id: String,
}

impl SignalChannel {
    #[allow(dead_code)] // Part of API, may be used in future
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl AlertChannel for SignalChannel {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        "Signal"
    }

    fn send(&mut self, _message: &str, _context: &AlertContext) -> Result<()> {
        // Signal requires Signal REST API or Signal CLI
        // This is a placeholder implementation
        Err(anyhow::anyhow!("Signal channel not yet implemented - requires Signal REST API setup"))
    }
}
