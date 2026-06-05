use crate::{
    CatalogSourceKind, ExternalModelInput, ExternalProviderInput, ModelCapabilities,
    ModelCatalogReadOutput, ModelCatalogReadReport, ModelCatalogReaderOptions, ModelPricingMicros,
    ProviderAuthKind, ProviderKind, Result, RoutingRole, SkippedModelCatalogEntry, SourceMetadata,
    dx_providers_rkyv_input,
    file_limits::{
        DEFAULT_PROVIDER_ARCHIVE_MAX_BYTES, DEFAULT_PROVIDER_METADATA_SIDECAR_MAX_BYTES,
        ensure_file_with_limit, read_to_string_with_limit,
    },
};
use memmap2::MmapOptions;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

const DEFAULT_SOURCE_ID: &str = "dx-providers-rkyv";
const PROVIDER_METADATA_SCHEMA: &str = "dx.providers.metadata.v1";
const PROVIDER_METADATA_SIDECAR_FILE: &str = "provider-metadata.generated.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidersCatalogReaderOptions {
    pub source_id: String,
    pub source_revision: Option<String>,
    pub generated_unix_ms: Option<u64>,
    pub max_bytes: u64,
    pub metadata_sidecar_path: Option<PathBuf>,
    pub max_metadata_sidecar_bytes: u64,
}

impl ProvidersCatalogReaderOptions {
    pub fn new() -> Self {
        Self {
            source_id: DEFAULT_SOURCE_ID.to_string(),
            source_revision: None,
            generated_unix_ms: None,
            max_bytes: DEFAULT_PROVIDER_ARCHIVE_MAX_BYTES,
            metadata_sidecar_path: None,
            max_metadata_sidecar_bytes: DEFAULT_PROVIDER_METADATA_SIDECAR_MAX_BYTES,
        }
    }

    pub fn with_source_id(mut self, source_id: impl Into<String>) -> Self {
        self.source_id = source_id.into();
        self
    }

    pub fn with_source_revision(mut self, source_revision: impl Into<String>) -> Self {
        self.source_revision = Some(source_revision.into());
        self
    }

    pub fn with_generated_unix_ms(mut self, generated_unix_ms: u64) -> Self {
        self.generated_unix_ms = Some(generated_unix_ms);
        self
    }

    pub fn with_max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = max_bytes.max(1);
        self
    }

    pub fn with_metadata_sidecar_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.metadata_sidecar_path = Some(path.into());
        self
    }

    pub fn with_max_metadata_sidecar_bytes(mut self, max_bytes: u64) -> Self {
        self.max_metadata_sidecar_bytes = max_bytes.max(1);
        self
    }
}

impl Default for ProvidersCatalogReaderOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ModelCatalogReaderOptions> for ProvidersCatalogReaderOptions {
    fn from(options: ModelCatalogReaderOptions) -> Self {
        Self {
            source_id: options.source_id,
            source_revision: options.source_revision,
            generated_unix_ms: options.generated_unix_ms,
            max_bytes: options.max_bytes,
            metadata_sidecar_path: None,
            max_metadata_sidecar_bytes: DEFAULT_PROVIDER_METADATA_SIDECAR_MAX_BYTES,
        }
    }
}

pub fn read_providers_catalog_file(
    path: impl AsRef<Path>,
    options: ProvidersCatalogReaderOptions,
) -> Result<ModelCatalogReadOutput> {
    let path = path.as_ref().to_path_buf();
    let source_available = path.is_file();
    if !source_available {
        return Ok(providers_catalog_output(
            path,
            options,
            source_available,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));
    }

    ensure_file_with_limit(&path, options.max_bytes)?;
    let file = File::open(&path)?;
    // SAFETY: The file is mapped read-only and immediately validated with rkyv
    // before the owned catalog is deserialized from the mapped bytes.
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let archive = rkyv::from_bytes::<ProvidersData>(&mmap)
        .map_err(|error| crate::DxCatalogError::Archive(format!("{error:?}")))?;
    let (mut providers, mut models) = convert_providers_catalog(archive);
    let mut skipped_entries = Vec::new();
    if let Some(metadata) = read_metadata_sidecar(&path, &options, &mut skipped_entries) {
        apply_metadata_sidecar(&metadata, &mut providers, &mut models);
    }

    Ok(providers_catalog_output(
        path,
        options,
        source_available,
        providers,
        models,
        skipped_entries,
    ))
}

fn providers_catalog_output(
    path: PathBuf,
    options: ProvidersCatalogReaderOptions,
    source_available: bool,
    providers: Vec<ExternalProviderInput>,
    models: Vec<ExternalModelInput>,
    skipped_entries: Vec<SkippedModelCatalogEntry>,
) -> ModelCatalogReadOutput {
    let provider_count = providers.len() as u32;
    let model_count = models.len() as u32;
    let metadata = source_metadata(&options, &path, provider_count, model_count);
    let input = dx_providers_rkyv_input(metadata, providers, models);

    ModelCatalogReadOutput {
        input,
        report: ModelCatalogReadReport {
            path: Some(path),
            source_kind: CatalogSourceKind::DxProvidersRkyv,
            source_id: options.source_id,
            source_available,
            provider_count,
            model_count,
            skipped_entries,
        },
    }
}

fn source_metadata(
    options: &ProvidersCatalogReaderOptions,
    path: &Path,
    provider_count: u32,
    model_count: u32,
) -> SourceMetadata {
    let mut metadata = SourceMetadata::new(options.source_id.clone()).with_notes(format!(
        "DX Providers rkyv scan; path={}; providers={provider_count}; models={model_count}",
        path.display()
    ));

    if let Some(source_revision) = &options.source_revision {
        metadata = metadata.with_revision(source_revision.clone());
    }
    if let Some(generated_unix_ms) = options.generated_unix_ms {
        metadata = metadata.with_generated_unix_ms(generated_unix_ms);
    }

    metadata
}

fn convert_providers_catalog(
    catalog: ProvidersData,
) -> (Vec<ExternalProviderInput>, Vec<ExternalModelInput>) {
    let mut providers = Vec::with_capacity(catalog.providers.len());
    let mut models = Vec::new();

    for provider in catalog.providers {
        let provider_id = canonical_provider_id(&provider.id);
        let mut converted_provider = ExternalProviderInput::new(
            provider_id.clone(),
            provider_name(&provider, &provider_id),
            provider_kind(&provider_id, &provider),
        );
        converted_provider.auth = provider_auth(&converted_provider);
        converted_provider.aliases = provider_aliases(&provider.id, &provider_id);
        converted_provider.base_url = non_empty(provider.api_url.clone());
        converted_provider.homepage_url = non_empty(provider.docs_url.clone());
        converted_provider.supports_streaming = provider.supports_chat;
        converted_provider.supports_tools = false;
        converted_provider.supports_free_tier = provider.models.iter().any(model_is_free);
        converted_provider.supports_premium_account =
            provider.models.iter().any(|model| !model_is_free(model));
        converted_provider.is_enabled_by_default = false;
        converted_provider.notes = Some(format!(
            "Imported from DX Providers rkyv source `{}` with {} catalog model(s).",
            provider.source, provider.model_count
        ));

        let provider_capabilities = ProviderModelCapabilities {
            supports_chat: provider.supports_chat,
            supports_embedding: provider.supports_embedding,
            supports_image: provider.supports_image,
            supports_audio: provider.supports_audio,
        };
        for model in provider.models {
            models.push(convert_model(&provider_id, provider_capabilities, model));
        }

        providers.push(converted_provider);
    }

    (providers, models)
}

fn read_metadata_sidecar(
    catalog_path: &Path,
    options: &ProvidersCatalogReaderOptions,
    skipped_entries: &mut Vec<SkippedModelCatalogEntry>,
) -> Option<ProviderMetadataSidecar> {
    let sidecar_path = options
        .metadata_sidecar_path
        .clone()
        .unwrap_or_else(|| catalog_path.with_file_name(PROVIDER_METADATA_SIDECAR_FILE));
    if !sidecar_path.is_file() {
        return None;
    }

    let content = match read_to_string_with_limit(&sidecar_path, options.max_metadata_sidecar_bytes)
    {
        Ok(content) => content,
        Err(error) => {
            skip_metadata_sidecar(skipped_entries, &sidecar_path, error.to_string());
            return None;
        }
    };
    let sidecar: ProviderMetadataSidecar = match serde_json::from_str(&content) {
        Ok(sidecar) => sidecar,
        Err(error) => {
            skip_metadata_sidecar(
                skipped_entries,
                &sidecar_path,
                format!("metadata sidecar JSON parse failed: {error}"),
            );
            return None;
        }
    };

    if sidecar.schema != PROVIDER_METADATA_SCHEMA || sidecar.schema_version != 1 {
        skip_metadata_sidecar(
            skipped_entries,
            &sidecar_path,
            format!(
                "metadata sidecar schema/version mismatch: {} v{}",
                sidecar.schema, sidecar.schema_version
            ),
        );
        return None;
    }
    if sidecar.redaction.secrets_included {
        skip_metadata_sidecar(
            skipped_entries,
            &sidecar_path,
            "metadata sidecar declares that secrets are included".to_string(),
        );
        return None;
    }

    Some(sidecar)
}

fn skip_metadata_sidecar(
    skipped_entries: &mut Vec<SkippedModelCatalogEntry>,
    path: &Path,
    reason: String,
) {
    skipped_entries.push(SkippedModelCatalogEntry {
        location: path.display().to_string(),
        reason,
    });
}

fn apply_metadata_sidecar(
    sidecar: &ProviderMetadataSidecar,
    providers: &mut [ExternalProviderInput],
    models: &mut [ExternalModelInput],
) {
    for provider in providers {
        let Some(metadata) = sidecar.provider_for(provider) else {
            continue;
        };

        if let Some(display_name) = non_empty(metadata.identity.display_name.clone()) {
            provider.display_name = display_name;
        }
        merge_provider_aliases(provider, sidecar, metadata);

        if !metadata.freemium.free_model_ids.is_empty() {
            provider.supports_free_tier = true;
        }
        if matches!(
            metadata.identity.exposure_status.as_deref(),
            Some("implemented" | "verified_working")
        ) {
            provider.is_enabled_by_default = true;
        }
        mark_free_models(provider, metadata, models);
    }
}

fn merge_provider_aliases(
    provider: &mut ExternalProviderInput,
    sidecar: &ProviderMetadataSidecar,
    metadata: &ProviderMetadataRow,
) {
    let mut aliases = BTreeSet::from_iter(provider.aliases.iter().cloned());
    aliases.insert(metadata.identity.canonical_id.clone());
    if let Some(runtime_id) = &metadata.identity.runtime_id {
        aliases.insert(runtime_id.clone());
    }
    aliases.extend(metadata.identity.aliases.iter().cloned());
    aliases.extend(metadata.identity.database_ids.iter().cloned());

    let canonical = slug(&metadata.identity.canonical_id);
    for (alias, canonical_id) in &sidecar.alias_index {
        if slug(canonical_id) == canonical {
            aliases.insert(alias.clone());
        }
    }

    aliases.remove(&provider.id);
    provider.aliases = aliases.into_iter().collect();
}

fn mark_free_models(
    provider: &ExternalProviderInput,
    metadata: &ProviderMetadataRow,
    models: &mut [ExternalModelInput],
) {
    if metadata.freemium.free_model_ids.is_empty() {
        return;
    }

    for model in models
        .iter_mut()
        .filter(|model| model.provider_id == provider.id)
    {
        if metadata
            .freemium
            .free_model_ids
            .iter()
            .any(|free_id| model_matches_free_id(model, &provider.id, free_id))
        {
            model.capabilities.free_tier = true;
            model.free_tier_hint =
                Some("DX Providers metadata lists this provider/model as free tier.".to_string());
        }
    }
}

fn model_matches_free_id(model: &ExternalModelInput, provider_id: &str, free_id: &str) -> bool {
    let direct_id = direct_model_id(provider_id, free_id);
    model.id == free_id
        || model.id == direct_id
        || model
            .aliases
            .iter()
            .any(|alias| alias == free_id || alias == &direct_id)
}

#[derive(Debug, Deserialize)]
struct ProviderMetadataSidecar {
    schema: String,
    schema_version: u16,
    #[serde(default)]
    redaction: ProviderMetadataRedaction,
    #[serde(default)]
    providers: Vec<ProviderMetadataRow>,
    #[serde(default)]
    alias_index: BTreeMap<String, String>,
}

impl ProviderMetadataSidecar {
    fn provider_for(&self, provider: &ExternalProviderInput) -> Option<&ProviderMetadataRow> {
        let mut candidates = BTreeSet::from_iter(std::iter::once(provider.id.as_str()));
        candidates.extend(provider.aliases.iter().map(String::as_str));

        for candidate in candidates {
            let normalized = slug(candidate);
            let canonical_id = self
                .alias_index
                .get(&normalized)
                .map(String::as_str)
                .unwrap_or(candidate);
            let canonical_slug = slug(canonical_id);
            if let Some(metadata) = self
                .providers
                .iter()
                .find(|metadata| slug(&metadata.identity.canonical_id) == canonical_slug)
            {
                return Some(metadata);
            }
        }

        None
    }
}

#[derive(Debug, Default, Deserialize)]
struct ProviderMetadataRedaction {
    #[serde(default)]
    secrets_included: bool,
}

#[derive(Debug, Deserialize)]
struct ProviderMetadataRow {
    identity: ProviderMetadataIdentity,
    freemium: ProviderMetadataFreemium,
}

#[derive(Debug, Deserialize)]
struct ProviderMetadataIdentity {
    canonical_id: String,
    display_name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    database_ids: Vec<String>,
    #[serde(default)]
    runtime_id: Option<String>,
    #[serde(default)]
    exposure_status: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ProviderMetadataFreemium {
    #[serde(default)]
    free_model_ids: Vec<String>,
}

fn convert_model(
    provider_id: &str,
    provider_capabilities: ProviderModelCapabilities,
    model: Model,
) -> ExternalModelInput {
    let model_id = direct_model_id(provider_id, &model.id);
    let mut converted = ExternalModelInput::new(model_id, provider_id, model_name(&model));
    converted.aliases = model_aliases(&model.id, provider_id);
    converted.context_window_tokens = (model.max_tokens > 0).then_some(model.max_tokens);
    converted.max_output_tokens = None;
    converted.pricing = pricing(&model);
    converted.capabilities = model_capabilities(provider_capabilities, &model, converted.pricing);
    converted.recommended_roles = recommended_roles(&converted.capabilities);
    if converted.capabilities.free_tier {
        converted.free_tier_hint = Some("DX Providers catalog lists zero token cost.".to_string());
    } else {
        converted.premium_account_hint =
            Some("Provider account or credits may be required.".to_string());
    }
    converted.notes = Some(format!("DX Providers mode: `{}`.", model.mode));
    converted
}

fn canonical_provider_id(raw_id: &str) -> String {
    match slug(raw_id).as_str() {
        "google-gemini" | "gemini" | "gemini-oauth" | "google-ai" => "google".to_string(),
        "xai" | "x-ai" | "xai-grok" | "grok" => "x_ai".to_string(),
        "bedrock" | "aws-bedrock" | "amazon-bedrock" => "amazon-bedrock".to_string(),
        "open-router" => "openrouter".to_string(),
        other => other.to_string(),
    }
}

fn provider_name(provider: &Provider, provider_id: &str) -> String {
    non_empty(provider.name.clone()).unwrap_or_else(|| title_case(provider_id))
}

fn model_name(model: &Model) -> String {
    non_empty(model.name.clone()).unwrap_or_else(|| title_case(&model.id))
}

fn provider_aliases(raw_id: &str, provider_id: &str) -> Vec<String> {
    let raw = slug(raw_id);
    (raw != provider_id).then_some(raw).into_iter().collect()
}

fn model_aliases(raw_id: &str, provider_id: &str) -> Vec<String> {
    let mut aliases = vec![raw_id.to_string()];
    let direct_id = direct_model_id(provider_id, raw_id);
    if direct_id != raw_id {
        aliases.push(direct_id);
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

fn provider_kind(provider_id: &str, provider: &Provider) -> ProviderKind {
    match provider_id {
        "anthropic" => ProviderKind::Anthropic,
        "google" => ProviderKind::GoogleAi,
        "amazon-bedrock" => ProviderKind::Bedrock,
        "openrouter" => ProviderKind::OpenRouter,
        "ollama" => ProviderKind::OllamaCompatible,
        "openai" | "deepseek" | "mistral" | "x_ai" | "groq" | "cerebras" | "cohere" | "nvidia"
        | "sambanova" | "fireworks" | "together" | "perplexity" | "qwen" | "github-models" => {
            ProviderKind::OpenAiCompatible
        }
        _ if !provider.api_url.trim().is_empty() && provider.supports_chat => {
            ProviderKind::OpenAiCompatible
        }
        _ => ProviderKind::ModelsDev,
    }
}

fn provider_auth(provider: &ExternalProviderInput) -> ProviderAuthKind {
    match provider.kind {
        ProviderKind::ModelsDev | ProviderKind::Unknown => ProviderAuthKind::None,
        ProviderKind::NativeAccount => ProviderAuthKind::NativeAccount,
        ProviderKind::LocalLlamaCpp | ProviderKind::OllamaCompatible => {
            ProviderAuthKind::LocalRuntime
        }
        _ => ProviderAuthKind::ApiKey,
    }
}

fn direct_model_id(provider_id: &str, raw_model_id: &str) -> String {
    if raw_model_id.contains('/') {
        raw_model_id.to_string()
    } else {
        format!("{provider_id}/{raw_model_id}")
    }
}

fn pricing(model: &Model) -> Option<ModelPricingMicros> {
    let input = price_per_token_to_micros_per_million(model.input_cost);
    let output = price_per_token_to_micros_per_million(model.output_cost);
    (input.is_some() || output.is_some()).then_some(ModelPricingMicros {
        input_per_million_tokens: input,
        output_per_million_tokens: output,
    })
}

fn price_per_token_to_micros_per_million(value: f64) -> Option<u64> {
    value
        .is_finite()
        .then(|| (value.max(0.0) * 1_000_000_000_000.0).round() as u64)
}

#[derive(Debug, Clone, Copy)]
struct ProviderModelCapabilities {
    supports_chat: bool,
    supports_embedding: bool,
    supports_image: bool,
    supports_audio: bool,
}

fn model_capabilities(
    provider: ProviderModelCapabilities,
    model: &Model,
    pricing: Option<ModelPricingMicros>,
) -> ModelCapabilities {
    let haystack = format!(
        "{} {} {}",
        model.id.to_ascii_lowercase(),
        model.name.to_ascii_lowercase(),
        model.mode.to_ascii_lowercase()
    );
    let is_embedding = provider.supports_embedding || haystack.contains("embedding");

    ModelCapabilities {
        chat: provider.supports_chat && !is_embedding,
        tools: false,
        vision: provider.supports_image
            || haystack.contains("vision")
            || haystack.contains("image"),
        audio: provider.supports_audio || haystack.contains("audio"),
        video: haystack.contains("video"),
        embeddings: is_embedding,
        coding: contains_any(&haystack, &["code", "coder", "coding", "devstral"]),
        reasoning: contains_any(
            &haystack,
            &[
                "reasoning",
                "reasoner",
                "thinking",
                " o1",
                " o3",
                " o4",
                " r1",
            ],
        ),
        local_runtime: false,
        streaming: provider.supports_chat,
        free_tier: pricing.is_some_and(|pricing| {
            matches!(pricing.input_per_million_tokens, Some(0))
                && matches!(pricing.output_per_million_tokens, Some(0))
        }),
        premium_account: true,
    }
}

fn recommended_roles(capabilities: &ModelCapabilities) -> Vec<RoutingRole> {
    let mut roles = Vec::new();
    if capabilities.tools {
        roles.push(RoutingRole::ToolAgent);
    }
    if capabilities.coding {
        roles.push(RoutingRole::Coding);
    }
    if capabilities.reasoning {
        roles.push(RoutingRole::Reasoning);
    }
    if capabilities.vision {
        roles.push(RoutingRole::Vision);
    }
    if capabilities.audio {
        roles.push(RoutingRole::Audio);
    }
    if capabilities.embeddings {
        roles.push(RoutingRole::Embeddings);
    }
    if roles.is_empty() {
        roles.push(RoutingRole::Helper);
    }
    roles
}

fn model_is_free(model: &Model) -> bool {
    model.input_cost == 0.0 && model.output_cost == 0.0
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn title_case(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .split(['-', '_', '/', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slug(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut previous_dash = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "provider".to_string()
    } else {
        slug
    }
}

#[derive(Archive, RkyvDeserialize, RkyvSerialize, Debug)]
#[archive(check_bytes)]
struct ProvidersData {
    version: String,
    generated_at: String,
    total_providers: usize,
    total_models: usize,
    providers: Vec<Provider>,
}

#[derive(Archive, RkyvDeserialize, RkyvSerialize, Debug)]
#[archive(check_bytes)]
struct Provider {
    id: String,
    name: String,
    source: String,
    model_count: usize,
    supports_chat: bool,
    supports_embedding: bool,
    supports_image: bool,
    supports_audio: bool,
    api_url: String,
    docs_url: String,
    models: Vec<Model>,
}

#[derive(Archive, RkyvDeserialize, RkyvSerialize, Debug)]
#[archive(check_bytes)]
struct Model {
    id: String,
    name: String,
    mode: String,
    max_tokens: u32,
    input_cost: f64,
    output_cost: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CatalogSourceKind, ProviderKind};
    use rkyv::{
        Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
        ser::{Serializer, serializers::AllocSerializer},
    };
    use std::{fs, path::Path};

    #[test]
    fn reads_dx_providers_rkyv_catalog_into_catalog_input() {
        let path = unique_fixture_path("dx-providers-catalog.rkyv");
        write_fixture_catalog(&path);

        let output = read_providers_catalog_file(
            &path,
            ProvidersCatalogReaderOptions::new().with_source_id("dx-providers-test"),
        )
        .expect("providers catalog should load");

        assert_eq!(
            output.report.source_kind,
            CatalogSourceKind::DxProvidersRkyv
        );
        assert_eq!(output.report.provider_count, 3);
        assert_eq!(output.report.model_count, 4);
        assert_eq!(output.input.providers.len(), 3);
        assert_eq!(output.input.models.len(), 4);
        assert_eq!(output.input.providers[0].id, "deepseek");
        assert_eq!(
            output.input.providers[0].kind,
            ProviderKind::OpenAiCompatible
        );
        assert_eq!(output.input.models[0].id, "deepseek/deepseek-chat");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn reads_copied_g_drive_dx_providers_catalog_when_available() {
        let path = std::path::PathBuf::from(r"G:\Dx\providers\data\providers.rkyv");
        if !path.is_file() {
            return;
        }

        let output = read_providers_catalog_file(
            &path,
            ProvidersCatalogReaderOptions::new().with_source_id("copied-g-drive-dx-providers"),
        )
        .expect("copied G-drive providers catalog should load");

        assert_eq!(
            output.report.source_kind,
            CatalogSourceKind::DxProvidersRkyv
        );
        assert!(output.report.provider_count >= 100);
        assert!(output.report.model_count >= 1_000);
        assert_eq!(
            output.report.provider_count as usize,
            output.input.providers.len()
        );
        assert_eq!(
            output.report.model_count as usize,
            output.input.models.len()
        );
    }

    #[test]
    fn reads_copied_g_drive_dx_providers_catalog_snapshot_counts_when_available() {
        let path = std::path::PathBuf::from(r"G:\Dx\providers\data\providers.rkyv");
        if !path.is_file() {
            return;
        }

        let output = read_providers_catalog_file(
            &path,
            ProvidersCatalogReaderOptions::new().with_source_id("copied-g-drive-dx-providers"),
        )
        .expect("copied G-drive providers catalog should load");

        assert_eq!(output.report.provider_count, 184);
        assert_eq!(output.report.model_count, 6_557);
        assert_eq!(output.input.providers.len(), 184);
        assert_eq!(output.input.models.len(), 6_557);
    }

    #[test]
    fn providers_catalog_file_read_rejects_oversized_archive_before_mmap() {
        let path = unique_fixture_path("oversized-providers-catalog.rkyv");
        fs::write(&path, [0_u8, 1]).expect("fixture should write");

        let error = read_providers_catalog_file(
            &path,
            ProvidersCatalogReaderOptions::new().with_max_bytes(1),
        )
        .expect_err("oversized providers archive should fail before mmap and rkyv validation");

        assert!(
            matches!(error, crate::DxCatalogError::FileTooLarge { .. }),
            "unexpected error: {error}"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn providers_metadata_sidecar_overlays_provider_aliases_and_free_models() {
        let path = unique_fixture_path("dx-providers-catalog.rkyv");
        write_fixture_catalog(&path);
        let sidecar_path = path.with_extension("metadata.json");
        write_fixture_metadata_sidecar(
            &sidecar_path,
            "deepseek",
            "DeepSeek Verified",
            &["deepseek-platform"],
            &["deepseek-chat"],
        );

        let output = read_providers_catalog_file(
            &path,
            ProvidersCatalogReaderOptions::new()
                .with_source_id("dx-providers-test")
                .with_metadata_sidecar_path(&sidecar_path),
        )
        .expect("providers catalog should load with metadata sidecar");
        let provider = output
            .input
            .providers
            .iter()
            .find(|provider| provider.id == "deepseek")
            .expect("deepseek provider");
        let free_model = output
            .input
            .models
            .iter()
            .find(|model| model.id == "deepseek/deepseek-chat")
            .expect("deepseek chat model");

        assert_eq!(provider.display_name, "DeepSeek Verified");
        assert_eq!(
            provider.base_url.as_deref(),
            Some("https://api.deepseek.com/v1")
        );
        assert!(provider.aliases.contains(&"deepseek-platform".to_string()));
        assert!(provider.supports_free_tier);
        assert!(free_model.capabilities.free_tier);
        assert_eq!(
            free_model.free_tier_hint.as_deref(),
            Some("DX Providers metadata lists this provider/model as free tier.")
        );

        let _ = fs::remove_file(sidecar_path);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn providers_metadata_sidecar_uses_alias_index_for_canonical_id_mismatches() {
        let path = unique_fixture_path("dx-providers-catalog.rkyv");
        write_fixture_catalog(&path);
        let sidecar_path = path.with_extension("metadata.json");
        write_fixture_metadata_sidecar(
            &sidecar_path,
            "google-gemini",
            "Google AI Studio",
            &["google-ai-studio"],
            &[],
        );

        let output = read_providers_catalog_file(
            &path,
            ProvidersCatalogReaderOptions::new()
                .with_source_id("dx-providers-test")
                .with_metadata_sidecar_path(&sidecar_path),
        )
        .expect("providers catalog should load with metadata sidecar");
        let provider = output
            .input
            .providers
            .iter()
            .find(|provider| provider.id == "google")
            .expect("google provider");

        assert_eq!(provider.display_name, "Google AI Studio");
        assert!(provider.aliases.contains(&"google-ai-studio".to_string()));
        assert!(provider.aliases.contains(&"google-gemini".to_string()));

        let _ = fs::remove_file(sidecar_path);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn wrong_schema_provider_metadata_sidecar_does_not_block_rkyv_load() {
        let path = unique_fixture_path("dx-providers-catalog.rkyv");
        write_fixture_catalog(&path);
        let sidecar_path = path.with_extension("metadata.json");
        fs::write(
            &sidecar_path,
            r#"{"schema":"wrong.schema","schema_version":1}"#,
        )
        .expect("metadata sidecar should write");

        let output = read_providers_catalog_file(
            &path,
            ProvidersCatalogReaderOptions::new()
                .with_source_id("dx-providers-test")
                .with_metadata_sidecar_path(&sidecar_path),
        )
        .expect("providers catalog should load without applying bad metadata sidecar");
        let provider = output
            .input
            .providers
            .iter()
            .find(|provider| provider.id == "deepseek")
            .expect("deepseek provider");

        assert_eq!(provider.display_name, "DeepSeek");
        assert!(!provider.aliases.contains(&"deepseek-platform".to_string()));

        let _ = fs::remove_file(sidecar_path);
        let _ = fs::remove_file(path);
    }

    fn unique_fixture_path(file_name: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{nonce}-{file_name}", std::process::id()))
    }

    fn write_fixture_catalog(path: &Path) {
        let catalog = ProvidersData {
            version: "test-catalog".to_string(),
            generated_at: "2026-06-04T00:00:00Z".to_string(),
            total_providers: 3,
            total_models: 4,
            providers: vec![
                Provider {
                    id: "deepseek".to_string(),
                    name: "DeepSeek".to_string(),
                    source: "litellm+models.dev".to_string(),
                    model_count: 2,
                    supports_chat: true,
                    supports_embedding: false,
                    supports_image: false,
                    supports_audio: false,
                    api_url: "https://api.deepseek.com/v1".to_string(),
                    docs_url: "https://api-docs.deepseek.com".to_string(),
                    models: vec![
                        Model {
                            id: "deepseek-chat".to_string(),
                            name: "DeepSeek Chat".to_string(),
                            mode: "chat".to_string(),
                            max_tokens: 128_000,
                            input_cost: 0.27,
                            output_cost: 1.10,
                        },
                        Model {
                            id: "deepseek-reasoner".to_string(),
                            name: "DeepSeek Reasoner".to_string(),
                            mode: "chat".to_string(),
                            max_tokens: 64_000,
                            input_cost: 0.55,
                            output_cost: 2.19,
                        },
                    ],
                },
                Provider {
                    id: "groq".to_string(),
                    name: "Groq".to_string(),
                    source: "litellm".to_string(),
                    model_count: 1,
                    supports_chat: true,
                    supports_embedding: false,
                    supports_image: false,
                    supports_audio: false,
                    api_url: "https://api.groq.com/openai/v1".to_string(),
                    docs_url: "https://console.groq.com/docs".to_string(),
                    models: vec![Model {
                        id: "llama-3.3-70b-versatile".to_string(),
                        name: "Llama 3.3 70B Versatile".to_string(),
                        mode: "chat".to_string(),
                        max_tokens: 131_072,
                        input_cost: 0.59,
                        output_cost: 0.79,
                    }],
                },
                Provider {
                    id: "google-gemini".to_string(),
                    name: "Gemini".to_string(),
                    source: "models.dev".to_string(),
                    model_count: 1,
                    supports_chat: true,
                    supports_embedding: false,
                    supports_image: true,
                    supports_audio: false,
                    api_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                    docs_url: "https://ai.google.dev".to_string(),
                    models: vec![Model {
                        id: "gemini-2.5-pro".to_string(),
                        name: "Gemini 2.5 Pro".to_string(),
                        mode: "chat".to_string(),
                        max_tokens: 1_048_576,
                        input_cost: 1.25,
                        output_cost: 10.00,
                    }],
                },
            ],
        };

        let mut serializer = AllocSerializer::<4096>::default();
        serializer.serialize_value(&catalog).unwrap();
        fs::write(path, serializer.into_serializer().into_inner()).unwrap();
    }

    fn write_fixture_metadata_sidecar(
        path: &Path,
        canonical_id: &str,
        display_name: &str,
        aliases: &[&str],
        free_model_ids: &[&str],
    ) {
        let mut alias_index = serde_json::Map::new();
        alias_index.insert(
            slug(canonical_id),
            serde_json::Value::String(canonical_id.to_string()),
        );
        for alias in aliases {
            alias_index.insert(
                slug(alias),
                serde_json::Value::String(canonical_id.to_string()),
            );
        }

        let content = serde_json::json!({
            "schema": "dx.providers.metadata.v1",
            "schema_version": 1,
            "source": {
                "repo": r"G:\Dx\providers",
                "commit": "test",
                "generated_at": "2026-06-05T00:00:00Z"
            },
            "summary": {
                "provider_count": 1,
                "alias_count": alias_index.len(),
                "content_sha256": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            },
            "redaction": {
                "secrets_included": false,
                "statement": "environment variable names only"
            },
            "providers": [{
                "identity": {
                    "canonical_id": canonical_id,
                    "display_name": display_name,
                    "aliases": aliases,
                    "database_ids": [],
                    "runtime_id": canonical_id,
                    "exposure_status": "verified_working"
                },
                "freemium": {
                    "access": "api_key",
                    "auth": ["api_key"],
                    "env_vars": ["TEST_API_KEY"],
                    "note": "fixture free tier",
                    "free_model_ids": free_model_ids
                }
            }],
            "alias_index": alias_index
        });

        fs::write(path, serde_json::to_string_pretty(&content).unwrap()).unwrap();
    }

    #[derive(Archive, RkyvDeserialize, RkyvSerialize, Debug)]
    #[archive(check_bytes)]
    struct ProvidersData {
        version: String,
        generated_at: String,
        total_providers: usize,
        total_models: usize,
        providers: Vec<Provider>,
    }

    #[derive(Archive, RkyvDeserialize, RkyvSerialize, Debug)]
    #[archive(check_bytes)]
    struct Provider {
        id: String,
        name: String,
        source: String,
        model_count: usize,
        supports_chat: bool,
        supports_embedding: bool,
        supports_image: bool,
        supports_audio: bool,
        api_url: String,
        docs_url: String,
        models: Vec<Model>,
    }

    #[derive(Archive, RkyvDeserialize, RkyvSerialize, Debug)]
    #[archive(check_bytes)]
    struct Model {
        id: String,
        name: String,
        mode: String,
        max_tokens: u32,
        input_cost: f64,
        output_cost: f64,
    }
}
