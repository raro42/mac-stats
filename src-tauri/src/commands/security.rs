//! Security/Keychain Tauri commands.
//!
//! **Exposed to frontend:** only `store_credential` and `delete_credential`.
//! Do **not** add `get_credential` or `list_credentials` as Tauri commands:
//! they would allow credential enumeration and theft from the renderer.

use crate::security;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreCredentialRequest {
    pub account: String,
    pub password: String,
}

/// Store a credential in Keychain
#[tauri::command]
pub fn store_credential(request: StoreCredentialRequest) -> Result<(), String> {
    security::store_credential(&request.account, &request.password)
        .map_err(|e| e.to_string())
}

/// Delete a credential from Keychain
#[tauri::command]
pub fn delete_credential(account: String) -> Result<(), String> {
    security::delete_credential(&account)
        .map_err(|e| e.to_string())
}
