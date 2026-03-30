use std::sync::Arc;

use anyhow::Result;
use collections::HashSet;
use fs::Fs;
use gpui::{
    DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Render, ScrollHandle, Task,
};
use language_model::LanguageModelRegistry;
use language_models::provider::open_ai_compatible::{AvailableModel, ModelCapabilities};
use language_models::provider_hub::{ModelManifest as HubModelManifest, ProviderHubStore};
use settings::{OpenAiCompatibleSettingsContent, update_settings_file};
use ui::{
    Banner, Checkbox, KeyBinding, Modal, ModalFooter, ModalHeader, Section, ToggleState,
    WithScrollbar, prelude::*,
};
use ui_input::InputField;
use workspace::{ModalView, Workspace};

fn single_line_input(
    label: impl Into<SharedString>,
    placeholder: &str,
    text: Option<&str>,
    tab_index: isize,
    window: &mut Window,
    cx: &mut App,
) -> Entity<InputField> {
    cx.new(|cx| {
        let input = InputField::new(window, cx, placeholder)
            .label(label)
            .tab_index(tab_index)
            .tab_stop(true);

        if let Some(text) = text {
            input.set_text(text, window, cx);
        }
        input
    })
}

#[derive(Clone, Copy)]
pub enum LlmCompatibleProvider {
    OpenAi,
    OpenRouter,
    GitHubModels,
    Groq,
    TogetherAi,
    FireworksAi,
    Perplexity,
    DeepInfra,
    HuggingFace,
    Baseten,
    Replicate,
    LiteLlmProxy,
    Vllm,
    Llamafile,
    TextGenerationWebUi,
}

#[derive(Clone, Copy)]
enum ModelDiscoveryKind {
    OpenAiCompatible,
}

impl LlmCompatibleProvider {
    pub fn featured() -> &'static [Self] {
        &[
            Self::OpenAi,
            Self::OpenRouter,
            Self::GitHubModels,
            Self::Groq,
            Self::TogetherAi,
            Self::FireworksAi,
            Self::Perplexity,
            Self::DeepInfra,
            Self::HuggingFace,
            Self::Baseten,
            Self::Replicate,
            Self::LiteLlmProxy,
            Self::Vllm,
            Self::Llamafile,
            Self::TextGenerationWebUi,
        ]
    }

    fn catalog_id(&self) -> &'static str {
        match self {
            LlmCompatibleProvider::OpenAi => "openai",
            LlmCompatibleProvider::OpenRouter => "openrouter",
            LlmCompatibleProvider::GitHubModels => "github-models",
            LlmCompatibleProvider::Groq => "groq",
            LlmCompatibleProvider::TogetherAi => "together",
            LlmCompatibleProvider::FireworksAi => "fireworks-ai",
            LlmCompatibleProvider::Perplexity => "perplexity",
            LlmCompatibleProvider::DeepInfra => "deepinfra",
            LlmCompatibleProvider::HuggingFace => "huggingface",
            LlmCompatibleProvider::Baseten => "baseten",
            LlmCompatibleProvider::Replicate => "replicate",
            LlmCompatibleProvider::LiteLlmProxy => "litellm_proxy",
            LlmCompatibleProvider::Vllm => "vllm",
            LlmCompatibleProvider::Llamafile => "llamafile",
            LlmCompatibleProvider::TextGenerationWebUi => "text-generation-webui",
        }
    }

    fn suggested_provider_id(&self) -> Option<&'static str> {
        match self {
            LlmCompatibleProvider::OpenAi => None,
            _ => Some(self.catalog_id()),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            LlmCompatibleProvider::OpenAi => "OpenAI-Compatible",
            LlmCompatibleProvider::OpenRouter => "OpenRouter",
            LlmCompatibleProvider::GitHubModels => "GitHub Models",
            LlmCompatibleProvider::Groq => "Groq",
            LlmCompatibleProvider::TogetherAi => "Together AI",
            LlmCompatibleProvider::FireworksAi => "Fireworks AI",
            LlmCompatibleProvider::Perplexity => "Perplexity",
            LlmCompatibleProvider::DeepInfra => "DeepInfra",
            LlmCompatibleProvider::HuggingFace => "Hugging Face",
            LlmCompatibleProvider::Baseten => "Baseten",
            LlmCompatibleProvider::Replicate => "Replicate",
            LlmCompatibleProvider::LiteLlmProxy => "LiteLLM Proxy",
            LlmCompatibleProvider::Vllm => "vLLM",
            LlmCompatibleProvider::Llamafile => "Llamafile",
            LlmCompatibleProvider::TextGenerationWebUi => "Text Generation WebUI",
        }
    }

    fn api_url(&self) -> &'static str {
        match self {
            LlmCompatibleProvider::OpenAi => "https://api.openai.com/v1",
            LlmCompatibleProvider::OpenRouter => "https://openrouter.ai/api/v1",
            LlmCompatibleProvider::GitHubModels => "https://models.inference.ai.azure.com",
            LlmCompatibleProvider::Groq => "https://api.groq.com/openai/v1",
            LlmCompatibleProvider::TogetherAi => "https://api.together.xyz/v1",
            LlmCompatibleProvider::FireworksAi => "https://api.fireworks.ai/inference/v1",
            LlmCompatibleProvider::Perplexity => "https://api.perplexity.ai",
            LlmCompatibleProvider::DeepInfra => "https://api.deepinfra.com/v1/openai",
            LlmCompatibleProvider::HuggingFace => "https://api-inference.huggingface.co/v1",
            LlmCompatibleProvider::Baseten => "https://bridge.baseten.co/v1",
            LlmCompatibleProvider::Replicate => "https://api.replicate.com/v1",
            LlmCompatibleProvider::LiteLlmProxy => "http://localhost:4000",
            LlmCompatibleProvider::Vllm => "http://localhost:8000/v1",
            LlmCompatibleProvider::Llamafile => "http://localhost:8080/v1",
            LlmCompatibleProvider::TextGenerationWebUi => "http://localhost:5000/v1",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            LlmCompatibleProvider::OpenAi => {
                "Connect any OpenAI-compatible endpoint. Great for custom gateways and self-hosted stacks."
            }
            LlmCompatibleProvider::LiteLlmProxy => {
                "Use a LiteLLM gateway and sync its routed model list directly into the picker."
            }
            LlmCompatibleProvider::Vllm
            | LlmCompatibleProvider::Llamafile
            | LlmCompatibleProvider::TextGenerationWebUi => {
                "Connect a local OpenAI-compatible inference server and discover its exposed models."
            }
            _ => "Prefill a production-ready provider endpoint and discover its available models.",
        }
    }

    fn requires_api_key(&self) -> bool {
        !matches!(
            self,
            LlmCompatibleProvider::LiteLlmProxy
                | LlmCompatibleProvider::Vllm
                | LlmCompatibleProvider::Llamafile
                | LlmCompatibleProvider::TextGenerationWebUi
        )
    }

    fn discovery_kind(&self) -> ModelDiscoveryKind {
        ModelDiscoveryKind::OpenAiCompatible
    }
}

fn available_model_from_hub_model(model: &HubModelManifest) -> AvailableModel {
    AvailableModel {
        name: model.id.clone(),
        display_name: model.display_name.clone(),
        max_tokens: model.max_tokens.max(1),
        max_output_tokens: model.max_output_tokens,
        max_completion_tokens: model.max_completion_tokens.or(model.max_output_tokens),
        capabilities: ModelCapabilities {
            tools: model.capabilities.tools,
            images: model.capabilities.images,
            parallel_tool_calls: model.capabilities.parallel_tool_calls,
            prompt_cache_key: model.capabilities.prompt_cache_key,
            chat_completions: model.capabilities.chat_completions,
        },
    }
}

fn seeded_models_for_provider(provider: LlmCompatibleProvider, cx: &App) -> Vec<AvailableModel> {
    ProviderHubStore::try_global(cx)
        .and_then(|store| {
            store
                .read(cx)
                .provider_manifest(provider.catalog_id())
                .map(|manifest| {
                    manifest
                        .models
                        .iter()
                        .take(8)
                        .map(available_model_from_hub_model)
                        .collect::<Vec<_>>()
                })
        })
        .filter(|models| !models.is_empty())
        .unwrap_or_default()
}

struct AddLlmProviderInput {
    provider_name: Entity<InputField>,
    api_url: Entity<InputField>,
    api_key: Entity<InputField>,
    models: Vec<ModelInput>,
}

impl AddLlmProviderInput {
    fn new(provider: LlmCompatibleProvider, window: &mut Window, cx: &mut App) -> Self {
        let provider_name = single_line_input(
            "Provider Name",
            provider.name(),
            provider.suggested_provider_id(),
            1,
            window,
            cx,
        );
        let api_url = single_line_input("API URL", provider.api_url(), Some(provider.api_url()), 2, window, cx);
        let api_key = cx.new(|cx| {
            InputField::new(
                window,
                cx,
                "000000000000000000000000000000000000000000000000",
            )
            .label(if provider.requires_api_key() {
                "API Key"
            } else {
                "API Key (optional)"
            })
            .tab_index(3)
            .tab_stop(true)
            .masked(true)
        });
        let seeded_models = seeded_models_for_provider(provider, cx);

        Self {
            provider_name,
            api_url,
            api_key,
            models: if seeded_models.is_empty() {
                vec![ModelInput::new(0, window, cx)]
            } else {
                seeded_models
                    .iter()
                    .enumerate()
                    .map(|(ix, model)| ModelInput::from_available(ix, model, window, cx))
                    .collect()
            },
        }
    }

    fn add_model(&mut self, window: &mut Window, cx: &mut App) {
        let model_index = self.models.len();
        self.models.push(ModelInput::new(model_index, window, cx));
    }

    fn remove_model(&mut self, index: usize) {
        self.models.remove(index);
    }

    fn replace_models(
        &mut self,
        models: Vec<AvailableModel>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.models = if models.is_empty() {
            vec![ModelInput::new(0, window, cx)]
        } else {
            models
                .iter()
                .enumerate()
                .map(|(ix, model)| ModelInput::from_available(ix, model, window, cx))
                .collect()
        };
    }
}

struct ModelCapabilityToggles {
    pub supports_tools: ToggleState,
    pub supports_images: ToggleState,
    pub supports_parallel_tool_calls: ToggleState,
    pub supports_prompt_cache_key: ToggleState,
    pub supports_chat_completions: ToggleState,
}

struct ModelInput {
    name: Entity<InputField>,
    max_completion_tokens: Entity<InputField>,
    max_output_tokens: Entity<InputField>,
    max_tokens: Entity<InputField>,
    capabilities: ModelCapabilityToggles,
}

impl ModelInput {
    fn new(model_index: usize, window: &mut Window, cx: &mut App) -> Self {
        let base_tab_index = (3 + (model_index * 4)) as isize;

        let model_name = single_line_input(
            "Model Name",
            "e.g. gpt-5, claude-opus-4, gemini-2.5-pro",
            None,
            base_tab_index + 1,
            window,
            cx,
        );
        let max_completion_tokens = single_line_input(
            "Max Completion Tokens",
            "200000",
            Some("200000"),
            base_tab_index + 2,
            window,
            cx,
        );
        let max_output_tokens = single_line_input(
            "Max Output Tokens",
            "Max Output Tokens",
            Some("32000"),
            base_tab_index + 3,
            window,
            cx,
        );
        let max_tokens = single_line_input(
            "Max Tokens",
            "Max Tokens",
            Some("200000"),
            base_tab_index + 4,
            window,
            cx,
        );

        let ModelCapabilities {
            tools,
            images,
            parallel_tool_calls,
            prompt_cache_key,
            chat_completions,
        } = ModelCapabilities::default();

        Self {
            name: model_name,
            max_completion_tokens,
            max_output_tokens,
            max_tokens,
            capabilities: ModelCapabilityToggles {
                supports_tools: tools.into(),
                supports_images: images.into(),
                supports_parallel_tool_calls: parallel_tool_calls.into(),
                supports_prompt_cache_key: prompt_cache_key.into(),
                supports_chat_completions: chat_completions.into(),
            },
        }
    }

    fn from_available(
        model_index: usize,
        model: &AvailableModel,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let mut input = Self::new(model_index, window, cx);
        input.name.update(cx, |field, cx| {
            field.set_text(&model.name, window, cx);
        });
        input.max_tokens.update(cx, |field, cx| {
            field.set_text(model.max_tokens.to_string(), window, cx);
        });
        input.max_completion_tokens.update(cx, |field, cx| {
            field.set_text(
                model
                    .max_completion_tokens
                    .or(model.max_output_tokens)
                    .unwrap_or(model.max_tokens)
                    .to_string(),
                window,
                cx,
            );
        });
        input.max_output_tokens.update(cx, |field, cx| {
            field.set_text(
                model
                    .max_output_tokens
                    .or(model.max_completion_tokens)
                    .unwrap_or(model.max_tokens)
                    .to_string(),
                window,
                cx,
            );
        });
        input.capabilities.supports_tools = model.capabilities.tools.into();
        input.capabilities.supports_images = model.capabilities.images.into();
        input.capabilities.supports_parallel_tool_calls =
            model.capabilities.parallel_tool_calls.into();
        input.capabilities.supports_prompt_cache_key =
            model.capabilities.prompt_cache_key.into();
        input.capabilities.supports_chat_completions =
            model.capabilities.chat_completions.into();
        input
    }

    fn parse(&self, cx: &App) -> Result<AvailableModel, SharedString> {
        let name = self.name.read(cx).text(cx);
        if name.is_empty() {
            return Err(SharedString::from("Model Name cannot be empty"));
        }
        Ok(AvailableModel {
            name,
            display_name: None,
            max_completion_tokens: Some(
                self.max_completion_tokens
                    .read(cx)
                    .text(cx)
                    .parse::<u64>()
                    .map_err(|_| SharedString::from("Max Completion Tokens must be a number"))?,
            ),
            max_output_tokens: Some(
                self.max_output_tokens
                    .read(cx)
                    .text(cx)
                    .parse::<u64>()
                    .map_err(|_| SharedString::from("Max Output Tokens must be a number"))?,
            ),
            max_tokens: self
                .max_tokens
                .read(cx)
                .text(cx)
                .parse::<u64>()
                .map_err(|_| SharedString::from("Max Tokens must be a number"))?,
            capabilities: ModelCapabilities {
                tools: self.capabilities.supports_tools.selected(),
                images: self.capabilities.supports_images.selected(),
                parallel_tool_calls: self.capabilities.supports_parallel_tool_calls.selected(),
                prompt_cache_key: self.capabilities.supports_prompt_cache_key.selected(),
                chat_completions: self.capabilities.supports_chat_completions.selected(),
            },
        })
    }
}

fn parse_u64_value(value: Option<&serde_json::Value>) -> Option<u64> {
    match value {
        Some(serde_json::Value::Number(number)) => number.as_u64(),
        Some(serde_json::Value::String(value)) => value.parse().ok(),
        _ => None,
    }
}

fn parse_openai_compatible_models(payload: serde_json::Value) -> Vec<AvailableModel> {
    let models = payload
        .get("data")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .or_else(|| {
            payload
                .get("models")
                .and_then(serde_json::Value::as_array)
                .cloned()
        })
        .unwrap_or_default();

    models
        .into_iter()
        .filter_map(|model| {
            let model = model.as_object()?;
            let name = model
                .get("id")
                .or_else(|| model.get("name"))
                .and_then(serde_json::Value::as_str)?
                .to_owned();
            let context_window = parse_u64_value(
                model
                    .get("context_window")
                    .or_else(|| model.get("context_length"))
                    .or_else(|| model.get("max_context_length")),
            )
            .unwrap_or(128_000);
            let max_output_tokens = parse_u64_value(
                model.get("max_output_tokens")
                    .or_else(|| model.get("max_completion_tokens")),
            );
            let input_modalities = model
                .get("architecture")
                .and_then(serde_json::Value::as_object)
                .and_then(|architecture| architecture.get("input_modalities"))
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let supported_parameters = model
                .get("supported_parameters")
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let supports_tools = supported_parameters
                .iter()
                .any(|parameter| matches!(*parameter, "tools" | "tool_choice" | "response_format"));
            let supports_images = input_modalities
                .iter()
                .any(|modality| matches!(*modality, "image" | "video"));

            Some(AvailableModel {
                name,
                display_name: model
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                max_tokens: context_window,
                max_output_tokens,
                max_completion_tokens: max_output_tokens,
                capabilities: ModelCapabilities {
                    tools: supports_tools,
                    images: supports_images,
                    parallel_tool_calls: supports_tools,
                    prompt_cache_key: false,
                    chat_completions: true,
                },
            })
        })
        .collect()
}

async fn discover_models(
    provider: LlmCompatibleProvider,
    api_url: String,
    api_key: Option<String>,
) -> Result<Vec<AvailableModel>, SharedString> {
    let client = reqwest::Client::new();
    let mut request = match provider.discovery_kind() {
        ModelDiscoveryKind::OpenAiCompatible => {
            client.get(format!("{}/models", api_url.trim_end_matches('/')))
        }
    };

    if let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(api_key);
    }

    let payload = request
        .send()
        .await
        .map_err(|error| SharedString::from(format!("Failed to contact provider: {error}")))?
        .error_for_status()
        .map_err(|error| SharedString::from(format!("Provider returned an error: {error}")))?
        .json::<serde_json::Value>()
        .await
        .map_err(|error| SharedString::from(format!("Failed to parse provider models: {error}")))?;

    let models = parse_openai_compatible_models(payload);
    if models.is_empty() {
        Err("No models were returned by this endpoint".into())
    } else {
        Ok(models)
    }
}

fn save_provider_to_settings(
    provider: LlmCompatibleProvider,
    input: &AddLlmProviderInput,
    cx: &mut App,
) -> Task<Result<(), SharedString>> {
    let provider_name: Arc<str> = input.provider_name.read(cx).text(cx).into();
    if provider_name.is_empty() {
        return Task::ready(Err("Provider Name cannot be empty".into()));
    }

    if LanguageModelRegistry::read_global(cx)
        .providers()
        .iter()
        .any(|provider| {
            provider.id().0.as_ref() == provider_name.as_ref()
                || provider.name().0.as_ref() == provider_name.as_ref()
        })
    {
        return Task::ready(Err(
            "Provider Name is already taken by another provider".into()
        ));
    }

    let api_url = input.api_url.read(cx).text(cx);
    if api_url.is_empty() {
        return Task::ready(Err("API URL cannot be empty".into()));
    }

    let api_key = input.api_key.read(cx).text(cx);
    if provider.requires_api_key() && api_key.is_empty() {
        return Task::ready(Err("API Key cannot be empty".into()));
    }

    let mut models = Vec::new();
    let mut model_names: HashSet<String> = HashSet::default();
    for model in &input.models {
        match model.parse(cx) {
            Ok(model) => {
                if !model_names.insert(model.name.clone()) {
                    return Task::ready(Err("Model Names must be unique".into()));
                }
                models.push(model)
            }
            Err(err) => return Task::ready(Err(err)),
        }
    }

    let fs = <dyn Fs>::global(cx);
    let write_credentials_task = if api_key.is_empty() {
        None
    } else {
        Some(cx.write_credentials(&api_url, "Bearer", api_key.as_bytes()))
    };
    cx.spawn(async move |cx| {
        if let Some(task) = write_credentials_task {
            task.await
                .map_err(|_| SharedString::from("Failed to write API key to keychain"))?;
        }
        cx.update(|cx| {
            update_settings_file(fs, cx, |settings, _cx| {
                settings
                    .language_models
                    .get_or_insert_default()
                    .openai_compatible
                    .get_or_insert_default()
                    .insert(
                        provider_name,
                        OpenAiCompatibleSettingsContent {
                            api_url,
                            available_models: models,
                        },
                    );
            });
        });
        Ok(())
    })
}

pub struct AddLlmProviderModal {
    provider: LlmCompatibleProvider,
    input: AddLlmProviderInput,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    last_error: Option<SharedString>,
    discovery_message: Option<SharedString>,
    discover_models_task: Option<Task<()>>,
}

impl AddLlmProviderModal {
    pub fn toggle(
        provider: LlmCompatibleProvider,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        workspace.toggle_modal(window, cx, |window, cx| Self::new(provider, window, cx));
    }

    fn new(provider: LlmCompatibleProvider, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            input: AddLlmProviderInput::new(provider, window, cx),
            provider,
            last_error: None,
            discovery_message: Some("Seeded models from the synced provider catalog when available.".into()),
            discover_models_task: None,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
        }
    }

    fn confirm(&mut self, _: &menu::Confirm, _: &mut Window, cx: &mut Context<Self>) {
        let task = save_provider_to_settings(self.provider, &self.input, cx);
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| match result {
                Ok(_) => {
                    cx.emit(DismissEvent);
                }
                Err(error) => {
                    this.last_error = Some(error);
                    cx.notify();
                }
            })
        })
        .detach_and_log_err(cx);
    }

    fn discover_models(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.discover_models_task.is_some() {
            return;
        }

        self.last_error = None;
        self.discovery_message = Some("Discovering models from the provider endpoint...".into());

        let provider = self.provider;
        let api_url = self.input.api_url.read(cx).text(cx);
        let api_key = self.input.api_key.read(cx).text(cx);

        self.discover_models_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = discover_models(
                provider,
                api_url,
                if api_key.is_empty() { None } else { Some(api_key) },
            )
            .await;

            this.update_in(cx, |this, window, cx| {
                this.discover_models_task = None;
                match result {
                    Ok(models) => {
                        let count = models.len();
                        this.input.replace_models(models, window, cx);
                        this.discovery_message =
                            Some(format!("Discovered {count} models from the live endpoint.").into());
                    }
                    Err(error) => {
                        this.last_error = Some(error);
                        this.discovery_message =
                            Some("Model discovery failed. You can still edit models manually.".into());
                    }
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    fn cancel(&mut self, _: &menu::Cancel, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn render_model_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .mt_1()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .child(Label::new("Models").size(LabelSize::Small))
                    .child(
                        h_flex()
                            .gap_1()
                            .when(true, |this| {
                                this.child(
                                    Button::new("discover-models", "Discover Models")
                                        .style(ButtonStyle::Outlined)
                                        .disabled(self.discover_models_task.is_some())
                                        .start_icon(
                                            Icon::new(IconName::ArrowCircle)
                                                .size(IconSize::XSmall)
                                                .color(Color::Muted),
                                        )
                                        .label_size(LabelSize::Small)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.discover_models(window, cx);
                                        })),
                                )
                            })
                            .child(
                                Button::new("add-model", "Add Model")
                                    .start_icon(
                                        Icon::new(IconName::Plus)
                                            .size(IconSize::XSmall)
                                            .color(Color::Muted),
                                    )
                                    .label_size(LabelSize::Small)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.input.add_model(window, cx);
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .when_some(self.discovery_message.clone(), |this, message| {
                this.child(Label::new(message).size(LabelSize::Small).color(Color::Muted))
            })
            .children(
                self.input
                    .models
                    .iter()
                    .enumerate()
                    .map(|(ix, _)| self.render_model(ix, cx)),
            )
    }

    fn render_model(&self, ix: usize, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let has_more_than_one_model = self.input.models.len() > 1;
        let model = &self.input.models[ix];

        v_flex()
            .p_2()
            .gap_2()
            .rounded_sm()
            .border_1()
            .border_dashed()
            .border_color(cx.theme().colors().border.opacity(0.6))
            .bg(cx.theme().colors().element_active.opacity(0.15))
            .child(model.name.clone())
            .child(
                h_flex()
                    .gap_2()
                    .child(model.max_completion_tokens.clone())
                    .child(model.max_output_tokens.clone()),
            )
            .child(model.max_tokens.clone())
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        Checkbox::new(("supports-tools", ix), model.capabilities.supports_tools)
                            .label("Supports tools")
                            .on_click(cx.listener(move |this, checked, _window, cx| {
                                this.input.models[ix].capabilities.supports_tools = *checked;
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new(("supports-images", ix), model.capabilities.supports_images)
                            .label("Supports images")
                            .on_click(cx.listener(move |this, checked, _window, cx| {
                                this.input.models[ix].capabilities.supports_images = *checked;
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new(
                            ("supports-parallel-tool-calls", ix),
                            model.capabilities.supports_parallel_tool_calls,
                        )
                        .label("Supports parallel_tool_calls")
                        .on_click(cx.listener(
                            move |this, checked, _window, cx| {
                                this.input.models[ix]
                                    .capabilities
                                    .supports_parallel_tool_calls = *checked;
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        Checkbox::new(
                            ("supports-prompt-cache-key", ix),
                            model.capabilities.supports_prompt_cache_key,
                        )
                        .label("Supports prompt_cache_key")
                        .on_click(cx.listener(
                            move |this, checked, _window, cx| {
                                this.input.models[ix].capabilities.supports_prompt_cache_key =
                                    *checked;
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        Checkbox::new(
                            ("supports-chat-completions", ix),
                            model.capabilities.supports_chat_completions,
                        )
                        .label("Supports /chat/completions")
                        .on_click(cx.listener(
                            move |this, checked, _window, cx| {
                                this.input.models[ix].capabilities.supports_chat_completions =
                                    *checked;
                                cx.notify();
                            },
                        )),
                    ),
            )
            .when(has_more_than_one_model, |this| {
                this.child(
                    Button::new(("remove-model", ix), "Remove Model")
                        .start_icon(
                            Icon::new(IconName::Trash)
                                .size(IconSize::XSmall)
                                .color(Color::Muted),
                        )
                        .label_size(LabelSize::Small)
                        .style(ButtonStyle::Outlined)
                        .full_width()
                        .on_click(cx.listener(move |this, _, _window, cx| {
                            this.input.remove_model(ix);
                            cx.notify();
                        })),
                )
            })
    }

    fn on_tab(&mut self, _: &menu::SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(
        &mut self,
        _: &menu::SelectPrevious,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus_prev(cx);
    }
}

impl EventEmitter<DismissEvent> for AddLlmProviderModal {}

impl Focusable for AddLlmProviderModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for AddLlmProviderModal {}

impl Render for AddLlmProviderModal {
    fn render(&mut self, window: &mut ui::Window, cx: &mut ui::Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        let window_size = window.viewport_size();
        let rem_size = window.rem_size();
        let is_large_window = window_size.height / rem_size > rems_from_px(600.).0;

        let modal_max_height = if is_large_window {
            rems_from_px(450.)
        } else {
            rems_from_px(200.)
        };

        v_flex()
            .id("add-llm-provider-modal")
            .key_context("AddLlmProviderModal")
            .w(rems(34.))
            .elevation_3(cx)
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .capture_any_mouse_down(cx.listener(|this, _, window, cx| {
                this.focus_handle(cx).focus(window, cx);
            }))
            .child(
                Modal::new("configure-context-server", None)
                    .header(
                        ModalHeader::new()
                            .headline("Add LLM Provider")
                            .description(self.provider.description()),
                    )
                    .when_some(self.last_error.clone(), |this, error| {
                        this.section(
                            Section::new().child(
                                Banner::new()
                                    .severity(Severity::Warning)
                                    .child(div().text_xs().child(error)),
                            ),
                        )
                    })
                    .child(
                        div()
                            .size_full()
                            .vertical_scrollbar_for(&self.scroll_handle, window, cx)
                            .child(
                                v_flex()
                                    .id("modal_content")
                                    .size_full()
                                    .tab_group()
                                    .max_h(modal_max_height)
                                    .pl_3()
                                    .pr_4()
                                    .pb_2()
                                    .gap_2()
                                    .overflow_y_scroll()
                                    .track_scroll(&self.scroll_handle)
                                    .child(self.input.provider_name.clone())
                                    .child(self.input.api_url.clone())
                                    .child(self.input.api_key.clone())
                                    .child(self.render_model_section(cx)),
                            ),
                    )
                    .footer(
                        ModalFooter::new().end_slot(
                            h_flex()
                                .gap_1()
                                .child(
                                    Button::new("cancel", "Cancel")
                                        .key_binding(
                                            KeyBinding::for_action_in(
                                                &menu::Cancel,
                                                &focus_handle,
                                                cx,
                                            )
                                            .map(|kb| kb.size(rems_from_px(12.))),
                                        )
                                        .on_click(cx.listener(|this, _event, window, cx| {
                                            this.cancel(&menu::Cancel, window, cx)
                                        })),
                                )
                                .child(
                                    Button::new("save-server", "Save Provider")
                                        .key_binding(
                                            KeyBinding::for_action_in(
                                                &menu::Confirm,
                                                &focus_handle,
                                                cx,
                                            )
                                            .map(|kb| kb.size(rems_from_px(12.))),
                                        )
                                        .on_click(cx.listener(|this, _event, window, cx| {
                                            this.confirm(&menu::Confirm, window, cx)
                                        })),
                                ),
                        ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::FakeFs;
    use gpui::{TestAppContext, VisualTestContext};
    use language_model::{
        LanguageModelProviderId, LanguageModelProviderName,
        fake_provider::FakeLanguageModelProvider,
    };
    use project::Project;
    use settings::SettingsStore;
    use util::path;
    use workspace::MultiWorkspace;

    #[gpui::test]
    async fn test_save_provider_invalid_inputs(cx: &mut TestAppContext) {
        let cx = setup_test(cx).await;

        assert_eq!(
            save_provider_validation_errors("", "someurl", "somekey", vec![], cx,).await,
            Some("Provider Name cannot be empty".into())
        );

        assert_eq!(
            save_provider_validation_errors("someprovider", "", "somekey", vec![], cx,).await,
            Some("API URL cannot be empty".into())
        );

        assert_eq!(
            save_provider_validation_errors("someprovider", "someurl", "", vec![], cx,).await,
            Some("API Key cannot be empty".into())
        );

        assert_eq!(
            save_provider_validation_errors(
                "someprovider",
                "someurl",
                "somekey",
                vec![("", "200000", "200000", "32000")],
                cx,
            )
            .await,
            Some("Model Name cannot be empty".into())
        );

        assert_eq!(
            save_provider_validation_errors(
                "someprovider",
                "someurl",
                "somekey",
                vec![("somemodel", "abc", "200000", "32000")],
                cx,
            )
            .await,
            Some("Max Tokens must be a number".into())
        );

        assert_eq!(
            save_provider_validation_errors(
                "someprovider",
                "someurl",
                "somekey",
                vec![("somemodel", "200000", "abc", "32000")],
                cx,
            )
            .await,
            Some("Max Completion Tokens must be a number".into())
        );

        assert_eq!(
            save_provider_validation_errors(
                "someprovider",
                "someurl",
                "somekey",
                vec![("somemodel", "200000", "200000", "abc")],
                cx,
            )
            .await,
            Some("Max Output Tokens must be a number".into())
        );

        assert_eq!(
            save_provider_validation_errors(
                "someprovider",
                "someurl",
                "somekey",
                vec![
                    ("somemodel", "200000", "200000", "32000"),
                    ("somemodel", "200000", "200000", "32000"),
                ],
                cx,
            )
            .await,
            Some("Model Names must be unique".into())
        );
    }

    #[gpui::test]
    async fn test_save_provider_name_conflict(cx: &mut TestAppContext) {
        let cx = setup_test(cx).await;

        cx.update(|_window, cx| {
            LanguageModelRegistry::global(cx).update(cx, |registry, cx| {
                registry.register_provider(
                    Arc::new(FakeLanguageModelProvider::new(
                        LanguageModelProviderId::new("someprovider"),
                        LanguageModelProviderName::new("Some Provider"),
                    )),
                    cx,
                );
            });
        });

        assert_eq!(
            save_provider_validation_errors(
                "someprovider",
                "someurl",
                "someapikey",
                vec![("somemodel", "200000", "200000", "32000")],
                cx,
            )
            .await,
            Some("Provider Name is already taken by another provider".into())
        );
    }

    #[gpui::test]
    async fn test_model_input_default_capabilities(cx: &mut TestAppContext) {
        let cx = setup_test(cx).await;

        cx.update(|window, cx| {
            let model_input = ModelInput::new(0, window, cx);
            model_input.name.update(cx, |input, cx| {
                input.set_text("somemodel", window, cx);
            });
            assert_eq!(
                model_input.capabilities.supports_tools,
                ToggleState::Selected
            );
            assert_eq!(
                model_input.capabilities.supports_images,
                ToggleState::Unselected
            );
            assert_eq!(
                model_input.capabilities.supports_parallel_tool_calls,
                ToggleState::Unselected
            );
            assert_eq!(
                model_input.capabilities.supports_prompt_cache_key,
                ToggleState::Unselected
            );
            assert_eq!(
                model_input.capabilities.supports_chat_completions,
                ToggleState::Selected
            );

            let parsed_model = model_input.parse(cx).unwrap();
            assert!(parsed_model.capabilities.tools);
            assert!(!parsed_model.capabilities.images);
            assert!(!parsed_model.capabilities.parallel_tool_calls);
            assert!(!parsed_model.capabilities.prompt_cache_key);
            assert!(parsed_model.capabilities.chat_completions);
        });
    }

    #[gpui::test]
    async fn test_model_input_deselected_capabilities(cx: &mut TestAppContext) {
        let cx = setup_test(cx).await;

        cx.update(|window, cx| {
            let mut model_input = ModelInput::new(0, window, cx);
            model_input.name.update(cx, |input, cx| {
                input.set_text("somemodel", window, cx);
            });

            model_input.capabilities.supports_tools = ToggleState::Unselected;
            model_input.capabilities.supports_images = ToggleState::Unselected;
            model_input.capabilities.supports_parallel_tool_calls = ToggleState::Unselected;
            model_input.capabilities.supports_prompt_cache_key = ToggleState::Unselected;
            model_input.capabilities.supports_chat_completions = ToggleState::Unselected;

            let parsed_model = model_input.parse(cx).unwrap();
            assert!(!parsed_model.capabilities.tools);
            assert!(!parsed_model.capabilities.images);
            assert!(!parsed_model.capabilities.parallel_tool_calls);
            assert!(!parsed_model.capabilities.prompt_cache_key);
            assert!(!parsed_model.capabilities.chat_completions);
        });
    }

    #[gpui::test]
    async fn test_model_input_with_name_and_capabilities(cx: &mut TestAppContext) {
        let cx = setup_test(cx).await;

        cx.update(|window, cx| {
            let mut model_input = ModelInput::new(0, window, cx);
            model_input.name.update(cx, |input, cx| {
                input.set_text("somemodel", window, cx);
            });

            model_input.capabilities.supports_tools = ToggleState::Selected;
            model_input.capabilities.supports_images = ToggleState::Unselected;
            model_input.capabilities.supports_parallel_tool_calls = ToggleState::Selected;
            model_input.capabilities.supports_prompt_cache_key = ToggleState::Unselected;
            model_input.capabilities.supports_chat_completions = ToggleState::Selected;

            let parsed_model = model_input.parse(cx).unwrap();
            assert_eq!(parsed_model.name, "somemodel");
            assert!(parsed_model.capabilities.tools);
            assert!(!parsed_model.capabilities.images);
            assert!(parsed_model.capabilities.parallel_tool_calls);
            assert!(!parsed_model.capabilities.prompt_cache_key);
            assert!(parsed_model.capabilities.chat_completions);
        });
    }

    async fn setup_test(cx: &mut TestAppContext) -> &mut VisualTestContext {
        cx.update(|cx| {
            let store = SettingsStore::test(cx);
            cx.set_global(store);
            theme_settings::init(theme::LoadThemes::JustBase, cx);

            language_model::init_settings(cx);
            editor::init(cx);
        });

        let fs = FakeFs::new(cx.executor());
        cx.update(|cx| <dyn Fs>::set_global(fs.clone(), cx));
        let project = Project::test(fs, [path!("/dir").as_ref()], cx).await;
        let (multi_workspace, cx) =
            cx.add_window_view(|window, cx| MultiWorkspace::test_new(project.clone(), window, cx));
        let _workspace = multi_workspace.read_with(cx, |mw, _| mw.workspace().clone());

        cx
    }

    async fn save_provider_validation_errors(
        provider_name: &str,
        api_url: &str,
        api_key: &str,
        models: Vec<(&str, &str, &str, &str)>,
        cx: &mut VisualTestContext,
    ) -> Option<SharedString> {
        fn set_text(input: &Entity<InputField>, text: &str, window: &mut Window, cx: &mut App) {
            input.update(cx, |input, cx| {
                input.set_text(text, window, cx);
            });
        }

        let task = cx.update(|window, cx| {
            let mut input = AddLlmProviderInput::new(LlmCompatibleProvider::OpenAi, window, cx);
            set_text(&input.provider_name, provider_name, window, cx);
            set_text(&input.api_url, api_url, window, cx);
            set_text(&input.api_key, api_key, window, cx);

            for (i, (name, max_tokens, max_completion_tokens, max_output_tokens)) in
                models.iter().enumerate()
            {
                if i >= input.models.len() {
                    input.models.push(ModelInput::new(i, window, cx));
                }
                let model = &mut input.models[i];
                set_text(&model.name, name, window, cx);
                set_text(&model.max_tokens, max_tokens, window, cx);
                set_text(
                    &model.max_completion_tokens,
                    max_completion_tokens,
                    window,
                    cx,
                );
                set_text(&model.max_output_tokens, max_output_tokens, window, cx);
            }
            save_provider_to_settings(LlmCompatibleProvider::OpenAi, &input, cx)
        });

        task.await.err()
    }
}
