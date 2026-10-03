use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_llm);

#[cfg(not(target_os = "ios"))]
compile_error!("tauri-plugin-llm solo tiene implementación para iOS");

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Llm<R>> {
    let handle = api.register_ios_plugin(init_plugin_llm)?;
    Ok(Llm(handle))
}

/// Acceso al motor de llama.cpp (código Swift en `ios/`).
pub struct Llm<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Llm<R> {
    pub async fn status(&self) -> crate::Result<EngineStatus> {
        Ok(self.0.run_mobile_plugin_async("status", json!({})).await?)
    }

    pub async fn load(&self, request: LoadRequest) -> crate::Result<LoadInfo> {
        Ok(self.0.run_mobile_plugin_async("load", request).await?)
    }

    pub async fn unload(&self) -> crate::Result<()> {
        let _: Value = self.0.run_mobile_plugin_async("unload", json!({})).await?;
        Ok(())
    }

    pub async fn bench(&self, request: BenchRequest) -> crate::Result<BenchResult> {
        Ok(self.0.run_mobile_plugin_async("bench", request).await?)
    }

    pub async fn generate(&self, request: GenerateRequest) -> crate::Result<GenerateResult> {
        Ok(self.0.run_mobile_plugin_async("generate", request).await?)
    }

    pub async fn cancel(&self) -> crate::Result<()> {
        let _: Value = self.0.run_mobile_plugin_async("cancel", json!({})).await?;
        Ok(())
    }

    /// Devuelve la ruta final del modelo ya verificado.
    pub async fn download(&self, request: DownloadRequest) -> crate::Result<String> {
        let reply: Value = self.0.run_mobile_plugin_async("download", request).await?;
        Ok(reply.get("path").and_then(Value::as_str).unwrap_or_default().to_string())
    }

    pub async fn cancel_download(&self) -> crate::Result<()> {
        let _: Value = self.0.run_mobile_plugin_async("cancelDownload", json!({})).await?;
        Ok(())
    }

    pub async fn delete_model(&self, file: &str) -> crate::Result<()> {
        let _: Value = self
            .0
            .run_mobile_plugin_async("deleteModel", json!({ "file": file }))
            .await?;
        Ok(())
    }

    /// Solo depuración: `memoryWarning` o `resignActive`.
    pub async fn debug_simulate(&self, event: &str) -> crate::Result<()> {
        let _: Value = self
            .0
            .run_mobile_plugin_async("debugSimulate", json!({ "event": event }))
            .await?;
        Ok(())
    }

    pub async fn keep_awake(&self, enabled: bool) -> crate::Result<()> {
        let _: Value = self
            .0
            .run_mobile_plugin_async("keepAwake", json!({ "enabled": enabled }))
            .await?;
        Ok(())
    }
}
