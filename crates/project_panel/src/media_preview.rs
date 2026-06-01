use std::path::{Path, PathBuf};

use gpui::{
    FontWeight, Hsla, ObjectFit, SharedString, StatefulInteractiveElement, hsla, img,
    linear_color_stop, linear_gradient,
};
use project::Entry;
use ui::{ContextMenu, IconButtonShape, PopoverMenu, Tooltip, prelude::*};

pub(crate) const MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN: usize = 512;
pub(crate) const MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS: usize = 12;
pub(crate) const MAX_PROJECT_PANEL_MEDIA_INLINE_CARDS: usize = 4;
pub(crate) const PROJECT_PANEL_MEDIA_GALLERY_COLUMNS: u16 = 3;

const PROJECT_PANEL_MEDIA_CARD_WIDTH: f32 = 30.;
const PROJECT_PANEL_AUDIO_CARD_WIDTH: f32 = 72.;
const PROJECT_PANEL_MEDIA_CARD_HEIGHT: f32 = 20.;
const PROJECT_PANEL_MEDIA_GALLERY_CARD_WIDTH: f32 = 86.;
const PROJECT_PANEL_MEDIA_GALLERY_CARD_HEIGHT: f32 = 64.;

const IMAGE_MEDIA_EXTENSIONS: &[&str] = &[
    "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "tif", "tiff", "webp",
];
const VIDEO_MEDIA_EXTENSIONS: &[&str] = &["avi", "m4v", "mkv", "mov", "mp4", "mpeg", "mpg", "webm"];
const AUDIO_MEDIA_EXTENSIONS: &[&str] = &["aac", "flac", "m4a", "mp3", "ogg", "opus", "wav"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MediaPreviewKind {
    Image,
    Video,
    Audio,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MediaPreviewItem {
    pub(crate) kind: MediaPreviewKind,
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) absolute_path: PathBuf,
    pub(crate) video_frame_path: Option<PathBuf>,
    pub(crate) audio_duration_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FolderMediaPreview {
    pub(crate) image_count: usize,
    pub(crate) video_count: usize,
    pub(crate) audio_count: usize,
    pub(crate) total_count: usize,
    pub(crate) scanned_cap_hit: bool,
    pub(crate) items: Vec<MediaPreviewItem>,
}

pub(crate) fn build_folder_media_preview<'a>(
    parent_abs_path: &Path,
    children: impl Iterator<Item = &'a Entry>,
) -> Option<FolderMediaPreview> {
    let mut image_count = 0;
    let mut video_count = 0;
    let mut audio_count = 0;
    let mut scanned_count = 0;
    let mut media_scan_was_capped = false;
    let mut items = Vec::new();
    let mut image_frame_candidates = Vec::new();

    for child in children.take(MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN + 1) {
        if scanned_count >= MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN {
            media_scan_was_capped = true;
            break;
        }

        scanned_count += 1;

        if !child.is_file() {
            continue;
        }

        let absolute_path = child_absolute_path(parent_abs_path, child);
        let Some(kind) = media_preview_kind_for_path(&absolute_path) else {
            continue;
        };

        match kind {
            MediaPreviewKind::Image => {
                image_count += 1;
                if let Some(stem) = media_stem_key(&absolute_path) {
                    image_frame_candidates.push((stem, absolute_path.clone()));
                }
            }
            MediaPreviewKind::Video => video_count += 1,
            MediaPreviewKind::Audio => audio_count += 1,
        }

        if items.len() < MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS {
            let name = child
                .path
                .file_name()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| absolute_path.display().to_string());
            items.push(MediaPreviewItem {
                kind,
                name,
                size: child.size,
                absolute_path,
                video_frame_path: None,
                audio_duration_label: (kind == MediaPreviewKind::Audio)
                    .then(|| "Duration unavailable".to_string()),
            });
        }
    }

    for item in &mut items {
        if item.kind == MediaPreviewKind::Video {
            item.video_frame_path =
                video_preview_frame_path(&item.absolute_path, &image_frame_candidates);
        }
    }

    let total_count = image_count + video_count + audio_count;
    (total_count > 0).then_some(FolderMediaPreview {
        image_count,
        video_count,
        audio_count,
        total_count,
        scanned_cap_hit: media_scan_was_capped,
        items,
    })
}

pub(crate) fn render_folder_media_preview(
    preview: &FolderMediaPreview,
    cx: &mut App,
) -> AnyElement {
    let summary = media_preview_summary(preview);
    let tooltip_summary = summary.clone();
    let tooltip_id = SharedString::from(format!("project-panel-media-preview-{summary}"));
    let gallery_preview = preview.clone();
    let gallery_id = format!(
        "project-panel-media-gallery-{:016x}",
        stable_text_hash(&summary)
    );
    let cards = preview
        .items
        .iter()
        .take(MAX_PROJECT_PANEL_MEDIA_INLINE_CARDS)
        .map(|item| render_media_preview_card(item, cx))
        .collect::<Vec<_>>();

    h_flex()
        .id(tooltip_id)
        .h_6()
        .max_w(px(184.))
        .gap_0p5()
        .overflow_hidden()
        .block_mouse_except_scroll()
        .tooltip(move |_window, cx| Tooltip::with_meta(tooltip_summary.clone(), None, "Media", cx))
        .children(cards)
        .child(
            Label::new(summary)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .single_line()
                .truncate(),
        )
        .child(
            PopoverMenu::new(gallery_id)
                .trigger(
                    IconButton::new("project-panel-media-gallery-trigger", IconName::Blocks)
                        .shape(IconButtonShape::Square)
                        .style(ButtonStyle::Subtle)
                        .icon_size(IconSize::XSmall)
                        .tooltip(Tooltip::text("Open media grid")),
                )
                .anchor(gpui::Anchor::TopRight)
                .attach(gpui::Anchor::BottomRight)
                .menu(move |window, cx| {
                    let gallery_preview = gallery_preview.clone();
                    Some(ContextMenu::build(window, cx, move |menu, _window, _cx| {
                        menu.custom_row(move |_window, cx| {
                            render_folder_media_gallery(&gallery_preview, cx)
                        })
                    }))
                }),
        )
        .into_any_element()
}

pub(crate) fn render_media_preview_card(item: &MediaPreviewItem, cx: &mut App) -> AnyElement {
    let colors = cx.theme().colors();
    let tooltip_title = item.name.clone();
    let tooltip_meta = media_preview_card_tooltip_meta(item);
    let card = match item.kind {
        MediaPreviewKind::Image => div()
            .w(px(PROJECT_PANEL_MEDIA_CARD_WIDTH))
            .h(px(PROJECT_PANEL_MEDIA_CARD_HEIGHT))
            .rounded_sm()
            .overflow_hidden()
            .border_1()
            .border_color(colors.border_variant)
            .child(
                img(item.absolute_path.clone())
                    .size_full()
                    .object_fit(ObjectFit::Cover),
            ),
        MediaPreviewKind::Video => {
            if let Some(frame_path) = item.video_frame_path.as_ref() {
                div()
                    .relative()
                    .w(px(PROJECT_PANEL_MEDIA_CARD_WIDTH))
                    .h(px(PROJECT_PANEL_MEDIA_CARD_HEIGHT))
                    .rounded_sm()
                    .overflow_hidden()
                    .border_1()
                    .border_color(colors.border_variant)
                    .child(
                        img(frame_path.clone())
                            .size_full()
                            .object_fit(ObjectFit::Cover),
                    )
                    .child(
                        div()
                            .absolute()
                            .right_0()
                            .bottom_0()
                            .rounded_full()
                            .bg(colors.editor_background.opacity(0.72))
                            .child(
                                Icon::new(IconName::PlayOutlined)
                                    .size(IconSize::XSmall)
                                    .color(Color::Accent),
                            ),
                    )
            } else {
                div()
                    .w(px(PROJECT_PANEL_MEDIA_CARD_WIDTH))
                    .h(px(PROJECT_PANEL_MEDIA_CARD_HEIGHT))
                    .rounded_sm()
                    .border_1()
                    .border_color(colors.border_variant)
                    .bg(colors.elevated_surface_background)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new(IconName::PlayOutlined)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
            }
        }
        MediaPreviewKind::Audio => div()
            .w(px(PROJECT_PANEL_AUDIO_CARD_WIDTH))
            .h(px(PROJECT_PANEL_MEDIA_CARD_HEIGHT))
            .rounded_sm()
            .border_1()
            .border_color(colors.border_variant)
            .bg(audio_gradient_background(&item.name))
            .overflow_hidden()
            .flex()
            .items_center()
            .px_1()
            .justify_center()
            .child(
                Label::new(item.name.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Default)
                    .buffer_font(cx)
                    .single_line()
                    .truncate(),
            ),
    };

    let card_id = SharedString::from(format!(
        "project-panel-media-card-{:?}-{:016x}",
        item.kind,
        stable_text_hash(&item.name)
    ));

    card.id(card_id)
        .tooltip(move |_window, cx| {
            Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
        })
        .into_any_element()
}

pub(crate) fn render_folder_media_gallery(
    preview: &FolderMediaPreview,
    cx: &mut App,
) -> AnyElement {
    let summary = media_preview_summary(preview);
    let visible_count = preview
        .items
        .len()
        .min(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS);
    let gallery_cards = preview
        .items
        .iter()
        .take(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS)
        .map(|item| render_media_gallery_card(item, cx))
        .collect::<Vec<_>>();

    v_flex()
        .id(SharedString::from(format!(
            "project-panel-media-gallery-content-{:016x}",
            stable_text_hash(&summary)
        )))
        .min_w(px(294.))
        .max_w(px(312.))
        .gap_2()
        .p_2()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    Label::new("Media")
                        .size(LabelSize::Small)
                        .weight(FontWeight::SEMIBOLD),
                )
                .child(
                    Label::new(format!("{visible_count} shown / {summary}"))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .single_line()
                        .truncate(),
                ),
        )
        .child(
            div()
                .grid()
                .grid_cols(PROJECT_PANEL_MEDIA_GALLERY_COLUMNS)
                .gap_1p5()
                .children(gallery_cards),
        )
        .into_any_element()
}

fn render_media_gallery_card(item: &MediaPreviewItem, cx: &mut App) -> AnyElement {
    let colors = cx.theme().colors();
    let tooltip_title = item.name.clone();
    let tooltip_meta = media_preview_card_tooltip_meta(item);
    let media = match item.kind {
        MediaPreviewKind::Image => div()
            .relative()
            .w_full()
            .h(px(PROJECT_PANEL_MEDIA_GALLERY_CARD_HEIGHT))
            .rounded_sm()
            .overflow_hidden()
            .child(
                img(item.absolute_path.clone())
                    .size_full()
                    .object_fit(ObjectFit::Cover),
            ),
        MediaPreviewKind::Video => {
            let base = div()
                .relative()
                .w_full()
                .h(px(PROJECT_PANEL_MEDIA_GALLERY_CARD_HEIGHT))
                .rounded_sm()
                .overflow_hidden()
                .bg(colors.elevated_surface_background);

            let base = if let Some(frame_path) = item.video_frame_path.as_ref() {
                base.child(
                    img(frame_path.clone())
                        .size_full()
                        .object_fit(ObjectFit::Cover),
                )
            } else {
                base.flex().items_center().justify_center().child(
                    Icon::new(IconName::PlayOutlined)
                        .size(IconSize::Large)
                        .color(Color::Muted),
                )
            };

            base.child(
                div()
                    .absolute()
                    .right_1()
                    .bottom_1()
                    .rounded_full()
                    .bg(colors.editor_background.opacity(0.72))
                    .p_0p5()
                    .child(
                        Icon::new(IconName::PlayOutlined)
                            .size(IconSize::XSmall)
                            .color(Color::Accent),
                    ),
            )
        }
        MediaPreviewKind::Audio => div()
            .relative()
            .w_full()
            .h(px(PROJECT_PANEL_MEDIA_GALLERY_CARD_HEIGHT))
            .rounded_sm()
            .overflow_hidden()
            .bg(audio_gradient_background(&item.name))
            .flex()
            .items_center()
            .justify_center()
            .px_1()
            .child(
                Label::new(item.name.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Default)
                    .buffer_font(cx)
                    .single_line()
                    .truncate(),
            )
            .child(
                div()
                    .absolute()
                    .left_1()
                    .bottom_1()
                    .rounded_full()
                    .bg(colors.editor_background.opacity(0.55))
                    .p_0p5()
                    .child(
                        Icon::new(IconName::AudioOn)
                            .size(IconSize::XSmall)
                            .color(Color::Muted),
                    ),
            ),
    };

    div()
        .id(SharedString::from(format!(
            "project-panel-media-gallery-card-{:?}-{:016x}",
            item.kind,
            stable_text_hash(&item.name)
        )))
        .min_w(px(0.))
        .w(px(PROJECT_PANEL_MEDIA_GALLERY_CARD_WIDTH))
        .v_flex()
        .gap_1()
        .p_1()
        .rounded_sm()
        .border_1()
        .border_color(colors.border_variant)
        .bg(colors.element_background)
        .hover(|style| style.bg(colors.element_hover))
        .tooltip(move |_window, cx| {
            Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
        })
        .child(media)
        .child(
            Label::new(item.name.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .single_line()
                .truncate(),
        )
        .into_any_element()
}

fn media_preview_card_tooltip_meta(item: &MediaPreviewItem) -> String {
    let size_label = media_size_label(item.size);
    match item.kind {
        MediaPreviewKind::Image => format!("Image / {size_label}"),
        MediaPreviewKind::Video => {
            let duration = "Duration unavailable";
            if item.video_frame_path.is_some() {
                format!("Video frame preview / {duration} / {size_label}")
            } else {
                format!("Video preview unavailable / {duration} / {size_label}")
            }
        }
        MediaPreviewKind::Audio => format!(
            "{} / {size_label}",
            item.audio_duration_label
                .as_deref()
                .unwrap_or("Duration unavailable")
        ),
    }
}

fn child_absolute_path(parent_abs_path: &Path, child: &Entry) -> PathBuf {
    child
        .canonical_path
        .as_ref()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| match child.path.file_name() {
            Some(file_name) => parent_abs_path.join(file_name),
            None => parent_abs_path.join(child.path.as_std_path()),
        })
}

fn media_preview_summary(preview: &FolderMediaPreview) -> String {
    let mut parts = Vec::new();
    push_media_count(&mut parts, preview.image_count, "image", "images");
    push_media_count(&mut parts, preview.video_count, "video", "videos");
    push_media_count(&mut parts, preview.audio_count, "audio", "audio");

    if preview.scanned_cap_hit {
        parts.push("more".to_string());
    }

    parts.join(" / ")
}

fn push_media_count(parts: &mut Vec<String>, count: usize, singular: &str, plural: &str) {
    if count == 1 {
        parts.push(format!("1 {singular}"));
    } else if count > 1 {
        parts.push(format!("{count} {plural}"));
    }
}

fn media_preview_kind_for_path(path: &Path) -> Option<MediaPreviewKind> {
    let extension = path.extension()?.to_str()?;
    if matches_extension(extension, IMAGE_MEDIA_EXTENSIONS) {
        Some(MediaPreviewKind::Image)
    } else if matches_extension(extension, VIDEO_MEDIA_EXTENSIONS) {
        Some(MediaPreviewKind::Video)
    } else if matches_extension(extension, AUDIO_MEDIA_EXTENSIONS) {
        Some(MediaPreviewKind::Audio)
    } else {
        None
    }
}

fn matches_extension(extension: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
}

fn video_preview_frame_path(
    video_path: &Path,
    image_frame_candidates: &[(String, PathBuf)],
) -> Option<PathBuf> {
    let video_stem = media_stem_key(video_path)?;
    image_frame_candidates.iter().find_map(|(stem, path)| {
        (stem == &video_stem
            || (stem.starts_with(&video_stem)
                && (stem.contains("poster") || stem.contains("frame") || stem.contains("thumb"))))
        .then(|| path.clone())
    })
}

fn media_stem_key(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_ascii_lowercase())
}

fn audio_gradient_background(name: &str) -> gpui::Background {
    let (from, to) = audio_gradient_colors(name);
    linear_gradient(135., linear_color_stop(from, 0.), linear_color_stop(to, 1.))
}

fn audio_gradient_colors(name: &str) -> (Hsla, Hsla) {
    let hash = stable_text_hash(name);
    let hue = ((hash % 360) as f32) / 360.;
    let accent = (((hash >> 16) % 360) as f32) / 360.;
    (hsla(hue, 0.62, 0.46, 1.), hsla(accent, 0.58, 0.32, 1.))
}

fn media_size_label(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_ix = 0usize;
    while value >= 1024.0 && unit_ix < UNITS.len() - 1 {
        value /= 1024.0;
        unit_ix += 1;
    }

    if unit_ix == 0 {
        format!("{} {}", bytes, UNITS[unit_ix])
    } else if value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit_ix])
    } else {
        format!("{value:.1} {}", UNITS[unit_ix])
    }
}

fn stable_text_hash(value: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
