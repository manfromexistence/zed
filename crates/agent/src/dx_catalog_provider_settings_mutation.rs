use dx_catalog::{
    CatalogExecutionAdapterKind, CatalogProviderAdapterModelSpec,
    CatalogProviderAdapterRegistrationSpec,
};
use settings::{
    OpenAiCompatibleAvailableModel, OpenAiCompatibleModelCapabilities,
    OpenAiCompatibleSettingsContent, OpenRouterAvailableModel,
};
use std::sync::Arc;

const DEFAULT_CATALOG_MODEL_MAX_TOKENS: u64 = 200_000;

pub(crate) fn catalog_base_url_matches_settings(
    catalog_url: Option<&str>,
    settings_url: Option<&str>,
) -> bool {
    let Some(catalog_url) = catalog_url else {
        return true;
    };
    let Some(settings_url) = settings_url else {
        return false;
    };
    normalized_url(catalog_url) == normalized_url(settings_url)
}

pub(crate) fn can_write_provider_settings(spec: &CatalogProviderAdapterRegistrationSpec) -> bool {
    spec.can_register_settings
        && matches!(
            spec.adapter_kind,
            CatalogExecutionAdapterKind::OpenAiCompatibleHttp
                | CatalogExecutionAdapterKind::OllamaCompatibleHttp
                | CatalogExecutionAdapterKind::OpenRouterHttp
                | CatalogExecutionAdapterKind::LiteLlmProxy
        )
        && spec
            .base_url
            .as_ref()
            .is_some_and(|url| !url.trim().is_empty())
        && !spec.models.is_empty()
}

pub(crate) fn apply_provider_settings_spec(
    settings: &mut settings::SettingsContent,
    spec: &CatalogProviderAdapterRegistrationSpec,
) {
    match spec.adapter_kind {
        CatalogExecutionAdapterKind::OpenRouterHttp => {
            apply_open_router_provider_settings(settings, spec);
        }
        CatalogExecutionAdapterKind::OpenAiCompatibleHttp
        | CatalogExecutionAdapterKind::OllamaCompatibleHttp
        | CatalogExecutionAdapterKind::LiteLlmProxy => {
            apply_openai_compatible_provider_settings(settings, spec);
        }
        _ => {}
    }
}

fn normalized_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_ascii_lowercase()
}

fn apply_openai_compatible_provider_settings(
    settings: &mut settings::SettingsContent,
    spec: &CatalogProviderAdapterRegistrationSpec,
) {
    let Some(api_url) = spec.base_url.clone() else {
        return;
    };
    let language_models = settings.language_models.get_or_insert_default();
    let providers = language_models.openai_compatible.get_or_insert_default();
    let provider = providers
        .entry(Arc::from(spec.provider_id.as_str()))
        .or_insert_with(|| OpenAiCompatibleSettingsContent {
            api_url: api_url.clone(),
            available_models: Vec::new(),
        });

    if provider.api_url.trim().is_empty() {
        provider.api_url = api_url;
    } else if !catalog_base_url_matches_settings(
        Some(api_url.as_str()),
        Some(provider.api_url.as_str()),
    ) {
        log::warn!(
            "DX catalog provider settings registration skipped `{}` because existing api_url does not match catalog api_url",
            spec.provider_id
        );
        return;
    }

    for model in &spec.models {
        upsert_openai_compatible_model(
            &mut provider.available_models,
            openai_compatible_model_from_catalog(model),
        );
    }
}

fn apply_open_router_provider_settings(
    settings: &mut settings::SettingsContent,
    spec: &CatalogProviderAdapterRegistrationSpec,
) {
    let language_models = settings.language_models.get_or_insert_default();
    let open_router = language_models.open_router.get_or_insert_default();
    let should_set_api_url = open_router
        .api_url
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty();
    if should_set_api_url {
        if let Some(api_url) = &spec.base_url {
            open_router.api_url = Some(api_url.clone());
        }
    } else if !catalog_base_url_matches_settings(
        spec.base_url.as_deref(),
        open_router.api_url.as_deref(),
    ) {
        log::warn!(
            "DX catalog provider settings registration skipped OpenRouter because existing api_url does not match catalog api_url"
        );
        return;
    }

    let models = open_router.available_models.get_or_insert_default();
    for model in &spec.models {
        upsert_open_router_model(models, open_router_model_from_catalog(model));
    }
}

fn openai_compatible_model_from_catalog(
    model: &CatalogProviderAdapterModelSpec,
) -> OpenAiCompatibleAvailableModel {
    OpenAiCompatibleAvailableModel {
        name: model.api_model_id.clone(),
        display_name: Some(model.display_name.clone()),
        max_tokens: model
            .context_window_tokens
            .map(u64::from)
            .unwrap_or(DEFAULT_CATALOG_MODEL_MAX_TOKENS),
        max_output_tokens: model.max_output_tokens.map(u64::from),
        max_completion_tokens: model.max_output_tokens.map(u64::from),
        reasoning_effort: None,
        capabilities: OpenAiCompatibleModelCapabilities {
            tools: model.supports_tools,
            images: model.supports_images,
            parallel_tool_calls: false,
            prompt_cache_key: false,
            chat_completions: true,
            interleaved_reasoning: false,
        },
    }
}

fn open_router_model_from_catalog(
    model: &CatalogProviderAdapterModelSpec,
) -> OpenRouterAvailableModel {
    OpenRouterAvailableModel {
        name: model.api_model_id.clone(),
        display_name: Some(model.display_name.clone()),
        max_tokens: model
            .context_window_tokens
            .map(u64::from)
            .unwrap_or(DEFAULT_CATALOG_MODEL_MAX_TOKENS),
        max_output_tokens: model.max_output_tokens.map(u64::from),
        max_completion_tokens: model.max_output_tokens.map(u64::from),
        supports_tools: Some(model.supports_tools),
        supports_images: Some(model.supports_images),
        mode: None,
        provider: None,
    }
}

fn upsert_openai_compatible_model(
    models: &mut Vec<OpenAiCompatibleAvailableModel>,
    model: OpenAiCompatibleAvailableModel,
) {
    if let Some(existing) = models
        .iter_mut()
        .find(|existing| existing.name == model.name)
    {
        *existing = model;
    } else {
        models.push(model);
    }
}

fn upsert_open_router_model(
    models: &mut Vec<OpenRouterAvailableModel>,
    model: OpenRouterAvailableModel,
) {
    if let Some(existing) = models
        .iter_mut()
        .find(|existing| existing.name == model.name)
    {
        *existing = model;
    } else {
        models.push(model);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dx_catalog::CatalogExecutionPermission;
    use settings::{OpenRouterSettingsContent, SettingsContent};

    #[test]
    fn openai_compatible_settings_skip_catalog_models_when_api_url_differs() {
        let mut settings = SettingsContent::default();
        let language_models = settings.language_models.get_or_insert_default();
        let providers = language_models.openai_compatible.get_or_insert_default();
        providers.insert(
            Arc::from("groq"),
            OpenAiCompatibleSettingsContent {
                api_url: "https://wrong.example/v1".to_string(),
                available_models: vec![openai_compatible_available_model("existing-model")],
            },
        );

        apply_openai_compatible_provider_settings(
            &mut settings,
            &registration_spec(
                "groq",
                CatalogExecutionAdapterKind::OpenAiCompatibleHttp,
                "https://api.groq.com/openai/v1",
            ),
        );

        let provider = settings
            .language_models
            .as_ref()
            .and_then(|settings| settings.openai_compatible.as_ref())
            .and_then(|providers| providers.get("groq"))
            .expect("existing provider settings should remain");

        assert_eq!(provider.api_url, "https://wrong.example/v1");
        assert_eq!(provider.available_models.len(), 1);
        assert_eq!(provider.available_models[0].name, "existing-model");
    }

    #[test]
    fn open_router_settings_skip_catalog_models_when_api_url_differs() {
        let mut settings = SettingsContent::default();
        settings.language_models.get_or_insert_default().open_router =
            Some(OpenRouterSettingsContent {
                api_url: Some("https://wrong.example/api/v1".to_string()),
                available_models: Some(vec![open_router_available_model("existing-model")]),
            });

        apply_open_router_provider_settings(
            &mut settings,
            &registration_spec(
                "openrouter",
                CatalogExecutionAdapterKind::OpenRouterHttp,
                "https://openrouter.ai/api/v1",
            ),
        );

        let open_router = settings
            .language_models
            .as_ref()
            .and_then(|settings| settings.open_router.as_ref())
            .expect("existing OpenRouter settings should remain");
        let models = open_router
            .available_models
            .as_ref()
            .expect("existing OpenRouter models should remain");

        assert_eq!(
            open_router.api_url.as_deref(),
            Some("https://wrong.example/api/v1")
        );
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "existing-model");
    }

    #[test]
    fn provider_settings_write_api_model_ids_not_catalog_route_ids() {
        let mut settings = SettingsContent::default();

        apply_openai_compatible_provider_settings(
            &mut settings,
            &registration_spec(
                "groq",
                CatalogExecutionAdapterKind::OpenAiCompatibleHttp,
                "https://api.groq.com/openai/v1",
            ),
        );

        let provider = settings
            .language_models
            .as_ref()
            .and_then(|settings| settings.openai_compatible.as_ref())
            .and_then(|providers| providers.get("groq"))
            .expect("catalog provider settings should be written");

        assert_eq!(provider.available_models.len(), 1);
        assert_eq!(provider.available_models[0].name, "catalog-model");
        assert_ne!(provider.available_models[0].name, "groq/catalog-model");
    }

    #[test]
    fn open_router_settings_write_api_model_ids_not_catalog_route_ids() {
        let mut settings = SettingsContent::default();

        apply_open_router_provider_settings(
            &mut settings,
            &registration_spec(
                "openrouter",
                CatalogExecutionAdapterKind::OpenRouterHttp,
                "https://openrouter.ai/api/v1",
            ),
        );

        let models = settings
            .language_models
            .as_ref()
            .and_then(|settings| settings.open_router.as_ref())
            .and_then(|settings| settings.available_models.as_ref())
            .expect("catalog OpenRouter models should be written");

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "catalog-model");
        assert_ne!(models[0].name, "openrouter/catalog-model");
    }

    fn registration_spec(
        provider_id: &str,
        adapter_kind: CatalogExecutionAdapterKind,
        base_url: &str,
    ) -> CatalogProviderAdapterRegistrationSpec {
        CatalogProviderAdapterRegistrationSpec {
            provider_id: provider_id.to_string(),
            provider_name: provider_id.to_string(),
            adapter_kind,
            permission: CatalogExecutionPermission::ApiKey,
            settings_path: Some(format!("language_models.openai_compatible.{provider_id}")),
            base_url: Some(base_url.to_string()),
            auth_profile_id: None,
            auth_configured: false,
            user_approval_required: true,
            can_register_settings: true,
            ready_for_execution: false,
            registration_blockers: Vec::new(),
            execution_blockers: Vec::new(),
            models: vec![CatalogProviderAdapterModelSpec {
                model_id: format!("{provider_id}/catalog-model"),
                api_model_id: "catalog-model".to_string(),
                display_name: "Catalog Model".to_string(),
                context_window_tokens: Some(128_000),
                max_output_tokens: Some(8_192),
                supports_tools: true,
                supports_images: false,
                supports_audio: false,
                supports_video: false,
                supports_streaming: true,
                free_tier: false,
                premium_account: true,
            }],
            next_action: "Register provider settings".to_string(),
        }
    }

    fn openai_compatible_available_model(name: &str) -> OpenAiCompatibleAvailableModel {
        OpenAiCompatibleAvailableModel {
            name: name.to_string(),
            display_name: Some(name.to_string()),
            max_tokens: 8_192,
            max_output_tokens: Some(1_024),
            max_completion_tokens: Some(1_024),
            reasoning_effort: None,
            capabilities: OpenAiCompatibleModelCapabilities {
                tools: true,
                images: false,
                parallel_tool_calls: false,
                prompt_cache_key: false,
                chat_completions: true,
                interleaved_reasoning: false,
            },
        }
    }

    fn open_router_available_model(name: &str) -> OpenRouterAvailableModel {
        OpenRouterAvailableModel {
            name: name.to_string(),
            display_name: Some(name.to_string()),
            max_tokens: 8_192,
            max_output_tokens: Some(1_024),
            max_completion_tokens: Some(1_024),
            supports_tools: Some(true),
            supports_images: Some(false),
            mode: None,
            provider: None,
        }
    }
}
