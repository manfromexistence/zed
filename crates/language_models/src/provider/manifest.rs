use std::sync::Arc;

use anyhow::Result;
use futures::{
    AsyncBufReadExt, AsyncReadExt, FutureExt, StreamExt, future::BoxFuture, io::BufReader,
};
use gpui::{AnyView, App, AsyncApp, Context, Entity, SharedString, Task};
use http_client::{AsyncBody, Builder as HttpRequestBuilder, HttpClient, Method, Request as HttpRequest};
use language_model::{
    ApiKeyState, AuthenticateError, EnvVar, IconOrSvg, LanguageModel, LanguageModelCompletionError,
    LanguageModelCompletionEvent, LanguageModelCostInfo, LanguageModelId, LanguageModelName,
    LanguageModelProvider, LanguageModelProviderId, LanguageModelProviderName,
    LanguageModelProviderState, LanguageModelRequest, LanguageModelToolChoice,
    LanguageModelToolSchemaFormat, RateLimiter,
};
use menu;
use open_ai::{
    RequestError, ResponseStreamEvent, ResponseStreamResult,
    responses::{Request as ResponseRequest, StreamEvent as ResponsesStreamEvent},
};
use ui::{ElevationIndex, Tooltip, prelude::*};
use ui_input::InputField;
use util::ResultExt;

use crate::provider::open_ai::{
    OpenAiEventMapper, OpenAiResponseEventMapper, into_open_ai, into_open_ai_response,
};
use crate::provider_hub::auth_strategy::{ApiKeyHeaderStyle, AuthStrategy};
use crate::provider_hub::{ProviderIcon, ProviderManifest, ProviderModelCapabilities};
use crate::provider_icons;

pub struct ManifestLanguageModelProvider {
    manifest: ProviderManifest,
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
}

pub struct State {
    manifest: ProviderManifest,
    api_key_state: Option<ApiKeyState>,
}

impl State {
    fn is_authenticated(&self) -> bool {
        self.api_key_state
            .as_ref()
            .map(|state| state.has_key())
            .unwrap_or(true)
    }

    fn set_api_key(&mut self, api_key: Option<String>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let Some(state) = &mut self.api_key_state else {
            return Task::ready(Ok(()));
        };
        state.store(
            SharedString::new(self.manifest.api_base.as_str()),
            api_key,
            |this| this.api_key_state.as_mut().expect("api key state"),
            cx,
        )
    }

    fn authenticate(&mut self, cx: &mut Context<Self>) -> Task<Result<(), AuthenticateError>> {
        let Some(state) = &mut self.api_key_state else {
            return Task::ready(Ok(()));
        };
        state.load_if_needed(
            SharedString::new(self.manifest.api_base.as_str()),
            |this| this.api_key_state.as_mut().expect("api key state"),
            cx,
        )
    }
}

impl ManifestLanguageModelProvider {
    pub fn new(manifest: ProviderManifest, http_client: Arc<dyn HttpClient>, cx: &mut App) -> Self {
        let state = cx.new(|_cx| State {
            api_key_state: match &manifest.auth {
                AuthStrategy::ApiKey { env_var, .. } => Some(ApiKeyState::new(
                    SharedString::new(manifest.api_base.as_str()),
                    EnvVar::new(
                        env_var
                            .clone()
                            .unwrap_or_else(|| {
                                AuthStrategy::default_env_var_for_provider(&manifest.id)
                            })
                            .into(),
                    ),
                )),
                _ => None,
            },
            manifest: manifest.clone(),
        });

        Self {
            manifest,
            http_client,
            state,
        }
    }

    fn create_language_model(
        &self,
        model: crate::provider_hub::ModelManifest,
    ) -> Arc<dyn LanguageModel> {
        Arc::new(ManifestLanguageModel {
            provider_id: LanguageModelProviderId::from(self.manifest.id.clone()),
            provider_name: LanguageModelProviderName::from(self.manifest.display_name.clone()),
            model,
            state: self.state.clone(),
            http_client: self.http_client.clone(),
            request_limiter: RateLimiter::new(4),
        })
    }
}

fn apply_auth_headers(
    request_builder: HttpRequestBuilder,
    auth: &AuthStrategy,
    api_key: Option<&Arc<str>>,
    provider_name: &LanguageModelProviderName,
) -> Result<HttpRequestBuilder, LanguageModelCompletionError> {
    match auth {
        AuthStrategy::NoAuth => Ok(request_builder),
        AuthStrategy::ApiKey { header_style, .. } => {
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey {
                    provider: provider_name.clone(),
                });
            };
            let trimmed_key = api_key.trim();
            let request_builder = match header_style {
                ApiKeyHeaderStyle::Bearer => {
                    request_builder.header("Authorization", format!("Bearer {trimmed_key}"))
                }
                ApiKeyHeaderStyle::XApiKey => request_builder.header("x-api-key", trimmed_key),
                ApiKeyHeaderStyle::AuthorizationApiKey => {
                    request_builder.header("Authorization", format!("Api-Key {trimmed_key}"))
                }
            };
            Ok(request_builder)
        }
        unsupported => Err(LanguageModelCompletionError::Other(anyhow::anyhow!(
            "Unsupported auth strategy for {}: {unsupported:?}",
            provider_name.0
        ))),
    }
}

impl LanguageModelProviderState for ManifestLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for ManifestLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        LanguageModelProviderId::from(self.manifest.id.clone())
    }

    fn name(&self) -> LanguageModelProviderName {
        LanguageModelProviderName::from(self.manifest.display_name.clone())
    }

    fn icon(&self) -> IconOrSvg {
        if let Some(path) = provider_icons::get_provider_icon_path(&self.manifest.id) {
            return IconOrSvg::Svg(path.into());
        }

        match self.manifest.icon {
            ProviderIcon::OpenAi => IconOrSvg::Icon(IconName::AiOpenAi),
            ProviderIcon::OpenRouter => IconOrSvg::Icon(IconName::AiOpenRouter),
            ProviderIcon::Anthropic => IconOrSvg::Icon(IconName::AiAnthropic),
            ProviderIcon::Google => IconOrSvg::Icon(IconName::AiGoogle),
            ProviderIcon::Bedrock => IconOrSvg::Icon(IconName::AiBedrock),
            ProviderIcon::Mistral => IconOrSvg::Icon(IconName::AiMistral),
            ProviderIcon::XAi => IconOrSvg::Icon(IconName::AiXAi),
            ProviderIcon::Copilot => IconOrSvg::Icon(IconName::Copilot),
            ProviderIcon::OpenAiCompatible | ProviderIcon::Generic => {
                IconOrSvg::Icon(IconName::AiOpenAiCompat)
            }
        }
    }

    fn default_model(&self, _cx: &App) -> Option<Arc<dyn LanguageModel>> {
        self.manifest
            .models
            .first()
            .cloned()
            .map(|model| self.create_language_model(model))
    }

    fn default_fast_model(&self, cx: &App) -> Option<Arc<dyn LanguageModel>> {
        self.default_model(cx)
    }

    fn provided_models(&self, _cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        self.manifest
            .models
            .iter()
            .cloned()
            .map(|model| self.create_language_model(model))
            .collect()
    }

    fn recommended_models(&self, _cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        self.manifest
            .models
            .iter()
            .take(3)
            .cloned()
            .map(|model| self.create_language_model(model))
            .collect()
    }

    fn is_authenticated(&self, cx: &App) -> bool {
        self.state.read(cx).is_authenticated()
    }

    fn authenticate(&self, cx: &mut App) -> Task<Result<(), AuthenticateError>> {
        self.state.update(cx, |state, cx| state.authenticate(cx))
    }

    fn configuration_view(
        &self,
        _target_agent: language_model::ConfigurationViewTargetAgent,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| ConfigurationView::new(self.state.clone(), window, cx))
            .into()
    }

    fn reset_credentials(&self, cx: &mut App) -> Task<Result<()>> {
        self.state
            .update(cx, |state, cx| state.set_api_key(None, cx))
    }
}

pub struct ManifestLanguageModel {
    provider_id: LanguageModelProviderId,
    provider_name: LanguageModelProviderName,
    model: crate::provider_hub::ModelManifest,
    state: Entity<State>,
    http_client: Arc<dyn HttpClient>,
    request_limiter: RateLimiter,
}

impl ManifestLanguageModel {
    fn auth_key(&self, cx: &AsyncApp) -> Option<Arc<str>> {
        self.state.read_with(cx, |state, _| {
            state
                .api_key_state
                .as_ref()
                .and_then(|api_key_state| api_key_state.key(&state.manifest.api_base))
        })
    }

    fn stream_completion_impl(
        &self,
        request: open_ai::Request,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            futures::stream::BoxStream<'static, Result<ResponseStreamEvent>>,
            LanguageModelCompletionError,
        >,
    > {
        let http_client = self.http_client.clone();
        let provider_name = self.provider_name.clone();
        let (api_base, auth_strategy) = self.state.read_with(cx, |state, _| {
            (state.manifest.api_base.clone(), state.manifest.auth.clone())
        });
        let api_key = self.auth_key(cx);
        let future = self.request_limiter.stream(async move {
            let uri = format!("{api_base}/chat/completions");
            let request_builder = HttpRequest::builder()
                .method(Method::POST)
                .uri(uri)
                .header("Content-Type", "application/json");
            let request_builder =
                apply_auth_headers(request_builder, &auth_strategy, api_key.as_ref(), &provider_name)?;

            let request = request_builder
                .body(AsyncBody::from(
                    serde_json::to_string(&request)
                        .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?,
                ))
                .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?;

            let mut response = http_client.send(request).await?;
            if response.status().is_success() {
                let reader = BufReader::new(response.into_body());
                let provider_name_for_log = provider_name.0.to_string();
                Ok(reader
                    .lines()
                    .filter_map(move |line| {
                        let provider_name_for_log = provider_name_for_log.clone();
                        async move {
                        match line {
                            Ok(line) => {
                                let line = line
                                    .strip_prefix("data: ")
                                    .or_else(|| line.strip_prefix("data:"))?;
                                if line == "[DONE]" {
                                    None
                                } else {
                                    match serde_json::from_str(line) {
                                        Ok(ResponseStreamResult::Ok(response)) => Some(Ok(response)),
                                        Ok(ResponseStreamResult::Err { error }) => {
                                            Some(Err(anyhow::anyhow!("{error:?}")))
                                        }
                                        Err(error) => {
                                            log::error!(
                                                "Failed to parse {} stream response: `{}`\nResponse: `{}`",
                                                provider_name_for_log,
                                                error,
                                                line,
                                            );
                                            Some(Err(anyhow::anyhow!(error)))
                                        }
                                    }
                                }
                            }
                            Err(error) => Some(Err(anyhow::anyhow!(error))),
                        }
                    }})
                    .boxed())
            } else {
                let mut body = String::new();
                response
                    .body_mut()
                    .read_to_string(&mut body)
                    .await
                    .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?;

                Err(RequestError::HttpResponseError {
                    provider: provider_name.0.to_string(),
                    status_code: response.status(),
                    body,
                    headers: response.headers().clone(),
                }
                .into())
            }
        });
        async move { Ok(future.await?.boxed()) }.boxed()
    }

    fn stream_response_impl(
        &self,
        request: ResponseRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<'static, Result<futures::stream::BoxStream<'static, Result<ResponsesStreamEvent>>>>
    {
        let http_client = self.http_client.clone();
        let provider_name = self.provider_name.clone();
        let (api_base, auth_strategy) = self.state.read_with(cx, |state, _| {
            (state.manifest.api_base.clone(), state.manifest.auth.clone())
        });
        let api_key = self.auth_key(cx);
        let future = self.request_limiter.stream(async move {
            let uri = format!("{api_base}/responses");
            let request_builder = HttpRequest::builder()
                .method(Method::POST)
                .uri(uri)
                .header("Content-Type", "application/json");
            let request_builder =
                apply_auth_headers(request_builder, &auth_strategy, api_key.as_ref(), &provider_name)?;

            let is_streaming = request.stream;
            let request = request_builder
                .body(AsyncBody::from(
                    serde_json::to_string(&request)
                        .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?,
                ))
                .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?;

            let mut response = http_client.send(request).await?;
            if response.status().is_success() {
                if is_streaming {
                    let reader = BufReader::new(response.into_body());
                    let provider_name_for_log = provider_name.0.to_string();
                    Ok(reader
                        .lines()
                        .filter_map(move |line| {
                            let provider_name_for_log = provider_name_for_log.clone();
                            async move {
                            match line {
                                Ok(line) => {
                                    let line = line
                                        .strip_prefix("data: ")
                                        .or_else(|| line.strip_prefix("data:"))?;
                                    if line == "[DONE]" || line.is_empty() {
                                        None
                                    } else {
                                        match serde_json::from_str::<ResponsesStreamEvent>(line) {
                                            Ok(event) => Some(Ok(event)),
                                            Err(error) => {
                                                log::error!(
                                                    "Failed to parse {} responses stream event: `{}`\nResponse: `{}`",
                                                    provider_name_for_log,
                                                    error,
                                                    line,
                                                );
                                                Some(Err(anyhow::anyhow!(error)))
                                            }
                                        }
                                    }
                                }
                                Err(error) => Some(Err(anyhow::anyhow!(error))),
                            }
                        }})
                        .boxed())
                } else {
                    let mut body = String::new();
                    response
                        .body_mut()
                        .read_to_string(&mut body)
                        .await
                        .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?;
                    let response_event: ResponsesStreamEvent = serde_json::from_str(&body)
                        .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?;
                    Ok(futures::stream::iter([Ok(response_event)]).boxed())
                }
            } else {
                let mut body = String::new();
                response
                    .body_mut()
                    .read_to_string(&mut body)
                    .await
                    .map_err(|error| RequestError::Other(anyhow::Error::from(error)))?;

                Err(RequestError::HttpResponseError {
                    provider: provider_name.0.to_string(),
                    status_code: response.status(),
                    body,
                    headers: response.headers().clone(),
                }
                .into())
            }
        });
        async move { Ok(future.await?.boxed()) }.boxed()
    }
}

impl LanguageModel for ManifestLanguageModel {
    fn id(&self) -> LanguageModelId {
        LanguageModelId::from(self.model.id.clone())
    }

    fn name(&self) -> LanguageModelName {
        LanguageModelName::from(
            self.model
                .display_name
                .clone()
                .unwrap_or_else(|| self.model.id.clone()),
        )
    }

    fn provider_id(&self) -> LanguageModelProviderId {
        self.provider_id.clone()
    }

    fn provider_name(&self) -> LanguageModelProviderName {
        self.provider_name.clone()
    }

    fn supports_tools(&self) -> bool {
        self.model.capabilities.tools
    }

    fn supports_tool_choice(&self, choice: LanguageModelToolChoice) -> bool {
        match choice {
            LanguageModelToolChoice::Auto | LanguageModelToolChoice::Any => {
                self.model.capabilities.tools
            }
            LanguageModelToolChoice::None => true,
        }
    }

    fn supports_streaming_tools(&self) -> bool {
        self.model.capabilities.tools
    }

    fn supports_split_token_display(&self) -> bool {
        self.model.metadata.input_cost_per_1m.is_some()
            || self.model.metadata.output_cost_per_1m.is_some()
    }

    fn supports_images(&self) -> bool {
        self.model.capabilities.images
    }

    fn supports_thinking(&self) -> bool {
        self.model.metadata.supports_reasoning
    }

    fn tool_input_format(&self) -> LanguageModelToolSchemaFormat {
        LanguageModelToolSchemaFormat::JsonSchemaSubset
    }

    fn model_cost_info(&self) -> Option<LanguageModelCostInfo> {
        Some(LanguageModelCostInfo::TokenCost {
            input_token_cost_per_1m: self.model.metadata.input_cost_per_1m?,
            output_token_cost_per_1m: self.model.metadata.output_cost_per_1m?,
        })
    }

    fn telemetry_id(&self) -> String {
        format!("{}/{}", self.provider_id.0, self.model.id)
    }

    fn max_token_count(&self) -> u64 {
        self.model.max_tokens
    }

    fn max_output_tokens(&self) -> Option<u64> {
        self.model.max_output_tokens
    }

    fn count_tokens(
        &self,
        request: LanguageModelRequest,
        cx: &App,
    ) -> BoxFuture<'static, Result<u64>> {
        let max_token_count = self.max_token_count();
        cx.background_spawn(async move {
            let messages = crate::provider::open_ai::collect_tiktoken_messages(request);
            let tokenizer = if max_token_count >= 100_000 {
                "gpt-4o"
            } else {
                "gpt-4"
            };
            tiktoken_rs::num_tokens_from_messages(tokenizer, &messages)
                .map(|tokens| tokens as u64)
                .map_err(Into::into)
        })
        .boxed()
    }

    fn stream_completion(
        &self,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            futures::stream::BoxStream<
                'static,
                Result<LanguageModelCompletionEvent, LanguageModelCompletionError>,
            >,
            LanguageModelCompletionError,
        >,
    > {
        let capabilities: ProviderModelCapabilities = self.model.capabilities.clone();
        if capabilities.chat_completions {
            let request = into_open_ai(
                request,
                &self.model.id,
                capabilities.parallel_tool_calls,
                capabilities.prompt_cache_key,
                self.max_output_tokens(),
                None,
            );
            let completions = self.stream_completion_impl(request, cx);
            async move {
                let mapper = OpenAiEventMapper::new();
                Ok(mapper.map_stream(completions.await?).boxed())
            }
            .boxed()
        } else {
            let request = into_open_ai_response(
                request,
                &self.model.id,
                capabilities.parallel_tool_calls,
                capabilities.prompt_cache_key,
                self.max_output_tokens(),
                None,
            );
            let responses = self.stream_response_impl(request, cx);
            async move {
                let mapper = OpenAiResponseEventMapper::new();
                Ok(mapper.map_stream(responses.await?).boxed())
            }
            .boxed()
        }
    }
}

struct ConfigurationView {
    api_key_editor: Entity<InputField>,
    state: Entity<State>,
    load_credentials_task: Option<Task<()>>,
}

impl ConfigurationView {
    fn new(state: Entity<State>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let api_key_editor = cx.new(|cx| {
            InputField::new(
                window,
                cx,
                "000000000000000000000000000000000000000000000000000",
            )
            .label("API Key")
            .masked(true)
        });

        cx.observe(&state, |_, _, cx| {
            cx.notify();
        })
        .detach();

        let load_credentials_task = Some(cx.spawn_in(window, {
            let state = state.clone();
            async move |this, cx| {
                let _ = state.update(cx, |state, cx| state.authenticate(cx)).await;
                this.update(cx, |this, cx| {
                    this.load_credentials_task = None;
                    cx.notify();
                })
                .log_err();
            }
        }));

        Self {
            api_key_editor,
            state,
            load_credentials_task,
        }
    }

    fn save_api_key(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_editor.read(cx).text(cx).trim().to_string();
        if api_key.is_empty() {
            return;
        }

        self.api_key_editor
            .update(cx, |input, cx| input.set_text("", window, cx));
        let state = self.state.clone();
        cx.spawn_in(window, async move |_, cx| {
            state
                .update(cx, |state, cx| state.set_api_key(Some(api_key), cx))
                .await
        })
        .detach_and_log_err(cx);
    }

    fn reset_api_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.api_key_editor
            .update(cx, |input, cx| input.set_text("", window, cx));
        let state = self.state.clone();
        cx.spawn_in(window, async move |_, cx| {
            state
                .update(cx, |state, cx| state.set_api_key(None, cx))
                .await
        })
        .detach_and_log_err(cx);
    }
}

impl Render for ConfigurationView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let model_count = state.manifest.models.len();
        let requires_api_key = matches!(&state.manifest.auth, AuthStrategy::ApiKey { .. });
        let env_var_name = state
            .manifest
            .auth
            .env_var_name_for_provider(&state.manifest.id)
            .unwrap_or_else(|| "API_KEY".to_owned());

        let content = if !requires_api_key {
            h_flex()
                .mt_1()
                .p_1()
                .justify_between()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border)
                .bg(cx.theme().colors().background)
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .child(Icon::new(IconName::Check).color(Color::Success))
                        .child(div().w_full().overflow_x_hidden().text_ellipsis().child(
                            Label::new(format!(
                                "{} exposes {model_count} synced models and does not require an API key.",
                                state.manifest.display_name
                            )),
                        )),
                )
                .into_any()
        } else if state
            .api_key_state
            .as_ref()
            .is_none_or(|api_key_state| !api_key_state.has_key())
        {
            v_flex()
                .gap_2()
                .on_action(cx.listener(Self::save_api_key))
                .child(Label::new(format!(
                    "{} exposes {model_count} synced models. Add an API key to use them inside the agent.",
                    state.manifest.display_name
                )))
                .child(self.api_key_editor.clone())
                .child(
                    Label::new(format!(
                        "You can also set {env_var_name} in your environment and restart Zed."
                    ))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
                )
                .into_any()
        } else {
            h_flex()
                .mt_1()
                .p_1()
                .justify_between()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border)
                .bg(cx.theme().colors().background)
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .child(Icon::new(IconName::Check).color(Color::Success))
                        .child(div().w_full().overflow_x_hidden().text_ellipsis().child(
                            Label::new(format!(
                                "API key configured for {} at {}",
                                state.manifest.display_name, state.manifest.api_base
                            )),
                        )),
                )
                .child(
                    Button::new("reset-api-key", "Reset API Key")
                        .label_size(LabelSize::Small)
                        .start_icon(Icon::new(IconName::Undo).size(IconSize::Small))
                        .layer(ElevationIndex::ModalSurface)
                        .tooltip(Tooltip::text(format!(
                            "Reset the stored API key for {}",
                            state.manifest.display_name
                        )))
                        .on_click(
                            cx.listener(|this, _, window, cx| this.reset_api_key(window, cx)),
                        ),
                )
                .into_any()
        };

        if self.load_credentials_task.is_some() {
            div().child(Label::new("Loading credentials...")).into_any()
        } else {
            content
        }
    }
}
