//! Security and credential management module
//!
//! Provides secure storage for API keys, tokens, and passwords using macOS Keychain.
//! All credentials are stored securely and never exposed in logs, UI, or files.
//! Account names are persisted in `~/.mac-stats/credential_accounts.json` so we can
//! list them without relying on Keychain attribute enumeration (which security_framework
//! does not expose for generic password items).
//!
//! **No-logging rule:** Do not log credential values or any buffer that might contain them
//! (e.g. request/response headers or bodies). Use `mask_credential` for safe display only.

pub mod attachment_roots;
pub mod host_exec_env;

use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use security_framework::passwords::delete_generic_password;
#[cfg(target_os = "macos")]
use security_framework::passwords::get_generic_password;
#[cfg(target_os = "macos")]
use security_framework::passwords::set_generic_password;
#[cfg(target_os = "macos")]
use security_framework_sys::base::errSecItemNotFound;
use std::fs;
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
#[cfg(target_os = "macos")]
pub fn store_credential(account: &str, password: &str) -> Result<()> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;

    // Delete existing credential if it exists (update operation)
    let _ = delete_credential(account);

    if let Err(e) = set_generic_password(KEYCHAIN_SERVICE, account, password.as_bytes()) {
        tracing::error!(
            "Keychain: set_generic_password failed for account '{}' (service '{}'): {:?}",
            account,
            KEYCHAIN_SERVICE,
            e
        );
        return Err(anyhow::anyhow!(
            "Failed to store credential in Keychain: {:?}",
            e
        ));
    }

    tracing::info!(
        "Keychain: stored credential for account '{}' (service '{}')",
        account,
        KEYCHAIN_SERVICE
    );
    add_credential_account_to_list(account)?;
    Ok(())
}

/// Retrieve a credential from macOS Keychain (backend use only).
///
/// **Security:** Must never be exposed to the frontend as a Tauri command.
///
/// # Arguments
/// * `account` - Unique identifier for the credential
///
/// # Returns
/// Ok(Some(String)) if found, Ok(None) if not found, Err on error
#[cfg(target_os = "macos")]
pub fn get_credential(account: &str) -> Result<Option<String>> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;

    match get_generic_password(KEYCHAIN_SERVICE, account) {
        Ok(password_bytes) => {
            let password = String::from_utf8(password_bytes)
                .context("Failed to convert credential to UTF-8 string")?;
            tracing::trace!(
                target: "mac_stats::security",
                "Keychain: retrieved credential for account '{}' (preview {})",
                account,
                mask_credential(&password)
            );
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
#[cfg(target_os = "macos")]
pub fn delete_credential(account: &str) -> Result<()> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;

    match delete_generic_password(KEYCHAIN_SERVICE, account) {
        Ok(()) => {
            tracing::debug!("Credential deleted for account: {}", account);
            remove_credential_account_from_list(account)?;
            Ok(())
        }
        Err(e) => {
            if e.code() == errSecItemNotFound {
                // Not an error if it doesn't exist; still sync list in case it was present
                let _ = remove_credential_account_from_list(account);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to delete credential: {:?}", e))
            }
        }
    }
}

/// Linux secret file. Mode `0600`. Account names cannot contain a path separator.
#[cfg(not(target_os = "macos"))]
fn linux_secret_path(account: &str) -> Result<std::path::PathBuf> {
    if account.is_empty()
        || account.contains('/')
        || account.contains('\\')
        || account.contains("..")
    {
        return Err(anyhow::anyhow!("invalid credential account name"));
    }
    let path = credential_accounts_path();
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("credential directory missing"))?
        .join("secrets");
    Ok(dir.join(account))
}

#[cfg(not(target_os = "macos"))]
pub fn store_credential(account: &str, password: &str) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("credential lock poisoned: {:?}", e))?;
    let path = linux_secret_path(account)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create secrets directory")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
        }
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)
        .context("Failed to open secret file")?;
    file.write_all(password.as_bytes())
        .context("Failed to write secret file")?;
    tracing::info!(
        "Secrets: stored credential for account '{}' (file mode 0600)",
        account
    );
    add_credential_account_to_list(account)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn get_credential(account: &str) -> Result<Option<String>> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("credential lock poisoned: {:?}", e))?;
    let path = linux_secret_path(account)?;
    if !path.exists() {
        return Ok(None);
    }
    let password = fs::read_to_string(&path).context("Failed to read secret file")?;
    tracing::trace!(
        target: "mac_stats::security",
        "Secrets: retrieved credential for account '{}' (preview {})",
        account,
        mask_credential(&password)
    );
    Ok(Some(password))
}

#[cfg(not(target_os = "macos"))]
pub fn delete_credential(account: &str) -> Result<()> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("credential lock poisoned: {:?}", e))?;
    let path = linux_secret_path(account)?;
    match fs::remove_file(&path) {
        Ok(()) => {
            tracing::debug!("Secret deleted for account: {}", account);
            remove_credential_account_from_list(account)?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let _ = remove_credential_account_from_list(account);
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to delete secret: {}", e)),
    }
}

/// List all stored credential accounts (backend use only).
///
/// **Security:** Must never be exposed to the frontend as a Tauri command.
/// Account names plus get_credential would allow enumeration and theft of all
/// stored credentials from the renderer.
///
/// Reads the persisted list at `~/.mac-stats/credential_accounts.json` (updated on store/delete).
///
/// # Returns
/// Vector of account names stored in Keychain
pub fn list_credentials() -> Result<Vec<String>> {
    let _lock = KEYCHAIN_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("Keychain lock poisoned: {:?}", e))?;

    read_credential_accounts_list()
}

fn credential_accounts_path() -> std::path::PathBuf {
    crate::config::Config::credential_accounts_file_path()
}

/// Read the persisted list of credential account names.
fn read_credential_accounts_list() -> Result<Vec<String>> {
    let path = credential_accounts_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).context("Failed to read credential accounts file")?;
    let accounts: Vec<String> =
        serde_json::from_str(&content).context("Failed to parse credential_accounts.json")?;
    Ok(accounts)
}

/// Ensure parent directory exists and write the accounts list (JSON array).
fn write_credential_accounts_list(accounts: &[String]) -> Result<()> {
    let path = credential_accounts_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create .mac-stats directory")?;
    }
    let content =
        serde_json::to_string_pretty(accounts).context("Failed to serialize accounts list")?;
    crate::config::write_text_atomic(&path, &content)
        .map_err(|e| anyhow::anyhow!("Failed to write credential_accounts.json: {}", e))?;
    Ok(())
}

fn add_credential_account_to_list(account: &str) -> Result<()> {
    let mut accounts = read_credential_accounts_list()?;
    if !accounts.contains(&account.to_string()) {
        accounts.push(account.to_string());
        write_credential_accounts_list(&accounts)?;
    }
    Ok(())
}

fn remove_credential_account_from_list(account: &str) -> Result<()> {
    let mut accounts = read_credential_accounts_list()?;
    accounts.retain(|a| a != account);
    write_credential_accounts_list(&accounts)?;
    Ok(())
}

/// Mask a credential for safe display only (shows only first/last few characters).
/// Never log raw credentials or headers/bodies that may contain them.
pub fn mask_credential(credential: &str) -> String {
    if credential.len() <= 8 {
        return "****".to_string();
    }
    format!(
        "{}...{}",
        &credential[..4],
        &credential[credential.len() - 4..]
    )
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
