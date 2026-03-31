use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result};
use collections::HashMap;
use gpui::{App, AppContext as _, Context, Entity, Global, SharedString, Task, WeakEntity};
use http_client::HttpClient;
use language_model::{LanguageModelProviderId, LanguageModelRegistry};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use settings::{Settings as _, SettingsStore};

use crate::provider::manifest::ManifestLanguageModelProvider;
use crate::provider_hub::auth_strategy::{ApiKeyHeaderStyle, AuthStrategy};
use crate::provider_hub::{
    ModelManifest, ProviderCategory, ProviderIcon, ProviderManifest, ProviderModelCapabilities,
    ProviderTransport,
};
use crate::provider_icons;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderHubSnapshot {
    pub providers: BTreeMap<String, ProviderManifest>,
}

impl ProviderHubSnapshot {
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    pub fn model_count(&self) -> usize {
        self.providers
            .values()
            .map(|provider| provider.models.len())
            .sum()
    }

    pub fn provider(&self, provider_id: &str) -> Option<&ProviderManifest> {
        self.providers.get(provider_id)
    }

    pub fn model(&self, provider_id: &str, model_id: &str) -> Option<&ModelManifest> {
        self.provider(provider_id)?
            .models
            .iter()
            .find(|model| model.id == model_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogSyncState {
    Idle,
    Syncing,
    Ready {
        provider_count: usize,
        model_count: usize,
        cached: bool,
    },
    Error(SharedString),
}

impl Default for CatalogSyncState {
    fn default() -> Self {
        Self::Idle
    }
}

impl CatalogSyncState {
    pub fn label(&self) -> Option<SharedString> {
        match self {
            Self::Idle => None,
            Self::Syncing => Some("Syncing provider catalog".into()),
            Self::Ready {
                provider_count,
                model_count,
                cached,
            } => Some(
                if *cached {
                    format!("{provider_count} providers / {model_count} models (cached)")
                } else {
                    format!("{provider_count} providers / {model_count} models")
                }
                .into(),
            ),
            Self::Error(message) => Some(format!("Provider sync failed: {message}").into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderOverride {
    pub category: Option<ProviderCategory>,
    pub icon: Option<ProviderIcon>,
    pub featured_in_settings: Option<bool>,
    pub auth: Option<AuthStrategy>,
    pub api_base: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Default)]
pub struct ProviderHubStore {
    http_client: Option<Arc<dyn HttpClient>>,
    sync_state: CatalogSyncState,
    snapshot: ProviderHubSnapshot,
    registered_provider_ids: BTreeSet<String>,
    refresh_task: Option<Task<()>>,
}

struct GlobalProviderHubStore(Entity<ProviderHubStore>);

impl Global for GlobalProviderHubStore {}

pub fn init(http_client: Arc<dyn HttpClient>, cx: &mut App) {
    ProviderHubStore::init(http_client, cx);
}

impl ProviderHubStore {
    pub fn init(http_client: Arc<dyn HttpClient>, cx: &mut App) {
        let store = cx.new(|cx| {
            let weak_store: WeakEntity<Self> = cx.entity().downgrade();
            cx.observe_global::<SettingsStore>(move |_this: &mut Self, cx| {
                let weak_store = weak_store.clone();
                cx.defer(move |cx| {
                    if let Some(store) = weak_store.upgrade() {
                        let _ = store.update(cx, |this, cx| this.schedule_refresh(cx));
                    }
                });
            })
            .detach();

            Self {
                http_client: Some(http_client.clone()),
                ..Default::default()
            }
        });

        cx.set_global(GlobalProviderHubStore(store.clone()));
        cx.defer(move |cx| {
            if let Some(store) = ProviderHubStore::try_global(cx) {
                let _ = store.update(cx, |this, cx| this.schedule_refresh(cx));
            }
        });
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalProviderHubStore>().0.clone()
    }

    pub fn try_global(cx: &App) -> Option<Entity<Self>> {
        cx.try_global::<GlobalProviderHubStore>()
            .map(|g| g.0.clone())
    }

    pub fn refresh_global(cx: &mut App) {
        if let Some(store) = Self::try_global(cx) {
            store.update(cx, |this, cx| this.schedule_refresh(cx));
        }
    }

    pub fn snapshot(&self) -> &ProviderHubSnapshot {
        &self.snapshot
    }

    pub fn sync_state(&self) -> &CatalogSyncState {
        &self.sync_state
    }

    pub fn provider_manifest(&self, provider_id: &str) -> Option<&ProviderManifest> {
        self.snapshot.provider(provider_id)
    }

    pub fn featured_providers(&self) -> Vec<&ProviderManifest> {
        self.snapshot
            .providers
            .values()
            .filter(|provider| provider.featured_in_settings && provider.should_register())
            .collect()
    }

    pub fn provider_category(&self, provider_id: &LanguageModelProviderId) -> ProviderCategory {
        self.snapshot
            .provider(provider_id.0.as_ref())
            .map(|provider| provider.category)
            .unwrap_or_else(|| infer_category(provider_id.0.as_ref(), ""))
    }

    pub fn provider_display_name(&self, provider_id: &LanguageModelProviderId) -> Option<&str> {
        self.snapshot
            .provider(provider_id.0.as_ref())
            .map(|provider| provider.display_name.as_str())
    }

    pub fn model_manifest(
        &self,
        provider_id: &LanguageModelProviderId,
        model_id: &str,
    ) -> Option<&ModelManifest> {
        self.snapshot.model(provider_id.0.as_ref(), model_id)
    }

    fn schedule_refresh(&mut self, cx: &mut Context<Self>) {
        if self.refresh_task.is_some() {
            return;
        }

        self.sync_state = CatalogSyncState::Syncing;
        cx.notify();

        let litellm_config = litellm_configuration(cx);

        self.refresh_task = Some(cx.spawn(async move |this, cx| {
            let cached_snapshot = load_cached_snapshot().ok().flatten();

            if let Some(snapshot) = cached_snapshot {
                this.update(cx, |this, cx| {
                    if this.snapshot.providers.is_empty() && snapshot.provider_count() > 0 {
                        this.sync_state = CatalogSyncState::Ready {
                            provider_count: snapshot.provider_count(),
                            model_count: snapshot.model_count(),
                            cached: true,
                        };
                        this.snapshot = snapshot;
                        this.apply_registry(cx);
                        cx.notify();
                    }
                })
                .ok();
            }

            let snapshot = cx
                .background_executor()
                .spawn(async move {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .context("failed to create provider hub tokio runtime")?;
                    runtime.block_on(load_snapshot(litellm_config))
                })
                .await;

            this.update(cx, |this, cx| {
                this.refresh_task = None;
                match snapshot {
                    Ok(snapshot) => {
                        if let Err(error) = write_cached_snapshot(&snapshot) {
                            log::warn!("failed to cache provider hub snapshot: {error:#}");
                        }
                        this.sync_state = CatalogSyncState::Ready {
                            provider_count: snapshot.provider_count(),
                            model_count: snapshot.model_count(),
                            cached: false,
                        };
                        this.snapshot = snapshot;
                        this.apply_registry(cx);
                    }
                    Err(error) => {
                        log::error!("provider hub sync failed: {error:#}");
                        if this.snapshot.providers.is_empty() {
                            this.sync_state = CatalogSyncState::Error(error.to_string().into());
                        }
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn apply_registry(&mut self, cx: &mut Context<Self>) {
        let native_provider_ids = native_provider_ids();
        let http_client = self.http_client.clone();
        LanguageModelRegistry::global(cx).update(cx, |registry, cx| {
            for provider_id in std::mem::take(&mut self.registered_provider_ids) {
                registry.unregister_provider(LanguageModelProviderId::from(provider_id), cx);
            }

            let Some(http_client) = http_client.clone() else {
                return;
            };

            for provider in self.snapshot.providers.values() {
                if native_provider_ids.contains(provider.id.as_str()) || !provider.should_register()
                {
                    continue;
                }

                registry.register_provider(
                    Arc::new(ManifestLanguageModelProvider::new(
                        provider.clone(),
                        http_client.clone(),
                        cx,
                    )),
                    cx,
                );
                self.registered_provider_ids.insert(provider.id.clone());
            }
        });
    }
}

fn litellm_configuration(cx: &App) -> Option<(String, Option<String>)> {
    let settings = crate::AllLanguageModelSettings::get_global(cx);
    let litellm = settings.openai_compatible.get("litellm_proxy")?;
    let api_key = std::env::var("LITELLM_PROXY_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty());
    Some((litellm.api_url.clone(), api_key))
}

async fn load_snapshot(litellm: Option<(String, Option<String>)>) -> Result<ProviderHubSnapshot> {
    let reqwest = reqwest::Client::new();
    let overrides = provider_overrides();

    let mut providers = fetch_models_dev(&reqwest, &overrides).await?;
    enrich_from_openrouter(&reqwest, &mut providers).await?;
    if let Some((url, api_key)) = litellm {
        enrich_from_litellm(&reqwest, &url, api_key.as_deref(), &mut providers).await?;
    }

    Ok(ProviderHubSnapshot { providers })
}

fn cache_path() -> PathBuf {
    paths::data_dir()
        .join("language-models")
        .join("provider-hub-cache.json")
}

fn load_cached_snapshot() -> Result<Option<ProviderHubSnapshot>> {
    let path = cache_path();
    if !path.exists() {
        return Ok(None);
    }

    let snapshot = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read provider hub cache at {}", path.display()))?;
    Ok(Some(serde_json::from_str(&snapshot).with_context(
        || format!("failed to parse provider hub cache at {}", path.display()),
    )?))
}

fn write_cached_snapshot(snapshot: &ProviderHubSnapshot) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create provider hub cache directory {}",
                parent.display()
            )
        })?;
    }

    std::fs::write(&path, serde_json::to_vec_pretty(snapshot)?)
        .with_context(|| format!("failed to write provider hub cache to {}", path.display()))
}

async fn fetch_models_dev(
    reqwest: &reqwest::Client,
    overrides: &HashMap<&'static str, ProviderOverride>,
) -> Result<BTreeMap<String, ProviderManifest>> {
    let payload: Value = serde_json::from_str(
        &reqwest
            .get("https://models.dev/api.json")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?,
    )?;

    let Some(providers) = payload.as_object() else {
        anyhow::bail!("models.dev returned an unexpected payload");
    };

    let mut manifests = BTreeMap::new();

    for (raw_provider_id, provider_value) in providers {
        let Some(provider) = provider_value.as_object() else {
            continue;
        };
        let provider_id = canonical_provider_id(raw_provider_id);

        let override_ = overrides
            .get(provider_id.as_str())
            .or_else(|| overrides.get(raw_provider_id.as_str()));
        let api_base = override_
            .and_then(|override_| override_.api_base.clone())
            .or_else(|| {
                provider
                    .get("api")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_default();
        let display_name = override_
            .and_then(|override_| override_.display_name.clone())
            .or_else(|| {
                provider
                    .get("name")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .or_else(|| {
                provider_icons::get_provider_display_name(&provider_id).map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| title_case(&provider_id));
        let transport = infer_transport(provider, &api_base, override_);
        let category = override_
            .and_then(|override_| override_.category)
            .unwrap_or_else(|| infer_category(&provider_id, &api_base));
        let icon = override_
            .and_then(|override_| override_.icon)
            .unwrap_or_else(|| infer_icon(&provider_id));
        let featured_in_settings = override_
            .and_then(|override_| override_.featured_in_settings)
            .unwrap_or_else(|| is_featured_provider(&provider_id));
        let auth = override_
            .and_then(|override_| override_.auth.clone())
            .unwrap_or_else(|| AuthStrategy::ApiKey {
                header_style: ApiKeyHeaderStyle::Bearer,
                env_var: provider
                    .get("env")
                    .and_then(Value::as_array)
                    .and_then(|envs| envs.first())
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            });

        let mut models = Vec::new();
        if let Some(model_entries) = provider.get("models").and_then(Value::as_object) {
            for (model_id, model_value) in model_entries {
                let Some(model) = model_value.as_object() else {
                    continue;
                };

                let context_window = model
                    .get("limit")
                    .and_then(Value::as_object)
                    .and_then(|limit| limit.get("context"))
                    .and_then(Value::as_u64);
                let output_limit = model
                    .get("limit")
                    .and_then(Value::as_object)
                    .and_then(|limit| limit.get("output"))
                    .and_then(Value::as_u64);
                let input_modalities = model
                    .get("modalities")
                    .and_then(Value::as_object)
                    .and_then(|modalities| modalities.get("input"))
                    .and_then(string_array)
                    .unwrap_or_default();
                let output_modalities = model
                    .get("modalities")
                    .and_then(Value::as_object)
                    .and_then(|modalities| modalities.get("output"))
                    .and_then(string_array)
                    .unwrap_or_default();
                let supports_reasoning = model
                    .get("reasoning")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let supports_tool_calling = model
                    .get("tool_call")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let supports_vision = input_modalities
                    .iter()
                    .any(|modality| modality == "image" || modality == "video");
                let supports_audio = input_modalities
                    .iter()
                    .chain(output_modalities.iter())
                    .any(|modality| modality == "audio");
                let supports_video = input_modalities
                    .iter()
                    .chain(output_modalities.iter())
                    .any(|modality| modality == "video");
                let supports_pdf = input_modalities.iter().any(|modality| modality == "pdf");

                models.push(ModelManifest {
                    id: model_id.clone(),
                    display_name: model
                        .get("name")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    max_tokens: context_window.unwrap_or(128_000),
                    max_output_tokens: output_limit,
                    max_completion_tokens: output_limit,
                    capabilities: ProviderModelCapabilities {
                        tools: supports_tool_calling,
                        images: supports_vision,
                        parallel_tool_calls: supports_tool_calling,
                        prompt_cache_key: false,
                        chat_completions: true,
                    },
                    metadata: crate::provider_hub::ProviderModelMetadata {
                        context_window,
                        max_output_tokens: output_limit,
                        input_cost_per_1m: None,
                        output_cost_per_1m: None,
                        input_modalities,
                        output_modalities,
                        supports_reasoning,
                        supports_tool_calling,
                        supports_structured_output: supports_tool_calling,
                        supports_vision,
                        supports_audio,
                        supports_video,
                        supports_pdf,
                        release_date: model
                            .get("release_date")
                            .and_then(Value::as_str)
                            .map(ToOwned::to_owned),
                        last_updated: model
                            .get("last_updated")
                            .and_then(Value::as_str)
                            .map(ToOwned::to_owned),
                    },
                });
            }
        }

        manifests.insert(
            provider_id.clone(),
            ProviderManifest {
                id: provider_id.clone(),
                display_name,
                api_base,
                category,
                icon,
                transport,
                featured_in_settings,
                auth,
                models,
            },
        );
    }

    Ok(manifests)
}

async fn enrich_from_openrouter(
    reqwest: &reqwest::Client,
    providers: &mut BTreeMap<String, ProviderManifest>,
) -> Result<()> {
    let payload: Value = serde_json::from_str(
        &reqwest
            .get("https://openrouter.ai/api/v1/models")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?,
    )?;

    let Some(models) = payload.get("data").and_then(Value::as_array) else {
        return Ok(());
    };

    for model in models {
        let Some(model) = model.as_object() else {
            continue;
        };
        let Some(id) = model.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some((provider_id_raw, model_id)) = id.split_once('/') else {
            continue;
        };
        let provider_id = canonical_provider_id(provider_id_raw);

        let provider = providers
            .entry(provider_id.clone())
            .or_insert_with(|| ProviderManifest {
                id: provider_id.clone(),
                display_name: provider_icons::get_provider_display_name(&provider_id)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| title_case(&provider_id)),
                api_base: String::new(),
                category: infer_category(&provider_id, ""),
                icon: infer_icon(&provider_id),
                transport: ProviderTransport::Unsupported,
                featured_in_settings: is_featured_provider(&provider_id),
                auth: AuthStrategy::ApiKey {
                    header_style: ApiKeyHeaderStyle::Bearer,
                    env_var: Some(AuthStrategy::default_env_var_for_provider(&provider_id)),
                },
                models: Vec::new(),
            });

        let model_name = model
            .get("name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let context_window = model.get("context_length").and_then(Value::as_u64);
        let max_output_tokens = model
            .get("top_provider")
            .and_then(Value::as_object)
            .and_then(|provider| provider.get("max_completion_tokens"))
            .and_then(Value::as_u64);
        let input_modalities = model
            .get("architecture")
            .and_then(Value::as_object)
            .and_then(|architecture| architecture.get("input_modalities"))
            .and_then(string_array)
            .unwrap_or_default();
        let output_modalities = model
            .get("architecture")
            .and_then(Value::as_object)
            .and_then(|architecture| architecture.get("output_modalities"))
            .and_then(string_array)
            .unwrap_or_default();
        let supports_vision = input_modalities
            .iter()
            .any(|modality| modality == "image" || modality == "video");
        let supports_audio = input_modalities
            .iter()
            .chain(output_modalities.iter())
            .any(|modality| modality == "audio");
        let supports_video = input_modalities
            .iter()
            .chain(output_modalities.iter())
            .any(|modality| modality == "video");
        let supports_pdf = input_modalities.iter().any(|modality| modality == "pdf");
        let input_cost_per_1m = model
            .get("pricing")
            .and_then(Value::as_object)
            .and_then(|pricing| pricing.get("prompt"))
            .and_then(parse_f64_value);
        let output_cost_per_1m = model
            .get("pricing")
            .and_then(Value::as_object)
            .and_then(|pricing| pricing.get("completion"))
            .and_then(parse_f64_value);
        let supports_reasoning = model
            .get("supported_parameters")
            .and_then(Value::as_array)
            .map(|params| {
                params
                    .iter()
                    .any(|value| value.as_str() == Some("reasoning"))
            })
            .unwrap_or(false);

        let existing = provider
            .models
            .iter_mut()
            .find(|existing| existing.id == model_id);
        if let Some(existing) = existing {
            if existing.display_name.is_none() {
                existing.display_name = model_name;
            }
            if existing.metadata.context_window.is_none() {
                existing.metadata.context_window = context_window;
            }
            if existing.max_output_tokens.is_none() {
                existing.max_output_tokens = max_output_tokens;
            }
            if existing.max_completion_tokens.is_none() {
                existing.max_completion_tokens = max_output_tokens;
            }
            if existing.metadata.input_cost_per_1m.is_none() {
                existing.metadata.input_cost_per_1m = input_cost_per_1m;
            }
            if existing.metadata.output_cost_per_1m.is_none() {
                existing.metadata.output_cost_per_1m = output_cost_per_1m;
            }
            if existing.metadata.input_modalities.is_empty() {
                existing.metadata.input_modalities = input_modalities.clone();
            }
            if existing.metadata.output_modalities.is_empty() {
                existing.metadata.output_modalities = output_modalities.clone();
            }
            existing.metadata.supports_reasoning |= supports_reasoning;
            existing.metadata.supports_tool_calling |= existing.capabilities.tools;
            existing.metadata.supports_structured_output |= existing.capabilities.tools;
            existing.metadata.supports_vision |= supports_vision;
            existing.metadata.supports_audio |= supports_audio;
            existing.metadata.supports_video |= supports_video;
            existing.metadata.supports_pdf |= supports_pdf;
        } else {
            provider.models.push(ModelManifest {
                id: model_id.to_owned(),
                display_name: model_name,
                max_tokens: context_window.unwrap_or(128_000),
                max_output_tokens,
                max_completion_tokens: max_output_tokens,
                capabilities: ProviderModelCapabilities {
                    tools: false,
                    images: supports_vision,
                    parallel_tool_calls: false,
                    prompt_cache_key: false,
                    chat_completions: true,
                },
                metadata: crate::provider_hub::ProviderModelMetadata {
                    context_window,
                    max_output_tokens,
                    input_cost_per_1m,
                    output_cost_per_1m,
                    input_modalities,
                    output_modalities,
                    supports_reasoning,
                    supports_tool_calling: false,
                    supports_structured_output: false,
                    supports_vision,
                    supports_audio,
                    supports_video,
                    supports_pdf,
                    release_date: None,
                    last_updated: None,
                },
            });
        }
    }

    Ok(())
}

async fn enrich_from_litellm(
    reqwest: &reqwest::Client,
    api_base: &str,
    api_key: Option<&str>,
    providers: &mut BTreeMap<String, ProviderManifest>,
) -> Result<()> {
    let mut headers = HeaderMap::new();
    if let Some(api_key) = api_key {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .context("invalid LiteLLM authorization header")?,
        );
    }

    let payload = reqwest
        .get(format!("{}/models", api_base.trim_end_matches('/')))
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;
    let payload: Value = serde_json::from_str(&payload.text().await?)?;

    let model_entries = if let Some(entries) = payload.get("data").and_then(Value::as_array) {
        entries.clone()
    } else if let Some(entries) = payload.get("models").and_then(Value::as_array) {
        entries.clone()
    } else {
        Vec::new()
    };

    for model in model_entries {
        let Some(model) = model.as_object() else {
            continue;
        };
        let id = model
            .get("id")
            .or_else(|| model.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some((provider_id_raw, model_id)) = id.split_once('/') else {
            continue;
        };
        let provider_id = canonical_provider_id(provider_id_raw);

        let provider = providers
            .entry(provider_id.clone())
            .or_insert_with(|| ProviderManifest {
                id: provider_id.clone(),
                display_name: provider_icons::get_provider_display_name(&provider_id)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| title_case(&provider_id)),
                api_base: api_base.to_owned(),
                category: infer_category(&provider_id, api_base),
                icon: infer_icon(&provider_id),
                transport: ProviderTransport::OpenAiCompatible,
                featured_in_settings: false,
                auth: AuthStrategy::ApiKey {
                    header_style: ApiKeyHeaderStyle::Bearer,
                    env_var: Some("LITELLM_PROXY_API_KEY".to_owned()),
                },
                models: Vec::new(),
            });

        provider.api_base = api_base.to_owned();
        provider.transport = ProviderTransport::OpenAiCompatible;

        let context_window = model
            .get("context_window")
            .or_else(|| model.get("context_length"))
            .and_then(Value::as_u64);
        let max_output_tokens = model
            .get("max_output_tokens")
            .or_else(|| model.get("max_completion_tokens"))
            .and_then(Value::as_u64);

        if provider
            .models
            .iter()
            .all(|existing| existing.id != model_id)
        {
            provider.models.push(ModelManifest {
                id: model_id.to_owned(),
                display_name: model
                    .get("display_name")
                    .or_else(|| model.get("name"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                max_tokens: context_window.unwrap_or(128_000),
                max_output_tokens,
                max_completion_tokens: max_output_tokens,
                capabilities: ProviderModelCapabilities::default(),
                metadata: crate::provider_hub::ProviderModelMetadata {
                    context_window,
                    max_output_tokens,
                    ..Default::default()
                },
            });
        }
    }

    Ok(())
}

fn infer_transport(
    provider: &serde_json::Map<String, Value>,
    api_base: &str,
    override_: Option<&ProviderOverride>,
) -> ProviderTransport {
    if let Some(override_) = override_ {
        if let Some(api_base) = &override_.api_base {
            if !api_base.is_empty() {
                return ProviderTransport::OpenAiCompatible;
            }
        }
    }

    let npm = provider
        .get("npm")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let api_base = api_base.to_ascii_lowercase();

    if api_base.is_empty() {
        return ProviderTransport::Unsupported;
    }

    if npm.contains("openai")
        || api_base.ends_with("/v1")
        || api_base.contains("/openai/")
        || api_base.contains("openrouter.ai/api")
        || api_base.contains("openai.azure.com")
        || api_base.contains("services.ai.azure.com")
    {
        ProviderTransport::OpenAiCompatible
    } else {
        ProviderTransport::Unsupported
    }
}

fn native_provider_ids() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "anthropic",
        "amazon-bedrock",
        "bedrock",
        "google",
        "openai",
        "openrouter",
        "x_ai",
        "deepseek",
        "mistral",
        "lmstudio",
        "ollama",
        "opencode",
        "vercel",
        "vercel_ai_gateway",
        "copilot_chat",
        "zed.dev",
    ])
}

fn api_key_provider(
    category: ProviderCategory,
    icon: ProviderIcon,
    featured_in_settings: bool,
    header_style: ApiKeyHeaderStyle,
    env_var: &'static str,
    api_base: &'static str,
    display_name: &'static str,
) -> ProviderOverride {
    ProviderOverride {
        category: Some(category),
        icon: Some(icon),
        featured_in_settings: Some(featured_in_settings),
        auth: Some(AuthStrategy::ApiKey {
            header_style,
            env_var: Some(env_var.to_owned()),
        }),
        api_base: Some(api_base.to_owned()),
        display_name: Some(display_name.to_owned()),
    }
}

fn no_auth_provider(
    category: ProviderCategory,
    icon: ProviderIcon,
    featured_in_settings: bool,
    api_base: &'static str,
    display_name: &'static str,
) -> ProviderOverride {
    ProviderOverride {
        category: Some(category),
        icon: Some(icon),
        featured_in_settings: Some(featured_in_settings),
        auth: Some(AuthStrategy::NoAuth),
        api_base: Some(api_base.to_owned()),
        display_name: Some(display_name.to_owned()),
    }
}

fn provider_overrides() -> HashMap<&'static str, ProviderOverride> {
    HashMap::from_iter([
        (
            "openai",
            api_key_provider(
                ProviderCategory::Frontier,
                ProviderIcon::OpenAi,
                true,
                ApiKeyHeaderStyle::Bearer,
                "OPENAI_API_KEY",
                "https://api.openai.com/v1",
                "OpenAI",
            ),
        ),
        (
            "anthropic",
            api_key_provider(
                ProviderCategory::Frontier,
                ProviderIcon::Anthropic,
                true,
                ApiKeyHeaderStyle::XApiKey,
                "ANTHROPIC_API_KEY",
                "https://api.anthropic.com",
                "Anthropic",
            ),
        ),
        (
            "google",
            api_key_provider(
                ProviderCategory::Frontier,
                ProviderIcon::Google,
                true,
                ApiKeyHeaderStyle::Bearer,
                "GOOGLE_API_KEY",
                "https://generativelanguage.googleapis.com",
                "Google AI",
            ),
        ),
        (
            "github-models",
            api_key_provider(
                ProviderCategory::Frontier,
                ProviderIcon::Copilot,
                true,
                ApiKeyHeaderStyle::Bearer,
                "GITHUB_TOKEN",
                "https://models.inference.ai.azure.com",
                "GitHub Models",
            ),
        ),
        (
            "github_models",
            api_key_provider(
                ProviderCategory::Frontier,
                ProviderIcon::Copilot,
                true,
                ApiKeyHeaderStyle::Bearer,
                "GITHUB_TOKEN",
                "https://models.inference.ai.azure.com",
                "GitHub Models",
            ),
        ),
        (
            "openrouter",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenRouter,
                true,
                ApiKeyHeaderStyle::Bearer,
                "OPENROUTER_API_KEY",
                "https://openrouter.ai/api/v1",
                "OpenRouter",
            ),
        ),
        (
            "groq",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                true,
                ApiKeyHeaderStyle::Bearer,
                "GROQ_API_KEY",
                "https://api.groq.com/openai/v1",
                "Groq",
            ),
        ),
        (
            "together",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                true,
                ApiKeyHeaderStyle::Bearer,
                "TOGETHER_API_KEY",
                "https://api.together.xyz/v1",
                "Together AI",
            ),
        ),
        (
            "fireworks-ai",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                true,
                ApiKeyHeaderStyle::Bearer,
                "FIREWORKS_API_KEY",
                "https://api.fireworks.ai/inference/v1",
                "Fireworks AI",
            ),
        ),
        (
            "perplexity",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                true,
                ApiKeyHeaderStyle::Bearer,
                "PERPLEXITY_API_KEY",
                "https://api.perplexity.ai",
                "Perplexity",
            ),
        ),
        (
            "nvidia_nim",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "NVIDIA_NIM_API_KEY",
                "https://integrate.api.nvidia.com/v1",
                "NVIDIA NIM",
            ),
        ),
        (
            "cerebras",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "CEREBRAS_API_KEY",
                "https://api.cerebras.ai/v1",
                "Cerebras",
            ),
        ),
        (
            "deepinfra",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                true,
                ApiKeyHeaderStyle::Bearer,
                "DEEPINFRA_API_KEY",
                "https://api.deepinfra.com/v1/openai",
                "DeepInfra",
            ),
        ),
        (
            "lepton",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "LEPTON_API_KEY",
                "https://api.lepton.ai/api/v1",
                "Lepton",
            ),
        ),
        (
            "anyscale",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "ANYSCALE_API_KEY",
                "https://api.endpoints.anyscale.com/v1",
                "Anyscale",
            ),
        ),
        (
            "replicate",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "REPLICATE_API_KEY",
                "https://api.replicate.com/v1",
                "Replicate",
            ),
        ),
        (
            "baseten",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "BASETEN_API_KEY",
                "https://bridge.baseten.co/v1",
                "Baseten",
            ),
        ),
        (
            "bytez",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "BYTEZ_API_KEY",
                "https://api.bytez.com",
                "Bytez",
            ),
        ),
        (
            "friendliai",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "FRIENDLI_API_KEY",
                "https://inference.friendli.ai/v1",
                "FriendliAI",
            ),
        ),
        (
            "friendli",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "FRIENDLI_API_KEY",
                "https://inference.friendli.ai/v1",
                "FriendliAI",
            ),
        ),
        (
            "aiml",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "AIML_API_KEY",
                "https://api.aimlapi.com/v1",
                "AIML API",
            ),
        ),
        (
            "302ai",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "AI302_API_KEY",
                "https://api.302.ai/v1",
                "302.AI",
            ),
        ),
        (
            "cloudflare_workers_ai",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "CLOUDFLARE_API_TOKEN",
                "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai",
                "Cloudflare Workers AI",
            ),
        ),
        (
            "cloudflare_ai_gateway",
            api_key_provider(
                ProviderCategory::FastInference,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "CLOUDFLARE_API_TOKEN",
                "https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_id}",
                "Cloudflare AI Gateway",
            ),
        ),
        (
            "cohere",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "COHERE_API_KEY",
                "https://api.cohere.ai/v2",
                "Cohere",
            ),
        ),
        (
            "huggingface",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "HUGGINGFACE_API_KEY",
                "https://api-inference.huggingface.co/v1",
                "Hugging Face",
            ),
        ),
        (
            "elevenlabs",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "ELEVENLABS_API_KEY",
                "https://api.elevenlabs.io/v1",
                "ElevenLabs",
            ),
        ),
        (
            "deepgram",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "DEEPGRAM_API_KEY",
                "https://api.deepgram.com/v1",
                "Deepgram",
            ),
        ),
        (
            "fal",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "FAL_KEY",
                "https://fal.run",
                "fal",
            ),
        ),
        (
            "fal_ai",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "FAL_KEY",
                "https://fal.run",
                "fal",
            ),
        ),
        (
            "black-forest-labs",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "BFL_API_KEY",
                "https://api.bfl.ml/v1",
                "Black Forest Labs",
            ),
        ),
        (
            "stability",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "STABILITY_API_KEY",
                "https://api.stability.ai/v2",
                "Stability AI",
            ),
        ),
        (
            "runway",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "RUNWAY_API_KEY",
                "https://api.runwayml.com/v1",
                "Runway",
            ),
        ),
        (
            "pika",
            api_key_provider(
                ProviderCategory::Specialist,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "PIKA_API_KEY",
                "https://api.pika.art/v1",
                "Pika",
            ),
        ),
        (
            "azure_ai_foundry",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "AZURE_AI_FOUNDRY_API_KEY",
                "https://{project}.services.ai.azure.com",
                "Azure AI Foundry",
            ),
        ),
        (
            "oci_genai",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "OCI_GENAI_API_KEY",
                "https://inference.generativeai.{region}.oci.oraclecloud.com",
                "Oracle OCI GenAI",
            ),
        ),
        (
            "sap_ai_hub",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "SAP_AI_HUB_API_KEY",
                "https://api.aihub.sap.com/v1",
                "SAP AI Hub",
            ),
        ),
        (
            "scaleway",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "SCALEWAY_API_KEY",
                "https://api.scaleway.ai/v1",
                "Scaleway",
            ),
        ),
        (
            "datarobot",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "DATAROBOT_API_KEY",
                "https://app.datarobot.com/api/v2/genai",
                "DataRobot",
            ),
        ),
        (
            "nlp_cloud",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "NLP_CLOUD_API_KEY",
                "https://api.nlpcloud.io/v1",
                "NLP Cloud",
            ),
        ),
        (
            "aleph_alpha",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "ALEPH_ALPHA_API_KEY",
                "https://api.aleph-alpha.com/v1",
                "Aleph Alpha",
            ),
        ),
        (
            "ai21",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "AI21_API_KEY",
                "https://api.ai21.com/studio/v1",
                "AI21",
            ),
        ),
        (
            "clarifai",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "CLARIFAI_PAT",
                "https://api.clarifai.com/v2",
                "Clarifai",
            ),
        ),
        (
            "gitlab_duo",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "GITLAB_TOKEN",
                "https://gitlab.com/api/v4/ai",
                "GitLab Duo",
            ),
        ),
        (
            "amazon_q",
            api_key_provider(
                ProviderCategory::Enterprise,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "AMAZON_Q_TOKEN",
                "https://codewhisperer.us-east-1.amazonaws.com",
                "Amazon Q",
            ),
        ),
        (
            "qwen",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "DASHSCOPE_API_KEY",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "Qwen",
            ),
        ),
        (
            "qwen_alibaba",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "DASHSCOPE_API_KEY",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "Qwen",
            ),
        ),
        (
            "zhipu_chatglm",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "ZHIPU_API_KEY",
                "https://open.bigmodel.cn/api/paas/v4",
                "Zhipu GLM",
            ),
        ),
        (
            "moonshot_kimi",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "MOONSHOT_API_KEY",
                "https://api.moonshot.cn/v1",
                "Moonshot Kimi",
            ),
        ),
        (
            "minimax",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "MINIMAX_API_KEY",
                "https://api.minimax.chat/v1",
                "MiniMax",
            ),
        ),
        (
            "yi_01ai",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "YI_API_KEY",
                "https://api.01.ai/v1",
                "01.AI Yi",
            ),
        ),
        (
            "baichuan",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "BAICHUAN_API_KEY",
                "https://api.baichuan-ai.com/v1",
                "Baichuan",
            ),
        ),
        (
            "stepfun",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "STEPFUN_API_KEY",
                "https://api.stepfun.com/v1",
                "StepFun",
            ),
        ),
        (
            "doubao",
            api_key_provider(
                ProviderCategory::RegionalChinese,
                ProviderIcon::OpenAiCompatible,
                false,
                ApiKeyHeaderStyle::Bearer,
                "VOLCANO_API_KEY",
                "https://ark.cn-beijing.volces.com/api/v3",
                "Doubao",
            ),
        ),
        (
            "ollama",
            no_auth_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                true,
                "http://localhost:11434/v1",
                "Ollama",
            ),
        ),
        (
            "lmstudio",
            no_auth_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                true,
                "http://localhost:1234/v1",
                "LM Studio",
            ),
        ),
        (
            "llamafile",
            no_auth_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                false,
                "http://localhost:8080/v1",
                "Llamafile",
            ),
        ),
        (
            "text-generation-webui",
            no_auth_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                false,
                "http://localhost:5000/v1",
                "Text Generation WebUI",
            ),
        ),
        (
            "text_generation_webui",
            no_auth_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                false,
                "http://localhost:5000/v1",
                "Text Generation WebUI",
            ),
        ),
        (
            "vllm",
            no_auth_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                false,
                "http://localhost:8000/v1",
                "vLLM",
            ),
        ),
        (
            "lemonade",
            no_auth_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                false,
                "http://localhost:11434/v1",
                "Lemonade",
            ),
        ),
        (
            "litellm_proxy",
            api_key_provider(
                ProviderCategory::Local,
                ProviderIcon::OpenAiCompatible,
                true,
                ApiKeyHeaderStyle::Bearer,
                "LITELLM_PROXY_API_KEY",
                "http://localhost:4000",
                "LiteLLM Proxy",
            ),
        ),
    ])
}

fn infer_category(provider_id: &str, api_base: &str) -> ProviderCategory {
    match provider_id {
        "openai" | "anthropic" | "google" | "github-models" | "github_models" => {
            ProviderCategory::Frontier
        }
        "openrouter"
        | "groq"
        | "together"
        | "fireworks-ai"
        | "deepinfra"
        | "perplexity"
        | "nvidia_nim"
        | "cerebras"
        | "replicate"
        | "anyscale"
        | "lepton"
        | "302ai"
        | "cloudflare_workers_ai"
        | "cloudflare_ai_gateway"
        | "friendliai"
        | "friendli"
        | "aiml"
        | "baseten"
        | "bytez" => ProviderCategory::FastInference,
        "huggingface" | "cohere" | "fal" | "fal_ai" | "elevenlabs" | "deepgram" | "runway"
        | "pika" | "stability" | "black-forest-labs" => ProviderCategory::Specialist,
        "azure_ai_foundry" | "oci_genai" | "sap_ai_hub" | "scaleway" | "datarobot"
        | "nlp_cloud" | "aleph_alpha" | "ai21" | "clarifai" | "gitlab_duo" | "amazon_q" => {
            ProviderCategory::Enterprise
        }
        "qwen" | "qwen_alibaba" | "zhipu_chatglm" | "moonshot_kimi" | "minimax" | "yi_01ai"
        | "baichuan" | "stepfun" | "doubao" => ProviderCategory::RegionalChinese,
        "ollama"
        | "lmstudio"
        | "vllm"
        | "llamafile"
        | "text-generation-webui"
        | "litellm_proxy" => ProviderCategory::Local,
        _ if api_base.contains("localhost") || api_base.contains("127.0.0.1") => {
            ProviderCategory::Local
        }
        _ => ProviderCategory::FastInference,
    }
}

fn infer_icon(provider_id: &str) -> ProviderIcon {
    match provider_id {
        "openai" => ProviderIcon::OpenAi,
        "anthropic" => ProviderIcon::Anthropic,
        "google" => ProviderIcon::Google,
        "openrouter" => ProviderIcon::OpenRouter,
        "github-models" | "github_models" | "copilot_chat" => ProviderIcon::Copilot,
        "bedrock" | "amazon-bedrock" | "amazon_q" => ProviderIcon::Bedrock,
        "mistral" => ProviderIcon::Mistral,
        "x_ai" | "xai" => ProviderIcon::XAi,
        _ => ProviderIcon::OpenAiCompatible,
    }
}

fn is_featured_provider(provider_id: &str) -> bool {
    matches!(
        provider_id,
        "openai"
            | "anthropic"
            | "google"
            | "openrouter"
            | "groq"
            | "together"
            | "fireworks-ai"
            | "perplexity"
            | "ollama"
            | "lmstudio"
            | "github-models"
    )
}

fn string_array(value: &Value) -> Option<Vec<String>> {
    Some(
        value
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
    )
}

fn parse_f64_value(value: &Value) -> Option<f64> {
    match value {
        Value::String(value) => value.parse().ok(),
        Value::Number(value) => value.as_f64(),
        _ => None,
    }
}

fn title_case(input: &str) -> String {
    input
        .split(['-', '_', '/'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut title = first.to_uppercase().collect::<String>();
                    title.push_str(chars.as_str());
                    title
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn canonical_provider_id(input: &str) -> String {
    match input {
        "x-ai" => "x_ai".to_owned(),
        "together-ai" => "together".to_owned(),
        "fireworks" => "fireworks-ai".to_owned(),
        "github_models" => "github-models".to_owned(),
        "github-model" => "github-models".to_owned(),
        "hugging-face" => "huggingface".to_owned(),
        "friendli-ai" => "friendliai".to_owned(),
        "cloudflare-workers-ai" => "cloudflare_workers_ai".to_owned(),
        "cloudflare-ai-gateway" => "cloudflare_ai_gateway".to_owned(),
        "text_generation_webui" => "text-generation-webui".to_owned(),
        "amazon_q" => "amazon_q".to_owned(),
        other => other.to_owned(),
    }
}
