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
    let query = build_search_query(&request);
    let result = DxMedia::new()?.search_query(&query).await?;

    Ok(from_search_result(result))
}

fn build_search_query(request: &PanelMediaSearchRequest) -> SearchQuery {
    let mut query = SearchQuery::new(request.query.clone())
        .count(request.count)
        .page(request.page)
        .mode(SearchMode::Quality);

    if let Some(media_type) = media_type_for_filter(request.filter) {
        query = query.media_type(media_type);
    }

    query
}

fn from_search_result(result: SearchResult) -> PanelMediaSearchResult {
    let assets = result
        .assets
        .into_iter()
        .filter_map(panel_asset_from_media_asset)
        .collect();

    PanelMediaSearchResult {
        assets,
        total_count: result.total_count,
        providers_searched: result.providers_searched,
        provider_errors: result.provider_errors,
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

fn media_type_for_filter(filter: PanelMediaKindFilter) -> Option<MediaType> {
    match filter {
        PanelMediaKindFilter::All => None,
        PanelMediaKindFilter::Images => Some(MediaType::Image),
        PanelMediaKindFilter::Videos => Some(MediaType::Video),
        PanelMediaKindFilter::Audio => Some(MediaType::Audio),
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
    !matches!(
        asset.download_url_kind,
        DownloadUrlKind::AssetManifest | DownloadUrlKind::LandingPage
    )
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
