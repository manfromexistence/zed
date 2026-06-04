use dx_media::DownloadUrlKind;
use dx_media::{DxMedia, MediaAsset, MediaType, SearchMode, SearchQuery, SearchResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelMediaKindFilter {
    All,
    Images,
    Videos,
    Audio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelMediaKind {
    Image,
    Video,
    Audio,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PanelMediaSearchRequest {
    query: String,
    filter: PanelMediaKindFilter,
    count: usize,
    page: usize,
}

impl PanelMediaSearchRequest {
    pub(crate) fn new(query: String, filter: PanelMediaKindFilter, count: usize) -> Self {
        Self {
            query,
            filter,
            count,
            page: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PanelMediaAsset {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) provider: String,
    pub(crate) url: String,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) kind: PanelMediaKind,
    pub(crate) license: String,
    pub(crate) tags: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PanelMediaSearchResult {
    pub(crate) assets: Vec<PanelMediaAsset>,
    pub(crate) total_count: usize,
    pub(crate) providers_searched: Vec<String>,
    pub(crate) provider_errors: Vec<(String, String)>,
}

pub(crate) async fn fetch_panel_media(
    request: PanelMediaSearchRequest,
) -> anyhow::Result<PanelMediaSearchResult> {
    let queries = build_search_queries(&request);
    let dx_media = DxMedia::new()?;
    let mut results = Vec::with_capacity(queries.len());
    let mut provider_errors = Vec::new();

    for query in queries {
        match dx_media.search_query(&query).await {
            Ok(result) => results.push(result),
            Err(error) => provider_errors.push((search_query_label(&query), error.to_string())),
        }
    }

    if results.is_empty() && !provider_errors.is_empty() {
        let errors = provider_errors
            .iter()
            .map(|(provider, error)| format!("{provider}: {error}"))
            .collect::<Vec<_>>();
        anyhow::bail!(errors.join("; "));
    }

    Ok(from_search_results(results, provider_errors))
}

fn build_search_queries(request: &PanelMediaSearchRequest) -> Vec<SearchQuery> {
    let media_types = media_types_for_filter(request.filter);
    if media_types.is_empty() {
        return vec![build_search_query(request, None)];
    }

    media_types
        .iter()
        .map(|media_type| build_search_query(request, Some(*media_type)))
        .collect()
}

fn build_search_query(
    request: &PanelMediaSearchRequest,
    media_type: Option<MediaType>,
) -> SearchQuery {
    let mut query = SearchQuery::new(request.query.clone())
        .count(request.count)
        .page(request.page)
        .mode(SearchMode::Quality);

    if let Some(media_type) = media_type {
        query = query.media_type(media_type);
    }

    query
}

fn from_search_results(
    results: Vec<SearchResult>,
    mut provider_errors: Vec<(String, String)>,
) -> PanelMediaSearchResult {
    let mut assets = Vec::new();
    let mut total_count = 0;
    let mut providers_searched = Vec::new();

    for result in results {
        total_count += result.total_count;
        providers_searched.extend(result.providers_searched);
        provider_errors.extend(result.provider_errors);
        assets.extend(
            result
                .assets
                .into_iter()
                .filter_map(panel_asset_from_media_asset),
        );
    }

    PanelMediaSearchResult {
        assets,
        total_count,
        providers_searched,
        provider_errors,
    }
}

fn panel_asset_from_media_asset(asset: MediaAsset) -> Option<PanelMediaAsset> {
    let kind = panel_kind_for_media_type(asset.media_type)?;
    if !is_panel_renderable_download(&asset) {
        return None;
    }

    Some(PanelMediaAsset {
        id: format!("{}:{}", asset.provider, asset.id),
        label: clean_panel_label(&asset.title),
        provider: clean_panel_label(&asset.provider),
        url: asset.download_url,
        thumbnail_url: asset.preview_url,
        kind,
        license: clean_panel_label(asset.license.as_str()),
        tags: asset.tags.join(", "),
    })
}

fn media_types_for_filter(filter: PanelMediaKindFilter) -> &'static [MediaType] {
    match filter {
        PanelMediaKindFilter::All => &[],
        PanelMediaKindFilter::Images => &[MediaType::Image, MediaType::Gif, MediaType::Vector],
        PanelMediaKindFilter::Videos => &[MediaType::Video],
        PanelMediaKindFilter::Audio => &[MediaType::Audio],
    }
}

fn panel_kind_for_media_type(media_type: MediaType) -> Option<PanelMediaKind> {
    match media_type {
        MediaType::Image | MediaType::Gif | MediaType::Vector => Some(PanelMediaKind::Image),
        MediaType::Video => Some(PanelMediaKind::Video),
        MediaType::Audio => Some(PanelMediaKind::Audio),
        _ => None,
    }
}

fn is_panel_renderable_download(asset: &MediaAsset) -> bool {
    match asset.download_url_kind {
        DownloadUrlKind::DirectFile | DownloadUrlKind::PreviewDerivative => true,
        DownloadUrlKind::Unknown => has_media_type_evidence(asset),
        DownloadUrlKind::AssetManifest | DownloadUrlKind::LandingPage => false,
    }
}

fn has_media_type_evidence(asset: &MediaAsset) -> bool {
    asset
        .mime_type
        .as_deref()
        .is_some_and(|mime_type| asset.media_type.matches_mime(mime_type))
        || download_url_extension(asset.download_url.as_str())
            .is_some_and(|extension| asset.media_type.matches_extension(extension))
}

fn download_url_extension(url: &str) -> Option<&str> {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim_end_matches('/');
    let extension = path.rsplit_once('.')?.1;
    (!extension.is_empty() && !extension.contains('/')).then_some(extension)
}

fn search_query_label(query: &SearchQuery) -> String {
    query
        .media_type
        .map(|media_type| format!("dx-media {}", media_type.as_str()))
        .unwrap_or_else(|| "dx-media".to_string())
}

fn clean_panel_label(value: &str) -> String {
    let mut label = String::with_capacity(value.len().min(160));
    for word in value.split_whitespace() {
        if !label.is_empty() {
            label.push(' ');
        }
        label.push_str(word);
    }

    if label.is_empty() {
        "Remote media".to_string()
    } else {
        label
    }
}
