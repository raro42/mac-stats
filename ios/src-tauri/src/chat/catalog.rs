//! Catálogo de modelos (ios/models/catalog.json): URL fijadas a un commit de Hugging
//! Face, tamaño exacto y SHA-256 para verificar la descarga.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub commit: String,
    pub file: String,
    pub size: u64,
    pub sha256: String,
    pub license: String,
    pub think_prefill: bool,
}

impl CatalogEntry {
    pub fn url(&self) -> String {
        format!("https://huggingface.co/{}/resolve/{}/{}", self.repo, self.commit, self.file)
    }

    pub fn path(&self) -> PathBuf {
        models_dir().join(&self.file)
    }

    /// Está en el iPhone con el tamaño exacto (la verificación SHA-256 se hizo al bajarlo).
    pub fn installed(&self) -> bool {
        std::fs::metadata(self.path()).map(|m| m.len() == self.size).unwrap_or(false)
    }
}

pub fn catalog() -> Vec<CatalogEntry> {
    serde_json::from_str(include_str!("../../../models/catalog.json"))
        .expect("models/catalog.json no es válido")
}

pub fn find(id: &str) -> Option<CatalogEntry> {
    catalog().into_iter().find(|m| m.id == id)
}

/// `Library/Application Support/models/` del contenedor de la app (la misma carpeta
/// que usa el `ModelStore` de Swift).
pub fn models_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Application Support/models")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_valid_and_ids_are_unique() {
        let c = catalog();
        assert_eq!(c.len(), 3);
        let mut ids: Vec<_> = c.iter().map(|m| m.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3);
        assert!(c.iter().all(|m| m.sha256.len() == 64 && m.commit.len() == 40 && m.size > 0));
    }

    #[test]
    fn download_url_is_pinned_to_a_commit() {
        let m = find("qwen2.5-1.5b").unwrap();
        assert_eq!(
            m.url(),
            "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/91cad51170dc346986eccefdc2dd33a9da36ead9/qwen2.5-1.5b-instruct-q4_k_m.gguf"
        );
    }
}
