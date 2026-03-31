use anyhow::{Result, anyhow};
use credentials_provider::CredentialsProvider;
use gpui::{App, AsyncApp};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredOAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at_unix: Option<i64>,
    pub token_type: Option<String>,
}

pub struct CredentialManager;

impl CredentialManager {
    pub fn scoped_url(provider_id: &str, purpose: &str) -> String {
        format!("zed://language-models/{provider_id}/{purpose}")
    }

    pub async fn read_secret(url: &str, cx: &AsyncApp) -> Result<Option<Vec<u8>>> {
        let provider: Arc<dyn CredentialsProvider> =
            cx.update(|cx| <dyn CredentialsProvider>::global(cx));
        Ok(provider
            .read_credentials(url, cx)
            .await?
            .map(|(_, value)| value))
    }

    pub async fn read_basic(url: &str, cx: &AsyncApp) -> Result<Option<(String, String)>> {
        let provider: Arc<dyn CredentialsProvider> =
            cx.update(|cx| <dyn CredentialsProvider>::global(cx));
        let credential = provider.read_credentials(url, cx).await?;
        credential
            .map(|(username, password)| {
                String::from_utf8(password)
                    .map(|password| (username, password))
                    .map_err(|err| anyhow!("stored credentials are not utf-8: {err}"))
            })
            .transpose()
    }

    pub async fn read_oauth_token(url: &str, cx: &AsyncApp) -> Result<Option<StoredOAuthToken>> {
        let Some(bytes) = Self::read_secret(url, cx).await? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    pub fn read_env(name: &Option<String>) -> Option<String> {
        let name = name.as_ref()?;
        std::env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
    }

    pub fn read_env_required(name: &Option<String>, what: &str) -> Result<String> {
        Self::read_env(name).ok_or_else(|| anyhow!("missing {what} environment variable"))
    }

    pub fn global(_cx: &App) -> Self {
        Self
    }
}
