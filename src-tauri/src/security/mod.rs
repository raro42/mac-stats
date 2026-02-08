//! Security and credential management module
//! 
//! Provides secure storage for API keys, tokens, and passwords using macOS Keychain.
//! All credentials are stored securely and never exposed in logs, UI, or files.

use security_framework::item::{ItemClass, ItemSearchOptions, SearchResult};
use security_framework::passwords::set_generic_password;
use security_framework::passwords::get_generic_password;
use security_framework::passwords::delete_generic_password;
use security_framework_sys::base::errSecItemNotFound;
use anyhow::{Result, Context};
use std::sync::Mutex;

// Service name for Keychain items (unique identifier for our app)
const KEYCHAIN_SERVICE: &str = "com.raro42.mac-stats";

// Global lock for Keychain operations (Keychain API is not thread-safe)
static KEYCHAIN_LOCK: Mutex<()> = Mutex::new(());

/// Store a credential securely in macOS Keychain
/// 
/// # Arguments
/// * `account` - Unique identifier for the credential (e.g., "telegram_bot_token", "mastodon_api_key")
/// * `password` - The secret value to store (API key, token, password)
/// 
/// # Returns
/// Ok(()) on success, Err on failure
pub fn store_credential(account: &str, password: &str) -> Result<()> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;
    
    // Delete existing credential if it exists (update operation)
    let _ = delete_credential(account);
    
    if let Err(e) = set_generic_password(KEYCHAIN_SERVICE, account, password.as_bytes()) {
        tracing::error!("Keychain: set_generic_password failed for account '{}' (service '{}'): {:?}", account, KEYCHAIN_SERVICE, e);
        return Err(anyhow::anyhow!("Failed to store credential in Keychain: {:?}", e));
    }
    
    tracing::info!("Keychain: stored credential for account '{}' (service '{}')", account, KEYCHAIN_SERVICE);
    Ok(())
}

/// Retrieve a credential from macOS Keychain
/// 
/// # Arguments
/// * `account` - Unique identifier for the credential
/// 
/// # Returns
/// Ok(Some(String)) if found, Ok(None) if not found, Err on error
pub fn get_credential(account: &str) -> Result<Option<String>> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;
    
    match get_generic_password(KEYCHAIN_SERVICE, account) {
        Ok(password_bytes) => {
            let password = String::from_utf8(password_bytes)
                .context("Failed to convert credential to UTF-8 string")?;
            Ok(Some(password))
        }
        Err(e) => {
            if e.code() == errSecItemNotFound {
                Ok(None)
            } else {
                Err(anyhow::anyhow!("Failed to retrieve credential: {:?}", e))
            }
        }
    }
}

/// Delete a credential from macOS Keychain
/// 
/// # Arguments
/// * `account` - Unique identifier for the credential
/// 
/// # Returns
/// Ok(()) on success (even if credential didn't exist), Err on error
pub fn delete_credential(account: &str) -> Result<()> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;
    
    match delete_generic_password(KEYCHAIN_SERVICE, account) {
        Ok(()) => {
            tracing::debug!("Credential deleted for account: {}", account);
            Ok(())
        }
        Err(e) => {
            if e.code() == errSecItemNotFound {
                // Not an error if it doesn't exist
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to delete credential: {:?}", e))
            }
        }
    }
}

/// List all stored credential accounts
/// 
/// # Returns
/// Vector of account names stored in Keychain
pub fn list_credentials() -> Result<Vec<String>> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;
    
    let mut accounts = Vec::new();
    
    // Search for all generic passwords with our service name
    let mut search_options = ItemSearchOptions::new();
    search_options.class(ItemClass::generic_password());
    search_options.service(KEYCHAIN_SERVICE);
    
    let results = search_options.search()?;
    
    for result in results {
        if let SearchResult::Ref(item_ref) = result {
            // Extract account name from the item
            // Note: This is a simplified approach - in production, you'd want to
            // properly extract attributes from the SecKeychainItem
            if let Ok(Some(account)) = get_account_from_item(&item_ref) {
                accounts.push(account);
            }
        }
    }
    
    Ok(accounts)
}

/// Helper function to extract account name from Keychain item
/// This is a simplified implementation - in production, use proper Keychain API
fn get_account_from_item(_item_ref: &security_framework::item::Reference) -> Result<Option<String>> {
    // TODO: Implement proper account extraction from Keychain item
    // For now, we'll use a different approach: store account names separately
    // or use a different Keychain API that provides account names
    Ok(None)
}

/// Mask a credential for logging (shows only first/last few characters)
pub fn mask_credential(credential: &str) -> String {
    if credential.len() <= 8 {
        return "****".to_string();
    }
    format!("{}...{}", &credential[..4], &credential[credential.len()-4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_credential() {
        assert_eq!(mask_credential("short"), "****");
        assert_eq!(mask_credential("verylongtoken12345"), "very...2345");
    }
}
