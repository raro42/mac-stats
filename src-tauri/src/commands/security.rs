//! Security/Keychain Tauri commands

use crate::security;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreCredentialRequest {
    pub account: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCredentialRequest {
    pub account: String,
}

/// Store a credential in Keychain
#[tauri::command]
pub fn store_credential(request: StoreCredentialRequest) -> Result<(), String> {
    security::store_credential(&request.account, &request.password)
        .map_err(|e| e.to_string())
}

/// Get a credential from Keychain
#[tauri::command]
pub fn get_credential(request: GetCredentialRequest) -> Result<Option<String>, String> {
    security::get_credential(&request.account)
        .map_err(|e| e.to_string())
}

/// Delete a credential from Keychain
#[tauri::command]
pub fn delete_credential(account: String) -> Result<(), String> {
    security::delete_credential(&account)
        .map_err(|e| e.to_string())
}

/// List all stored credentials
#[tauri::command]
pub fn list_credentials() -> Result<Vec<String>, String> {
    security::list_credentials()
        .map_err(|e| e.to_string())
}
