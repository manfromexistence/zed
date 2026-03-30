pub mod auth_strategy;
pub mod credential_manager;
pub mod provider_registry;

use language_model::LanguageModelProviderId;
use serde::{Deserialize, Serialize};

pub use provider_registry::{
    CatalogSyncState, ProviderHubSnapshot, ProviderHubStore, ProviderOverride, init,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCategory {
    Frontier,
    #[default]
    FastInference,
    Specialist,
    Enterprise,
    Local,
    RegionalChinese,
}

impl ProviderCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Frontier => "Frontier",
            Self::FastInference => "Fast Inference",
            Self::Specialist => "Specialist",
            Self::Enterprise => "Enterprise",
            Self::Local => "Local",
            Self::RegionalChinese => "Regional Chinese",
        }
    }

    pub fn sort_key(self) -> usize {
        match self {
            Self::Frontier => 0,
            Self::FastInference => 1,
            Self::Specialist => 2,
            Self::Enterprise => 3,
            Self::Local => 4,
            Self::RegionalChinese => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderTransport {
    #[default]
    OpenAiCompatible,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderIcon {
    OpenAi,
    OpenAiCompatible,
    OpenRouter,
    Anthropic,
    Google,
    Bedrock,
    Mistral,
    XAi,
    Copilot,
    #[default]
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderModelMetadata {
    pub context_window: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub input_cost_per_1m: Option<f64>,
    pub output_cost_per_1m: Option<f64>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    pub supports_reasoning: bool,
    pub supports_tool_calling: bool,
    pub supports_structured_output: bool,
    pub supports_vision: bool,
    pub supports_audio: bool,
    pub supports_video: bool,
    pub supports_pdf: bool,
    pub release_date: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderModelCapabilities {
    pub tools: bool,
    pub images: bool,
    pub parallel_tool_calls: bool,
    pub prompt_cache_key: bool,
    pub chat_completions: bool,
}

impl Default for ProviderModelCapabilities {
    fn default() -> Self {
        Self {
            tools: true,
            images: false,
            parallel_tool_calls: false,
            prompt_cache_key: false,
            chat_completions: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelManifest {
    pub id: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    #[serde(default)]
    pub capabilities: ProviderModelCapabilities,
    #[serde(default)]
    pub metadata: ProviderModelMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderManifest {
    pub id: String,
    pub display_name: String,
    pub api_base: String,
    #[serde(default)]
    pub category: ProviderCategory,
    #[serde(default)]
    pub icon: ProviderIcon,
    #[serde(default)]
    pub transport: ProviderTransport,
    #[serde(default)]
    pub featured_in_settings: bool,
    #[serde(default)]
    pub auth: auth_strategy::AuthStrategy,
    #[serde(default)]
    pub models: Vec<ModelManifest>,
}

impl ProviderManifest {
    pub fn provider_id(&self) -> LanguageModelProviderId {
        LanguageModelProviderId::from(self.id.clone())
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    pub fn should_register(&self) -> bool {
        self.transport == ProviderTransport::OpenAiCompatible
            && !self.api_base.trim().is_empty()
            && !self.models.is_empty()
    }
}
