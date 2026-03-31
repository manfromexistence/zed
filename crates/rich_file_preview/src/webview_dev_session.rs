use std::path::PathBuf;

use crate::browser_extensions::{BrowserProfile, sanitize_component};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevSessionMode {
    SharedWorkspace,
    IsolatedPerOrigin,
    Incognito,
}

impl DevSessionMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SharedWorkspace => "Shared",
            Self::IsolatedPerOrigin => "Per Origin",
            Self::Incognito => "Incognito",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::SharedWorkspace => Self::IsolatedPerOrigin,
            Self::IsolatedPerOrigin => Self::Incognito,
            Self::Incognito => Self::SharedWorkspace,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevSessionPolicy {
    pub mode: DevSessionMode,
    pub auto_clear_local_auth: bool,
}

impl Default for DevSessionPolicy {
    fn default() -> Self {
        Self {
            mode: DevSessionMode::IsolatedPerOrigin,
            auto_clear_local_auth: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDevSession {
    pub data_directory: PathBuf,
    pub incognito: bool,
    pub should_clear_before_navigation: bool,
    pub is_local_development: bool,
    pub origin_key: String,
    pub description: String,
}

impl DevSessionPolicy {
    pub fn resolve_for_url(
        &self,
        url: &str,
        browser_profile: Option<&BrowserProfile>,
        importing_extensions: bool,
    ) -> ResolvedDevSession {
        let origin_key = origin_key(url);
        let is_local_development = is_local_development_url(url);
        let browser_key = browser_profile
            .map(BrowserProfile::profile_key)
            .unwrap_or_else(|| "clean-preview".to_string());
        let extension_key = if importing_extensions {
            "with-ext"
        } else {
            "no-ext"
        };

        let base = paths::data_dir()
            .join("web_preview")
            .join("sessions")
            .join(browser_key)
            .join(extension_key);

        let (data_directory, incognito, description) = match self.mode {
            DevSessionMode::SharedWorkspace => (
                base.join("shared"),
                false,
                "Shared editor-wide browsing context".to_string(),
            ),
            DevSessionMode::IsolatedPerOrigin => (
                base.join("origin").join(&origin_key),
                false,
                format!("Isolated session for origin {origin_key}"),
            ),
            DevSessionMode::Incognito => (
                base.join("incognito").join(&origin_key),
                true,
                format!("Incognito session for origin {origin_key}"),
            ),
        };

        ResolvedDevSession {
            data_directory,
            incognito,
            should_clear_before_navigation: self.auto_clear_local_auth && is_local_development,
            is_local_development,
            origin_key,
            description,
        }
    }
}

pub fn origin_key(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return "blank".to_string();
    }

    if let Some((scheme, rest)) = trimmed.split_once("://") {
        let authority = rest.split('/').next().unwrap_or(rest);
        return sanitize_component(&format!("{}-{}", scheme, authority));
    }

    sanitize_component(trimmed)
}

pub fn is_local_development_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("://localhost")
        || lower.contains("://127.0.0.1")
        || lower.contains("://0.0.0.0")
        || lower.contains("://[::1]")
        || lower.contains("://host.docker.internal")
        || lower.starts_with("http://localhost")
        || lower.starts_with("https://localhost")
}
