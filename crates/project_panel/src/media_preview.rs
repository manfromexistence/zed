mod generated_metadata;
mod generated_video_frame;
mod metadata;
mod metadata_probe;

pub(crate) use generated_metadata::{
    build_generated_media_metadata_job_batch, collect_generated_media_metadata,
};
pub(crate) use metadata::GeneratedMediaMetadataIndex;

use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use gpui::{
    AnyElement, App, Context, Div, FocusHandle, Hsla, ObjectFit, SharedString, Stateful, hsla, img,
    linear_color_stop, linear_gradient,
};
use project::{Entry, ProjectEntryId, WorktreeId};
use settings::Settings;
use ui::{
    ButtonLike, ButtonSize, ContextMenu, ListHeader, PopoverMenu, TintColor, Tooltip, prelude::*,
};
use workspace::{PreviewTabsSettings, SelectedEntry};

pub(crate) const MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN: usize = 512;
pub(crate) const MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS: usize = 12;
pub(crate) const PROJECT_PANEL_MEDIA_GALLERY_COLUMNS: u16 = 3;
pub(crate) const PROJECT_PANEL_MEDIA_SHELF_COLUMNS: u16 = 4;

const PROJECT_PANEL_MEDIA_GALLERY_CARD_WIDTH: f32 = 86.;
const PROJECT_PANEL_MEDIA_GALLERY_CARD_HEIGHT: f32 = 64.;
const PROJECT_PANEL_MEDIA_SHELF_CARD_MIN_WIDTH: f32 = 96.;
const PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT: f32 = 96.;

const IMAGE_MEDIA_EXTENSIONS: &[&str] = &[
    "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "tif", "tiff", "webp",
];
const VIDEO_MEDIA_EXTENSIONS: &[&str] = &["avi", "m4v", "mkv", "mov", "mp4", "mpeg", "mpg", "webm"];
const AUDIO_MEDIA_EXTENSIONS: &[&str] = &["aac", "flac", "m4a", "mp3", "ogg", "opus", "wav"];
const CENTER_FRAME_HINTS: &[&str] = &["center", "centre", "middle", "mid"];
const PREVIEW_FRAME_HINTS: &[&str] = &["poster", "frame", "thumb", "thumbnail", "preview"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MediaPreviewKind {
    Image,
    Video,
    Audio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VideoFramePreviewKind {
    Center,
    Preview,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VideoFramePreview {
    pub(crate) path: PathBuf,
    pub(crate) kind: VideoFramePreviewKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MediaPreviewItem {
    pub(crate) entry_id: ProjectEntryId,
    pub(crate) kind: MediaPreviewKind,
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) absolute_path: PathBuf,
    pub(crate) video_frame_preview: Option<VideoFramePreview>,
    pub(crate) duration_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FolderMediaPreview {
    pub(crate) image_count: usize,
    pub(crate) video_count: usize,
    pub(crate) audio_count: usize,
    pub(crate) total_count: usize,
    pub(crate) scanned_cap_hit: bool,
    pub(crate) metadata_probe_plan: Option<metadata_probe::MediaMetadataProbePlan>,
    pub(crate) items: Vec<MediaPreviewItem>,
}

pub(crate) fn build_folder_media_preview<'a>(
    parent_abs_path: &Path,
    children: impl Iterator<Item = &'a Entry>,
) -> Option<FolderMediaPreview> {
    build_folder_media_preview_with_generated_metadata(parent_abs_path, children, None)
}

pub(crate) fn build_folder_media_preview_with_generated_metadata<'a>(
    parent_abs_path: &Path,
    children: impl Iterator<Item = &'a Entry>,
    generated_metadata: Option<&GeneratedMediaMetadataIndex>,
) -> Option<FolderMediaPreview> {
    let mut child_entries = Vec::new();
    let mut media_scan_was_capped = false;
    for child in children.take(MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN + 1) {
        if child_entries.len() >= MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN {
            media_scan_was_capped = true;
            break;
        }
        child_entries.push(child);
    }

    let mut image_count = 0;
    let mut video_count = 0;
    let mut audio_count = 0;
    let mut items = Vec::new();
    let mut image_frame_candidates = Vec::new();
    let mut media_metadata = metadata::build_media_metadata_index(parent_abs_path, &child_entries);
    if let Some(generated_metadata) = generated_metadata {
        media_metadata.merge_generated_media_metadata(parent_abs_path, generated_metadata);
    }

    for child in child_entries {
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

        let name = child
            .path
            .file_name()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| absolute_path.display().to_string());
        items.push(MediaPreviewItem {
            entry_id: child.id,
            kind,
            name,
            size: child.size,
            duration_label: media_metadata.duration_label_for_path(&absolute_path),
            absolute_path,
            video_frame_preview: None,
        });
    }

    items.sort_by(media_preview_item_sort_order);
    items = select_balanced_media_preview_items(items);

    for item in &mut items {
        if item.kind == MediaPreviewKind::Video {
            item.video_frame_preview = media_metadata
                .video_frame_for_path(&item.absolute_path)
                .or_else(|| video_preview_frame(&item.absolute_path, &image_frame_candidates));
        }
    }

    let metadata_probe_plan =
        metadata_probe::build_media_metadata_probe_plan(parent_abs_path, &items);
    let total_count = image_count + video_count + audio_count;
    (total_count > 0).then_some(FolderMediaPreview {
        image_count,
        video_count,
        audio_count,
        total_count,
        scanned_cap_hit: media_scan_was_capped,
        metadata_probe_plan,
        items,
    })
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
    let header_count_label = media_preview_count_label(visible_count, preview);
    let gallery_cards = preview
        .items
        .iter()
        .take(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS)
        .map(|item| render_media_gallery_card("project-panel-media-gallery-card", item, cx))
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
            ListHeader::new("Media")
                .start_slot(Icon::new(dx_icon(DxUiIcon::Media)).size(IconSize::Small))
                .end_slot(
                    Label::new(header_count_label)
                        .size(LabelSize::Small)
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

pub(crate) fn render_folder_media_shelf(
    preview: &FolderMediaPreview,
    worktree_id: WorktreeId,
    selected_entry_id: Option<ProjectEntryId>,
    focus_handle: FocusHandle,
    panel_controls: Option<AnyElement>,
    cx: &mut Context<super::ProjectPanel>,
) -> AnyElement {
    let summary = media_preview_summary(preview);
    let metadata_probe_tooltip = preview
        .metadata_probe_plan
        .as_ref()
        .map(|plan| plan.summary_label());
    let visible_slots = media_shelf_visible_slots(preview);
    let has_overflow = preview.total_count > visible_slots || preview.scanned_cap_hit;
    let media_card_limit = if has_overflow {
        visible_slots.saturating_sub(1)
    } else {
        visible_slots
    };
    let visible_media_count = media_card_limit.min(preview.items.len());
    let mut shelf_cards = preview
        .items
        .iter()
        .take(media_card_limit)
        .map(|item| {
            render_media_shelf_card(
                item,
                worktree_id,
                selected_entry_id == Some(item.entry_id),
                focus_handle.clone(),
                cx,
            )
        })
        .collect::<Vec<_>>();
    if has_overflow {
        shelf_cards.push(render_media_shelf_overflow_card(
            preview,
            focus_handle.clone(),
            cx,
        ));
    }
    let header_count_label = media_preview_count_label(visible_media_count, preview);
    let header_controls = h_flex()
        .gap_1()
        .items_center()
        .child(
            Label::new(header_count_label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .single_line()
                .truncate(),
        )
        .when_some(panel_controls, |this, controls| this.child(controls));

    v_flex()
        .id(SharedString::from(format!(
            "project-panel-media-shelf-{:016x}",
            stable_text_hash(&summary)
        )))
        .w_full()
        .flex_none()
        .gap_1p5()
        .px_2()
        .py_2()
        .border_b_1()
        .border_color(cx.theme().colors().border.opacity(0.6))
        .bg(cx.theme().colors().panel_background)
        .when_some(metadata_probe_tooltip, |this, tooltip| {
            this.tooltip(move |_window, cx| Tooltip::with_meta("Media", None, tooltip.clone(), cx))
        })
        .child(
            ListHeader::new("Media")
                .start_slot(Icon::new(dx_icon(DxUiIcon::Media)).size(IconSize::Small))
                .end_slot(header_controls),
        )
        .child(
            div()
                .grid()
                .grid_cols(PROJECT_PANEL_MEDIA_SHELF_COLUMNS)
                .gap_1p5()
                .children(shelf_cards),
        )
        .into_any_element()
}

fn media_shelf_visible_slots(preview: &FolderMediaPreview) -> usize {
    let relevant_count = preview
        .total_count
        .max(preview.items.len())
        .min(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS);

    match relevant_count {
        0..=2 => relevant_count.max(1),
        3..=4 => 4,
        5..=8 => 8,
        _ => MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS,
    }
}

fn media_preview_count_label(visible_media_count: usize, preview: &FolderMediaPreview) -> String {
    if preview.scanned_cap_hit {
        format!("{visible_media_count} of {}+", preview.total_count)
    } else {
        format!("{visible_media_count} of {}", preview.total_count)
    }
}

fn render_media_shelf_overflow_card(
    preview: &FolderMediaPreview,
    focus_handle: FocusHandle,
    cx: &mut App,
) -> AnyElement {
    let summary = media_preview_summary(preview);
    let hidden_count = preview
        .total_count
        .saturating_sub(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS.saturating_sub(1));
    let tooltip = if hidden_count > 0 {
        format!("Show {hidden_count} more")
    } else {
        "Show gallery".to_string()
    };
    let gallery_preview = preview.clone();
    let menu_id = format!(
        "project-panel-media-shelf-overflow-{:016x}",
        stable_text_hash(&summary)
    );
    let trigger_id = SharedString::from(format!(
        "project-panel-media-shelf-more-{:016x}",
        stable_text_hash(&summary)
    ));

    PopoverMenu::new(menu_id)
        .trigger_with_tooltip(
            ButtonLike::new(trigger_id)
                .full_width()
                .height(px(PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT).into())
                .style(ButtonStyle::Subtle)
                .tab_index(0_isize)
                .track_focus(&focus_handle)
                .child(
                    div()
                        .min_w(px(PROJECT_PANEL_MEDIA_SHELF_CARD_MIN_WIDTH))
                        .w_full()
                        .v_flex()
                        .items_center()
                        .justify_center()
                        .gap_1()
                        .p_1()
                        .rounded_sm()
                        .border_1()
                        .border_color(cx.theme().colors().border_variant)
                        .child(Icon::new(IconName::Ellipsis).size(IconSize::Medium))
                        .child(
                            Label::new(if hidden_count > 0 {
                                format!("+{hidden_count} more")
                            } else {
                                "More".to_string()
                            })
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .single_line(),
                        ),
                ),
            move |_window, cx| Tooltip::with_meta("More media", None, tooltip.clone(), cx),
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
        })
        .into_any_element()
}

fn render_media_shelf_card(
    item: &MediaPreviewItem,
    worktree_id: WorktreeId,
    is_selected: bool,
    focus_handle: FocusHandle,
    cx: &mut Context<super::ProjectPanel>,
) -> AnyElement {
    let entry_id = item.entry_id;
    media_shelf_card_container(
        "project-panel-media-shelf-card",
        item,
        is_selected,
        focus_handle,
        cx,
    )
    .on_click(
        cx.listener(move |panel, event: &gpui::ClickEvent, window, cx| {
            cx.stop_propagation();
            window.focus(&panel.focus_handle, cx);
            let selection = SelectedEntry {
                worktree_id,
                entry_id,
            };

            if event.modifiers().secondary() {
                panel.selection = Some(selection);
                if let Some(position) = panel
                    .marked_entries
                    .iter()
                    .position(|entry| *entry == selection)
                {
                    panel.marked_entries.remove(position);
                } else {
                    panel.marked_entries.push(selection);
                }
                cx.notify();
                return;
            }

            panel.marked_entries.clear();
            panel.selection = Some(selection);
            let preview_tabs_enabled =
                PreviewTabsSettings::get_global(cx).enable_preview_from_project_panel;
            let click_count = event.click_count();
            panel.open_entry(
                entry_id,
                click_count > 1,
                preview_tabs_enabled && click_count == 1,
                cx,
            );
            cx.notify();
        }),
    )
    .on_right_click(
        cx.listener(move |panel, event: &gpui::ClickEvent, window, cx| {
            cx.stop_propagation();
            window.focus(&panel.focus_handle, cx);
            let selection = SelectedEntry {
                worktree_id,
                entry_id,
            };
            if !panel.marked_entries.contains(&selection) {
                panel.marked_entries.clear();
            }
            panel.selection = Some(selection);
            panel.deploy_context_menu(event.position(), entry_id, window, cx);
            cx.notify();
        }),
    )
    .into_any_element()
}

fn render_media_gallery_card(
    id_prefix: &'static str,
    item: &MediaPreviewItem,
    cx: &mut App,
) -> AnyElement {
    media_gallery_card_container(id_prefix, item, false, cx).into_any_element()
}

fn media_shelf_card_container(
    id_prefix: &'static str,
    item: &MediaPreviewItem,
    is_selected: bool,
    focus_handle: FocusHandle,
    cx: &mut App,
) -> ButtonLike {
    let colors = cx.theme().colors();
    let tooltip_title = item.name.clone();
    let tooltip_meta = media_preview_card_tooltip_meta(item);
    ButtonLike::new(SharedString::from(format!(
        "{id_prefix}-{:?}-{:?}",
        item.kind, item.entry_id
    )))
    .full_width()
    .height(px(PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT).into())
    .style(ButtonStyle::OutlinedGhost)
    .selected_style(ButtonStyle::Tinted(TintColor::Accent))
    .toggle_state(is_selected)
    .size(ButtonSize::None)
    .tab_index(0_isize)
    .track_focus(&focus_handle)
    .tooltip(move |_window, cx| {
        Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
    })
    .child(
        div()
            .min_w(px(PROJECT_PANEL_MEDIA_SHELF_CARD_MIN_WIDTH))
            .h(px(PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT))
            .w_full()
            .v_flex()
            .p_0()
            .rounded_sm()
            .border_1()
            .overflow_hidden()
            .border_color(if is_selected {
                colors.border_focused
            } else {
                colors.border_variant
            })
            .bg(if is_selected {
                colors.element_selected
            } else {
                colors.element_background
            })
            .child(render_media_shelf_card_body(item, cx)),
    )
}

fn render_media_shelf_card_body(item: &MediaPreviewItem, cx: &mut App) -> Div {
    let colors = cx.theme().colors();
    match item.kind {
        MediaPreviewKind::Image => div()
            .relative()
            .w_full()
            .flex_1()
            .rounded_sm()
            .overflow_hidden()
            .child(
                img(item.absolute_path.clone())
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .with_fallback(|| media_card_image_fallback(MediaPreviewKind::Image)),
            )
            .child(media_shelf_name_overlay(&item.name, cx)),
        MediaPreviewKind::Video => {
            let base = div()
                .relative()
                .w_full()
                .flex_1()
                .rounded_sm()
                .overflow_hidden()
                .bg(colors.elevated_surface_background);

            let base = if let Some(preview) = item.video_frame_preview.as_ref() {
                base.child(
                    img(preview.path.clone())
                        .size_full()
                        .object_fit(ObjectFit::Cover)
                        .with_fallback(|| media_card_image_fallback(MediaPreviewKind::Video)),
                )
            } else {
                base.flex().items_center().justify_center().child(
                    Icon::new(IconName::PlayOutlined)
                        .size(IconSize::XLarge)
                        .color(Color::Muted),
                )
            };

            base.child(
                div()
                    .absolute()
                    .right_1()
                    .top_1()
                    .rounded_full()
                    .bg(colors.editor_background.opacity(0.72))
                    .p_0p5()
                    .child(
                        Icon::new(IconName::PlayOutlined)
                            .size(IconSize::Small)
                            .color(Color::Accent),
                    ),
            )
            .child(media_shelf_name_overlay(&item.name, cx))
        }
        MediaPreviewKind::Audio => div()
            .relative()
            .w_full()
            .flex_1()
            .rounded_sm()
            .overflow_hidden()
            .bg(audio_gradient_background(&item.name))
            .flex()
            .items_center()
            .justify_center()
            .px_2()
            .child(audio_media_label(&item.name, IconSize::Small, cx)),
    }
}

fn audio_media_label(name: &str, icon_size: IconSize, cx: &mut App) -> Div {
    let colors = cx.theme().colors();

    h_flex()
        .max_w_full()
        .items_center()
        .justify_center()
        .gap_1()
        .rounded_sm()
        .bg(colors.editor_background.opacity(0.78))
        .px_1p5()
        .py_0p5()
        .shadow_sm()
        .child(
            Icon::new(IconName::AudioOn)
                .size(icon_size)
                .color(Color::Accent),
        )
        .child(
            Label::new(name.to_string())
                .size(LabelSize::Small)
                .color(Color::Default)
                .buffer_font(cx)
                .single_line()
                .truncate(),
        )
}

fn media_shelf_name_overlay(name: &str, cx: &mut App) -> Div {
    let colors = cx.theme().colors();
    div()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .px_1()
        .py_0p5()
        .bg(colors.editor_background.opacity(0.74))
        .shadow_md()
        .child(
            Label::new(name.to_string())
                .size(LabelSize::Small)
                .color(Color::Default)
                .buffer_font(cx)
                .single_line()
                .truncate(),
        )
}

fn media_gallery_card_container(
    id_prefix: &'static str,
    item: &MediaPreviewItem,
    is_selected: bool,
    cx: &mut App,
) -> Stateful<Div> {
    let colors = cx.theme().colors();
    let elevated_surface_background = colors.elevated_surface_background;
    let editor_background = colors.editor_background;
    let border_focused = colors.border_focused;
    let border_variant = colors.border_variant;
    let element_selected = colors.element_selected;
    let element_background = colors.element_background;
    let element_hover = colors.element_hover;
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
                    .object_fit(ObjectFit::Cover)
                    .with_fallback(|| media_card_image_fallback(MediaPreviewKind::Image)),
            ),
        MediaPreviewKind::Video => {
            let base = div()
                .relative()
                .w_full()
                .h(px(PROJECT_PANEL_MEDIA_GALLERY_CARD_HEIGHT))
                .rounded_sm()
                .overflow_hidden()
                .bg(elevated_surface_background);

            let base = if let Some(preview) = item.video_frame_preview.as_ref() {
                base.child(
                    img(preview.path.clone())
                        .size_full()
                        .object_fit(ObjectFit::Cover)
                        .with_fallback(|| media_card_image_fallback(MediaPreviewKind::Video)),
                )
            } else {
                base.flex().items_center().justify_center().child(
                    Icon::new(IconName::PlayOutlined)
                        .size(IconSize::XLarge)
                        .color(Color::Muted),
                )
            };

            base.child(
                div()
                    .absolute()
                    .right_1()
                    .bottom_1()
                    .rounded_full()
                    .bg(editor_background.opacity(0.72))
                    .p_0p5()
                    .child(
                        Icon::new(IconName::PlayOutlined)
                            .size(IconSize::Small)
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
            .child(audio_media_label(&item.name, IconSize::Small, cx)),
    };

    div()
        .id(SharedString::from(format!(
            "{id_prefix}-{:?}-{:?}",
            item.kind, item.entry_id
        )))
        .min_w(px(0.))
        .w(px(PROJECT_PANEL_MEDIA_GALLERY_CARD_WIDTH))
        .v_flex()
        .gap_1()
        .p_1()
        .rounded_sm()
        .border_1()
        .border_color(if is_selected {
            border_focused
        } else {
            border_variant
        })
        .bg(if is_selected {
            element_selected
        } else {
            element_background
        })
        .when(!is_selected, |this| {
            this.hover(|style| style.bg(element_hover))
        })
        .tooltip(move |_window, cx| {
            Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
        })
        .child(media)
        .child(
            Label::new(item.name.clone())
                .size(LabelSize::Small)
                .color(Color::Muted)
                .single_line()
                .truncate(),
        )
}

fn media_preview_card_tooltip_meta(item: &MediaPreviewItem) -> String {
    let size_label = media_size_label(item.size);
    match item.kind {
        MediaPreviewKind::Image => size_label,
        MediaPreviewKind::Video => {
            let time_label = item.duration_label.as_deref().unwrap_or("Unknown duration");
            if let Some(preview) = item.video_frame_preview.as_ref() {
                let frame_label = video_frame_preview_label(preview);
                format!("{frame_label} - {time_label} - {size_label}")
            } else {
                format!("No thumbnail - {time_label} - {size_label}")
            }
        }
        MediaPreviewKind::Audio => format!(
            "{} - {size_label}",
            item.duration_label.as_deref().unwrap_or("Unknown duration")
        ),
    }
}

fn media_card_image_fallback(kind: MediaPreviewKind) -> AnyElement {
    let icon_name = match kind {
        MediaPreviewKind::Image => IconName::Image,
        MediaPreviewKind::Video => IconName::PlayOutlined,
        MediaPreviewKind::Audio => IconName::AudioOn,
    };

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child(
            Icon::new(icon_name)
                .size(IconSize::XLarge)
                .color(Color::Muted),
        )
        .into_any_element()
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

pub(crate) fn is_media_path(path: &Path) -> bool {
    media_preview_kind_for_path(path).is_some()
}

fn media_kind_sort_rank(kind: MediaPreviewKind) -> u8 {
    match kind {
        MediaPreviewKind::Image => 0,
        MediaPreviewKind::Video => 1,
        MediaPreviewKind::Audio => 2,
    }
}

fn media_preview_item_sort_order(left: &MediaPreviewItem, right: &MediaPreviewItem) -> Ordering {
    media_kind_sort_rank(left.kind)
        .cmp(&media_kind_sort_rank(right.kind))
        .then_with(|| {
            left.name
                .to_ascii_lowercase()
                .cmp(&right.name.to_ascii_lowercase())
        })
}

fn select_balanced_media_preview_items(items: Vec<MediaPreviewItem>) -> Vec<MediaPreviewItem> {
    if items.len() <= MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS {
        return items;
    }

    let mut selected = Vec::with_capacity(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS);
    for kind in [
        MediaPreviewKind::Image,
        MediaPreviewKind::Video,
        MediaPreviewKind::Audio,
    ] {
        if let Some(item) = items.iter().find(|item| item.kind == kind) {
            push_media_preview_item_if_missing(&mut selected, item);
        }
    }

    for item in &items {
        if selected.len() >= MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS {
            break;
        }
        push_media_preview_item_if_missing(&mut selected, item);
    }

    selected.sort_by(media_preview_item_sort_order);
    selected
}

fn push_media_preview_item_if_missing(
    selected: &mut Vec<MediaPreviewItem>,
    item: &MediaPreviewItem,
) {
    if !selected
        .iter()
        .any(|selected_item| selected_item.entry_id == item.entry_id)
    {
        selected.push(item.clone());
    }
}

fn matches_extension(extension: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
}

fn video_preview_frame(
    video_path: &Path,
    image_frame_candidates: &[(String, PathBuf)],
) -> Option<VideoFramePreview> {
    let video_stem = media_stem_key(video_path)?;
    image_frame_candidates
        .iter()
        .filter_map(|(stem, path)| {
            video_frame_candidate_rank(&video_stem, stem).map(|rank| (rank, path))
        })
        .min_by_key(|(rank, _)| *rank)
        .map(|(rank, path)| VideoFramePreview {
            path: path.clone(),
            kind: video_frame_preview_kind_for_rank(rank),
        })
}

fn video_frame_preview_kind_for_rank(rank: u8) -> VideoFramePreviewKind {
    if rank == 0 {
        VideoFramePreviewKind::Center
    } else {
        VideoFramePreviewKind::Preview
    }
}

fn video_frame_preview_label(preview: &VideoFramePreview) -> &'static str {
    match preview.kind {
        VideoFramePreviewKind::Center => "Thumbnail",
        VideoFramePreviewKind::Preview => "Thumbnail",
    }
}

fn video_frame_candidate_rank(video_stem: &str, candidate_stem: &str) -> Option<u8> {
    if candidate_stem == video_stem {
        return Some(2);
    }

    if !candidate_stem.starts_with(video_stem) {
        return None;
    }

    if CENTER_FRAME_HINTS
        .iter()
        .any(|hint| candidate_stem.contains(hint))
    {
        Some(0)
    } else if PREVIEW_FRAME_HINTS
        .iter()
        .any(|hint| candidate_stem.contains(hint))
    {
        Some(1)
    } else {
        None
    }
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
