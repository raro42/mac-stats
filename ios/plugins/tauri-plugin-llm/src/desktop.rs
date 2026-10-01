use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;
use crate::Error;

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Llm<R>> {
    Ok(Llm(PhantomData))
}

/// En escritorio no hay motor: todo devuelve `Unsupported` (sirve para `cargo test`).
pub struct Llm<R: Runtime>(PhantomData<fn() -> R>);

impl<R: Runtime> Llm<R> {
    pub async fn status(&self) -> crate::Result<EngineStatus> {
        Ok(EngineStatus::default())
    }

    pub async fn load(&self, _request: LoadRequest) -> crate::Result<LoadInfo> {
        Err(Error::Unsupported)
    }

    pub async fn unload(&self) -> crate::Result<()> {
        Ok(())
    }

    pub async fn bench(&self, _request: BenchRequest) -> crate::Result<BenchResult> {
        Err(Error::Unsupported)
    }

    pub async fn generate(&self, _request: GenerateRequest) -> crate::Result<GenerateResult> {
        Err(Error::Unsupported)
    }

    pub async fn cancel(&self) -> crate::Result<()> {
        Ok(())
    }

    pub async fn download(&self, _request: DownloadRequest) -> crate::Result<String> {
        Err(Error::Unsupported)
    }

    pub async fn cancel_download(&self) -> crate::Result<()> {
        Ok(())
    }

    pub async fn delete_model(&self, _file: &str) -> crate::Result<()> {
        Err(Error::Unsupported)
    }

    pub async fn debug_simulate(&self, _event: &str) -> crate::Result<()> {
        Err(Error::Unsupported)
    }

    pub async fn keep_awake(&self, _enabled: bool) -> crate::Result<()> {
        Ok(())
    }
}
