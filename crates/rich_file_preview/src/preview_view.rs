use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use calamine::{Reader, open_workbook_auto};
use csv::ReaderBuilder;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels, Point, Render, RenderImage, SharedString,
    Task, Window, canvas, img, point, prelude::*, px,
};
use image::{Delay, Frame, RgbaImage};
use markdown::{
    CodeBlockRenderer, Markdown, MarkdownElement, MarkdownFont, MarkdownOptions, MarkdownStyle,
};
use pdfium_render::prelude::*;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use rodio::{Decoder as RodioDecoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use smallvec::SmallVec;
use tiny_skia::Pixmap;
use ui::{Color, Icon, IconName, Label, prelude::*};
use workspace::ToolbarItemLocation;
use workspace::item::{Item, ItemBufferKind};

use crate::asset_item::PreviewAssetItem;
use crate::registry::PreviewKind;

pub struct UniversalPreviewView {
    project: Entity<project::Project>,
    item: Entity<PreviewAssetItem>,
    focus_handle: FocusHandle,
    _load_task: Task<()>,
    state: PreviewState,
}

enum PreviewState {
    Loading,
    Ready(LoadedPreview),
    Error(SharedString),
}

enum LoadedPreview {
    Markdown(Entity<Markdown>),
    Image(ImagePreview),
    Text(TextPreview),
    Table(TablePreview),
    Slides(SlidesPreview),
    Audio(AudioPreview),
    Video(VideoPreview),
    Mesh(MeshPreview),
}

struct ImagePreview {
    _title: String,
    summary: String,
    image: Arc<RenderImage>,
}

struct TextPreview {
    title: String,
    paragraphs: Vec<String>,
}

struct TablePreview {
    title: String,
    sheets: Vec<SheetPreview>,
    active_sheet: usize,
}

struct SheetPreview {
    name: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

struct SlidesPreview {
    title: String,
    slides: Vec<SlidePreview>,
    active_slide: usize,
}

struct SlidePreview {
    title: String,
    bullets: Vec<String>,
}

struct AudioPreview {
    _title: String,
    path: PathBuf,
    waveform: Vec<f32>,
    duration: Duration,
    position: Duration,
    playback: Option<AudioPlayback>,
    generation: u64,
}

struct AudioPlayback {
    _device_sink: MixerDeviceSink,
    player: Player,
}

struct VideoPreview {
    _title: String,
    frames: Arc<Vec<Arc<RenderImage>>>,
    fps: f32,
    duration: Duration,
    current_frame: usize,
    playing: bool,
    generation: u64,
}

struct MeshPreview {
    _title: String,
    vertices: Vec<Vec3>,
    indices: Vec<[usize; 3]>,
    yaw: f32,
    pitch: f32,
    distance: f32,
    dragging_from: Option<Point<Pixels>>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

enum LoadedPreviewPayload {
    Markdown(String),
    Image(ImagePreview),
    Text(TextPreview),
    Table(TablePreview),
    Slides(SlidesPreview),
    Audio(AudioPreview),
    #[cfg_attr(not(feature = "ffmpeg-video"), allow(dead_code))]
    Video(VideoPreview),
    Mesh(MeshPreview),
}

impl UniversalPreviewView {
    pub fn new(
        project: Entity<project::Project>,
        item: Entity<PreviewAssetItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let item_read = item.read(cx);
        let path = item_read.abs_path.clone();
        let kind = item_read.kind;
        let title = item_read.file_name.clone();
        let load_task = if let Some(path) = path {
            let bg = cx.background_spawn(async move { load_preview(kind, &path, &title) });
            cx.spawn_in(window, async move |this, cx| {
                let loaded = bg.await;
                let _ = this.update_in(cx, |this, _window, cx| {
                    this.finish_load(loaded, cx);
                });
            })
        } else {
            Task::ready(())
        };

        Self {
            project,
            item,
            focus_handle,
            _load_task: load_task,
            state: PreviewState::Loading,
        }
    }

    fn finish_load(&mut self, loaded: Result<LoadedPreviewPayload>, cx: &mut Context<Self>) {
        self.state = match loaded {
            Ok(LoadedPreviewPayload::Markdown(source)) => {
                let languages = self.project.read(cx).languages().clone();
                let markdown = cx.new(|cx| {
                    Markdown::new_with_options(
                        source.into(),
                        Some(languages),
                        None,
                        MarkdownOptions {
                            parse_html: true,
                            render_mermaid_diagrams: true,
                            ..Default::default()
                        },
                        cx,
                    )
                });
                PreviewState::Ready(LoadedPreview::Markdown(markdown))
            }
            Ok(LoadedPreviewPayload::Image(data)) => {
                PreviewState::Ready(LoadedPreview::Image(data))
            }
            Ok(LoadedPreviewPayload::Text(data)) => PreviewState::Ready(LoadedPreview::Text(data)),
            Ok(LoadedPreviewPayload::Table(data)) => {
                PreviewState::Ready(LoadedPreview::Table(data))
            }
            Ok(LoadedPreviewPayload::Slides(data)) => {
                PreviewState::Ready(LoadedPreview::Slides(data))
            }
            Ok(LoadedPreviewPayload::Audio(data)) => {
                PreviewState::Ready(LoadedPreview::Audio(data))
            }
            Ok(LoadedPreviewPayload::Video(data)) => {
                PreviewState::Ready(LoadedPreview::Video(data))
            }
            Ok(LoadedPreviewPayload::Mesh(data)) => PreviewState::Ready(LoadedPreview::Mesh(data)),
            Err(error) => PreviewState::Error(format!("{error:#}").into()),
        };
        cx.notify();
    }

    fn title(&self, cx: &App) -> String {
        self.item.read(cx).title()
    }

    fn kind_label(&self, cx: &App) -> &'static str {
        self.item.read(cx).kind.label()
    }
}
impl UniversalPreviewView {
    fn toggle_audio_playback(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let should_track = if let PreviewState::Ready(LoadedPreview::Audio(audio)) = &mut self.state
        {
            if let Some(playback) = &audio.playback {
                if playback.player.is_paused() {
                    playback.player.play();
                } else {
                    playback.player.pause();
                }
            } else {
                let stream = DeviceSinkBuilder::open_default_sink();
                let file = File::open(&audio.path);
                if let (Ok(mut device_sink), Ok(file)) = (stream, file) {
                    if let Ok(decoder) = RodioDecoder::try_from(BufReader::new(file)) {
                        device_sink.log_on_drop(false);
                        let player = Player::connect_new(device_sink.mixer());
                        player.append(decoder);
                        audio.playback = Some(AudioPlayback {
                            _device_sink: device_sink,
                            player,
                        });
                    }
                }
            }
            audio.generation += 1;
            audio
                .playback
                .as_ref()
                .is_some_and(|playback| !playback.player.is_paused())
        } else {
            false
        };

        if should_track {
            self.track_audio_progress(window, cx);
        }
        cx.notify();
    }

    fn track_audio_progress(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let generation = if let PreviewState::Ready(LoadedPreview::Audio(audio)) = &mut self.state {
            audio.generation
        } else {
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(120))
                    .await;
                let keep_going = this
                    .update(cx, |this, cx| {
                        let PreviewState::Ready(LoadedPreview::Audio(audio)) = &mut this.state
                        else {
                            return false;
                        };
                        let Some(playback) = &audio.playback else {
                            return false;
                        };
                        if generation != audio.generation {
                            return false;
                        }
                        audio.position = playback.player.get_pos();
                        cx.notify();
                        !playback.player.empty()
                    })
                    .unwrap_or(false);
                if !keep_going {
                    break;
                }
            }
        })
        .detach();
    }

    fn toggle_video_playback(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let start = if let PreviewState::Ready(LoadedPreview::Video(video)) = &mut self.state {
            video.playing = !video.playing;
            video.generation += 1;
            video.playing
        } else {
            false
        };
        if start {
            self.start_video_loop(window, cx);
        }
        cx.notify();
    }

    fn start_video_loop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let generation = if let PreviewState::Ready(LoadedPreview::Video(video)) = &mut self.state {
            video.generation
        } else {
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            loop {
                let frame_delay = this
                    .update(cx, |this, _| {
                        let PreviewState::Ready(LoadedPreview::Video(video)) = &mut this.state
                        else {
                            return None;
                        };
                        if !video.playing || generation != video.generation {
                            return None;
                        }
                        Some(Duration::from_secs_f32(
                            (1.0 / video.fps.max(1.0)).max(0.01),
                        ))
                    })
                    .ok()
                    .flatten();

                let Some(frame_delay) = frame_delay else {
                    break;
                };

                cx.background_executor().timer(frame_delay).await;
                let stop = this
                    .update(cx, |this, cx| {
                        let PreviewState::Ready(LoadedPreview::Video(video)) = &mut this.state
                        else {
                            return true;
                        };
                        if !video.playing || generation != video.generation {
                            return true;
                        }
                        video.current_frame += 1;
                        if video.current_frame >= video.frames.len() {
                            video.current_frame = 0;
                            video.playing = false;
                            cx.notify();
                            true
                        } else {
                            cx.notify();
                            false
                        }
                    })
                    .unwrap_or(true);
                if stop {
                    break;
                }
            }
        })
        .detach();
    }

    fn begin_model_drag(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let PreviewState::Ready(LoadedPreview::Mesh(mesh)) = &mut self.state {
            mesh.dragging_from = Some(event.position);
            cx.notify();
        }
    }

    fn end_model_drag(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let PreviewState::Ready(LoadedPreview::Mesh(mesh)) = &mut self.state {
            mesh.dragging_from = None;
            cx.notify();
        }
    }

    fn update_model_drag(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let PreviewState::Ready(LoadedPreview::Mesh(mesh)) = &mut self.state {
            let Some(last) = mesh.dragging_from else {
                return;
            };
            let delta = event.position - last;
            mesh.yaw += f32::from(delta.x) * 0.01;
            mesh.pitch = (mesh.pitch + f32::from(delta.y) * 0.01).clamp(-1.4, 1.4);
            mesh.dragging_from = Some(event.position);
            cx.notify();
        }
    }

    fn handle_model_scroll(
        &mut self,
        event: &gpui::ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let PreviewState::Ready(LoadedPreview::Mesh(mesh)) = &mut self.state {
            let delta = match event.delta {
                gpui::ScrollDelta::Pixels(delta) => f32::from(delta.y),
                gpui::ScrollDelta::Lines(lines) => lines.y * 24.0,
            };
            mesh.distance = (mesh.distance + delta * 0.01).clamp(1.5, 20.0);
            cx.notify();
        }
    }
}

impl Focusable for UniversalPreviewView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for UniversalPreviewView {}

impl Item for UniversalPreviewView {
    type Event = ();

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::FileDoc))
    }

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        self.title(cx).into()
    }

    fn buffer_kind(&self, _cx: &App) -> ItemBufferKind {
        ItemBufferKind::Singleton
    }

    fn breadcrumb_location(&self, _cx: &App) -> ToolbarItemLocation {
        ToolbarItemLocation::PrimaryLeft
    }
}

impl Render for UniversalPreviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match &self.state {
            PreviewState::Loading => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(Label::new("Loading rich preview..."))
                .into_any_element(),
            PreviewState::Error(message) => div()
                .size_full()
                .id("rich-preview-error-scroll")
                .overflow_y_scroll()
                .p_4()
                .flex()
                .flex_col()
                .gap_2()
                .child(Label::new("Preview failed").color(Color::Error))
                .child(Label::new(message.clone()))
                .into_any_element(),
            PreviewState::Ready(LoadedPreview::Markdown(markdown)) => MarkdownElement::new(
                markdown.clone(),
                MarkdownStyle::themed(MarkdownFont::Editor, window, cx),
            )
            .code_block_renderer(CodeBlockRenderer::Default {
                copy_button: true,
                copy_button_on_hover: true,
                border: true,
            })
            .into_any_element(),
            PreviewState::Ready(LoadedPreview::Image(image)) => div()
                .size_full()
                .id("rich-preview-image-scroll")
                .overflow_scroll()
                .p_4()
                .flex()
                .flex_col()
                .gap_2()
                .child(Label::new(image.summary.clone()).color(Color::Muted))
                .child(img(image.image.clone()).max_w_full().max_h_full())
                .into_any_element(),
            PreviewState::Ready(LoadedPreview::Text(text)) => div()
                .size_full()
                .id("rich-preview-text-scroll")
                .overflow_y_scroll()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(Label::new(text.title.clone()))
                .children(text.paragraphs.iter().map(|paragraph| {
                    Label::new(paragraph.clone())
                        .color(Color::Muted)
                        .into_any_element()
                }))
                .into_any_element(),
            PreviewState::Ready(LoadedPreview::Table(table)) => {
                let sheet = table.sheets.get(table.active_sheet);
                sheet.map_or_else(
                    || {
                        div()
                            .size_full()
                            .child(Label::new("No sheets parsed"))
                            .into_any_element()
                    },
                    |sheet| {
                        div()
                            .size_full()
                            .id("rich-preview-table-scroll")
                            .overflow_scroll()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(Label::new(format!("{} - {}", table.title, sheet.name)))
                            .child(div().flex().gap_2().children(sheet.headers.iter().map(
                                |header| {
                                    div()
                                        .min_w(px(180.0))
                                        .p_2()
                                        .rounded_sm()
                                        .bg(gpui::rgb(0x202734))
                                        .child(Label::new(header.clone()))
                                        .into_any_element()
                                },
                            )))
                            .children(sheet.rows.iter().take(200).map(|row| {
                                div()
                                    .flex()
                                    .gap_2()
                                    .children(row.iter().map(|cell| {
                                        div()
                                            .min_w(px(180.0))
                                            .p_2()
                                            .rounded_sm()
                                            .bg(gpui::rgb(0x11161E))
                                            .child(Label::new(cell.clone()).color(Color::Muted))
                                            .into_any_element()
                                    }))
                                    .into_any_element()
                            }))
                            .into_any_element()
                    },
                )
            }
            PreviewState::Ready(LoadedPreview::Slides(deck)) => {
                let slide = deck.slides.get(deck.active_slide);
                slide.map_or_else(
                    || {
                        div()
                            .size_full()
                            .child(Label::new("No slides parsed"))
                            .into_any_element()
                    },
                    |slide| {
                        div()
                            .size_full()
                            .id("rich-preview-slide-scroll")
                            .overflow_y_scroll()
                            .p_4()
                            .child(
                                div()
                                    .max_w(px(960.0))
                                    .mx_auto()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .p_6()
                                    .rounded_lg()
                                    .bg(gpui::rgb(0x161D2B))
                                    .child(Label::new(format!(
                                        "{} - Slide {}/{}",
                                        deck.title,
                                        deck.active_slide + 1,
                                        deck.slides.len()
                                    )))
                                    .child(Label::new(slide.title.clone()))
                                    .children(slide.bullets.iter().map(|bullet| {
                                        Label::new(format!("- {bullet}"))
                                            .color(Color::Muted)
                                            .into_any_element()
                                    })),
                            )
                            .into_any_element()
                    },
                )
            }
            PreviewState::Ready(LoadedPreview::Audio(audio)) => div()
                .size_full()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(
                    Button::new(
                        "audio-toggle",
                        if audio
                            .playback
                            .as_ref()
                            .is_some_and(|playback| !playback.player.is_paused())
                        {
                            "Pause"
                        } else {
                            "Play"
                        },
                    )
                    .on_click(
                        cx.listener(|this, _, window, cx| this.toggle_audio_playback(window, cx)),
                    ),
                )
                .child(
                    div()
                        .h(px(180.0))
                        .id("rich-preview-audio-waveform-scroll")
                        .items_end()
                        .gap_1()
                        .overflow_x_scroll()
                        .child(div().flex().items_end().gap_1().children(
                            audio.waveform.iter().enumerate().map(|(ix, value)| {
                                div()
                                    .w(px(3.0))
                                    .h(px((value * 88.0).max(6.0)))
                                    .rounded_sm()
                                    .bg(if ix % 4 == 0 {
                                        gpui::rgb(0x4DA3FF)
                                    } else {
                                        gpui::rgb(0x245C96)
                                    })
                                    .into_any_element()
                            }),
                        )),
                )
                .child(
                    Label::new(format!(
                        "{} / {}",
                        format_duration(audio.position),
                        format_duration(audio.duration)
                    ))
                    .color(Color::Muted),
                )
                .into_any_element(),
            PreviewState::Ready(LoadedPreview::Video(video)) => {
                let frame = video.frames.get(video.current_frame).cloned();
                frame.map_or_else(
                    || {
                        div()
                            .size_full()
                            .child(Label::new("No decoded frames"))
                            .into_any_element()
                    },
                    |frame| {
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_3()
                            .child(
                                Button::new(
                                    "video-toggle",
                                    if video.playing { "Pause" } else { "Play" },
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| this.toggle_video_playback(window, cx),
                                )),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .items_center()
                                    .justify_center()
                                    .rounded_lg()
                                    .bg(gpui::rgb(0x0F1218))
                                    .child(img(frame).max_w_full().max_h_full()),
                            )
                            .child(
                                Label::new(format!(
                                    "{} / {} - {} frames",
                                    format_duration(Duration::from_secs_f32(
                                        video.current_frame as f32 / video.fps.max(1.0)
                                    )),
                                    format_duration(video.duration),
                                    video.frames.len()
                                ))
                                .color(Color::Muted),
                            )
                            .into_any_element()
                    },
                )
            }
            PreviewState::Ready(LoadedPreview::Mesh(mesh)) => {
                let vertices = mesh.vertices.clone();
                let indices = mesh.indices.clone();
                let yaw = mesh.yaw;
                let pitch = mesh.pitch;
                let distance = mesh.distance;
                div()
                    .size_full()
                    .rounded_lg()
                    .bg(gpui::rgb(0x0D1117))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::begin_model_drag))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::end_model_drag))
                    .on_mouse_move(cx.listener(Self::update_model_drag))
                    .on_scroll_wheel(cx.listener(Self::handle_model_scroll))
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                let center = point(
                                    bounds.origin.x + bounds.size.width / 2.0,
                                    bounds.origin.y + bounds.size.height / 2.0,
                                );
                                for triangle in &indices {
                                    let [a, b, c] = *triangle;
                                    let Some(pa) = project_point(
                                        vertices.get(a).copied(),
                                        yaw,
                                        pitch,
                                        distance,
                                        center,
                                    ) else {
                                        continue;
                                    };
                                    let Some(pb) = project_point(
                                        vertices.get(b).copied(),
                                        yaw,
                                        pitch,
                                        distance,
                                        center,
                                    ) else {
                                        continue;
                                    };
                                    let Some(pc) = project_point(
                                        vertices.get(c).copied(),
                                        yaw,
                                        pitch,
                                        distance,
                                        center,
                                    ) else {
                                        continue;
                                    };
                                    let mut builder = PathBuilder::stroke(px(1.0));
                                    builder.move_to(pa);
                                    builder.line_to(pb);
                                    builder.line_to(pc);
                                    builder.line_to(pa);
                                    if let Ok(path) = builder.build() {
                                        window.paint_path(path, gpui::rgb(0x7CC6FF));
                                    }
                                }
                            },
                        )
                        .size_full(),
                    )
                    .into_any_element()
            }
        };

        div()
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().colors().editor_background)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .gap_2()
                    .p_2()
                    .border_b_1()
                    .border_color(gpui::rgb(0x283244))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(Icon::new(IconName::FileDoc).color(Color::Muted))
                            .child(Label::new(self.title(cx)))
                            .child(
                                Label::new(format!("- {}", self.kind_label(cx)))
                                    .color(Color::Muted),
                            ),
                    ),
            )
            .child(div().size_full().child(body))
    }
}
fn load_preview(kind: PreviewKind, path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    match kind {
        PreviewKind::Markdown => Ok(LoadedPreviewPayload::Markdown(std::fs::read_to_string(
            path,
        )?)),
        PreviewKind::Svg => load_svg(path, title),
        PreviewKind::Pdf => load_pdf(path, title),
        PreviewKind::Latex => load_latex(path, title),
        PreviewKind::Docx => load_docx(path, title),
        PreviewKind::Spreadsheet => load_spreadsheet(path, title),
        PreviewKind::Presentation => load_presentation(path, title),
        PreviewKind::Audio => load_audio(path, title),
        PreviewKind::Video => load_video(path, title),
        PreviewKind::Model3d => load_mesh(path, title),
    }
}

fn load_svg(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let bytes = std::fs::read(path)?;
    let tree = resvg::usvg::Tree::from_data(&bytes, &resvg::usvg::Options::default())?;
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).context("creating SVG pixmap")?;
    let _ = resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    Ok(LoadedPreviewPayload::Image(ImagePreview {
        _title: title.to_string(),
        summary: format!("SVG - {}x{}", size.width(), size.height()),
        image: rgba_to_render_image(pixmap.width(), pixmap.height(), pixmap.data().to_vec())?,
    }))
}

fn load_pdf(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let bindings = Pdfium::bind_to_system_library()?;
    let pdfium = Pdfium::new(bindings);
    let document = pdfium.load_pdf_from_file(path, None)?;
    let page = document
        .pages()
        .iter()
        .next()
        .context("pdf contained no pages")?;
    let bitmap = page.render_with_config(&PdfRenderConfig::new().set_target_width(1600))?;
    let summary = format!(
        "PDF - {} pages - {} chars",
        document.pages().len(),
        page.text().map(|text| text.all().len()).unwrap_or_default()
    );
    Ok(LoadedPreviewPayload::Image(ImagePreview {
        _title: title.to_string(),
        summary,
        image: rgba_to_render_image(
            bitmap.width() as u32,
            bitmap.height() as u32,
            bitmap.as_rgba_bytes().to_vec(),
        )?,
    }))
}

fn load_latex(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let temp = tempfile::tempdir().context("creating tectonic temp dir")?;
    let output = std::process::Command::new("tectonic")
        .arg(path)
        .arg("--outdir")
        .arg(temp.path())
        .output()
        .with_context(|| format!("running tectonic for {}", path.display()))?;
    if !output.status.success() {
        bail!(
            "tectonic failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let pdf_path = temp.path().join(
        path.file_stem()
            .map(|stem| format!("{}.pdf", stem.to_string_lossy()))
            .unwrap_or_else(|| "output.pdf".to_string()),
    );
    load_pdf(&pdf_path, title)
}

fn load_docx(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let file = File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")?.read_to_string(&mut xml)?;
    Ok(LoadedPreviewPayload::Text(TextPreview {
        title: title.to_string(),
        paragraphs: extract_xml_text(&xml, b"w:t")?,
    }))
}

fn load_spreadsheet(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    if ext.eq_ignore_ascii_case("csv") {
        let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;
        let headers = reader
            .headers()?
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let rows = reader
            .records()
            .map(|record| {
                record
                    .map(|row| {
                        row.iter()
                            .map(|value| value.to_string())
                            .collect::<Vec<_>>()
                    })
                    .map_err(anyhow::Error::from)
            })
            .collect::<Result<Vec<_>>>()?;
        return Ok(LoadedPreviewPayload::Table(TablePreview {
            title: title.to_string(),
            sheets: vec![SheetPreview {
                name: "Sheet 1".to_string(),
                headers,
                rows,
            }],
            active_sheet: 0,
        }));
    }

    let mut workbook = open_workbook_auto(path)?;
    let mut sheets = Vec::new();
    for sheet_name in workbook.sheet_names().to_vec() {
        let range = workbook.worksheet_range(&sheet_name)?;
        let mut rows = range
            .rows()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let headers = rows.first().cloned().unwrap_or_default();
        if !rows.is_empty() {
            rows.remove(0);
        }
        sheets.push(SheetPreview {
            name: sheet_name,
            headers,
            rows,
        });
    }
    Ok(LoadedPreviewPayload::Table(TablePreview {
        title: title.to_string(),
        sheets,
        active_sheet: 0,
    }))
}

fn load_presentation(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let file = File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut slide_names = Vec::new();
    for ix in 0..zip.len() {
        let name = zip.by_index(ix)?.name().to_string();
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            slide_names.push(name);
        }
    }
    slide_names.sort();
    let mut slides = Vec::new();
    for slide_name in slide_names {
        let mut xml = String::new();
        zip.by_name(&slide_name)?.read_to_string(&mut xml)?;
        let texts = extract_xml_text(&xml, b"a:t")?;
        slides.push(SlidePreview {
            title: texts
                .first()
                .cloned()
                .unwrap_or_else(|| "Untitled slide".to_string()),
            bullets: texts.into_iter().skip(1).collect(),
        });
    }
    Ok(LoadedPreviewPayload::Slides(SlidesPreview {
        title: title.to_string(),
        slides,
        active_slide: 0,
    }))
}

fn load_audio(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let file = File::open(path)?;
    let decoder = RodioDecoder::try_from(BufReader::new(file))?;
    let duration = decoder.total_duration().unwrap_or_default();
    let samples = decoder.take((48_000 * 60) as usize).collect::<Vec<_>>();
    Ok(LoadedPreviewPayload::Audio(AudioPreview {
        _title: title.to_string(),
        path: path.to_path_buf(),
        waveform: build_waveform(&samples, 192),
        duration,
        position: Duration::ZERO,
        playback: None,
        generation: 0,
    }))
}

#[cfg(feature = "ffmpeg-video")]
fn load_video(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    video_rs::init()?;
    let source = path.to_path_buf().into();
    let mut decoder = video_rs::decode::Decoder::new(&source)?;
    let mut frames = Vec::new();
    for decoded in decoder.decode_iter().take(180) {
        let (_timestamp, frame) = decoded?;
        let shape = frame.shape();
        if shape.len() != 3 || shape[2] < 3 {
            continue;
        }
        let height = shape[0] as u32;
        let width = shape[1] as u32;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..shape[0] {
            for x in 0..shape[1] {
                rgba.push(frame[[y, x, 0]]);
                rgba.push(frame[[y, x, 1]]);
                rgba.push(frame[[y, x, 2]]);
                rgba.push(255);
            }
        }
        frames.push(rgba_to_render_image(width, height, rgba)?);
    }
    if frames.is_empty() {
        bail!("video decode produced no frames")
    }
    let fps = 30.0;
    let duration = Duration::from_secs_f32(frames.len() as f32 / fps);
    Ok(LoadedPreviewPayload::Video(VideoPreview {
        _title: title.to_string(),
        frames: Arc::new(frames),
        fps,
        duration,
        current_frame: 0,
        playing: false,
        generation: 0,
    }))
}

#[cfg(not(feature = "ffmpeg-video"))]
fn load_video(path: &Path, _title: &str) -> Result<LoadedPreviewPayload> {
    bail!(
        "video preview support is disabled in this build for {}. Enable the `ffmpeg-video` feature on a machine with FFmpeg/pkg-config available.",
        path.display()
    )
}

fn load_mesh(path: &Path, title: &str) -> Result<LoadedPreviewPayload> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    let (vertices, indices) = match ext.to_ascii_lowercase().as_str() {
        "obj" => load_obj_mesh(path)?,
        "stl" => load_stl_mesh(path)?,
        "gltf" | "glb" => load_gltf_mesh(path)?,
        "fbx" => bail!("FBX parsing is not wired in yet for {}", path.display()),
        _ => bail!("unsupported 3D extension for {}", path.display()),
    };
    Ok(LoadedPreviewPayload::Mesh(MeshPreview {
        _title: title.to_string(),
        vertices,
        indices,
        yaw: 0.5,
        pitch: -0.3,
        distance: 4.0,
        dragging_from: None,
    }))
}

fn load_obj_mesh(path: &Path) -> Result<(Vec<Vec3>, Vec<[usize; 3]>)> {
    let options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };
    let (models, _) = tobj::load_obj(path, &options)?;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for model in models {
        let base = vertices.len();
        for chunk in model.mesh.positions.chunks(3) {
            if let [x, y, z] = chunk {
                vertices.push(Vec3 {
                    x: *x,
                    y: *y,
                    z: *z,
                });
            }
        }
        for face in model.mesh.indices.chunks(3) {
            if let [a, b, c] = face {
                indices.push([base + *a as usize, base + *b as usize, base + *c as usize]);
            }
        }
    }
    Ok(normalize_mesh(vertices, indices))
}

fn load_stl_mesh(path: &Path) -> Result<(Vec<Vec3>, Vec<[usize; 3]>)> {
    let mut file = File::open(path)?;
    let mesh = stl_io::read_stl(&mut file)?;
    let vertices = mesh
        .vertices
        .iter()
        .map(|vertex| Vec3 {
            x: vertex[0],
            y: vertex[1],
            z: vertex[2],
        })
        .collect::<Vec<_>>();
    let indices = mesh
        .faces
        .iter()
        .map(|face| [face.vertices[0], face.vertices[1], face.vertices[2]])
        .collect::<Vec<_>>();
    Ok(normalize_mesh(vertices, indices))
}

fn load_gltf_mesh(path: &Path) -> Result<(Vec<Vec3>, Vec<[usize; 3]>)> {
    let (document, buffers, _) = gltf::import(path)?;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let base = vertices.len();
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(positions) = reader.read_positions() {
                vertices.extend(positions.map(|[x, y, z]| Vec3 { x, y, z }));
            }
            if let Some(read_indices) = reader.read_indices() {
                for triangle in read_indices.into_u32().collect::<Vec<_>>().chunks(3) {
                    if let [a, b, c] = triangle {
                        indices.push([base + *a as usize, base + *b as usize, base + *c as usize]);
                    }
                }
            }
        }
    }
    let _ = easy_gltf::load(path).ok();
    Ok(normalize_mesh(vertices, indices))
}

fn extract_xml_text(xml: &str, tag_name: &[u8]) -> Result<Vec<String>> {
    let mut reader = XmlReader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut values = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(start)) if start.name().as_ref() == tag_name => {
                let text = reader.read_text(start.name())?;
                let text = text.trim().to_string();
                if !text.is_empty() {
                    values.push(text);
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buffer.clear();
    }
    Ok(values)
}

fn rgba_to_render_image(width: u32, height: u32, mut rgba: Vec<u8>) -> Result<Arc<RenderImage>> {
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let buffer = RgbaImage::from_raw(width, height, rgba).context("building RGBA image buffer")?;
    let frame = Frame::from_parts(buffer, 0, 0, Delay::from_numer_denom_ms(0, 1));
    Ok(Arc::new(RenderImage::new(SmallVec::from_vec(vec![frame]))))
}

fn build_waveform(samples: &[f32], buckets: usize) -> Vec<f32> {
    if samples.is_empty() || buckets == 0 {
        return Vec::new();
    }
    let stride = (samples.len() / buckets).max(1);
    (0..buckets)
        .map(|ix| {
            let start = ix * stride;
            let end = ((ix + 1) * stride).min(samples.len());
            samples[start..end]
                .iter()
                .fold(0.0f32, |peak, sample| peak.max(sample.abs()))
        })
        .collect()
}

fn normalize_mesh(vertices: Vec<Vec3>, indices: Vec<[usize; 3]>) -> (Vec<Vec3>, Vec<[usize; 3]>) {
    if vertices.is_empty() {
        return (vertices, indices);
    }
    let mut min = vertices[0];
    let mut max = vertices[0];
    for vertex in &vertices {
        min.x = min.x.min(vertex.x);
        min.y = min.y.min(vertex.y);
        min.z = min.z.min(vertex.z);
        max.x = max.x.max(vertex.x);
        max.y = max.y.max(vertex.y);
        max.z = max.z.max(vertex.z);
    }
    let center = Vec3 {
        x: (min.x + max.x) * 0.5,
        y: (min.y + max.y) * 0.5,
        z: (min.z + max.z) * 0.5,
    };
    let extent = (max.x - min.x)
        .max(max.y - min.y)
        .max(max.z - min.z)
        .max(1.0);
    let normalized = vertices
        .into_iter()
        .map(|vertex| Vec3 {
            x: (vertex.x - center.x) / extent * 2.0,
            y: (vertex.y - center.y) / extent * 2.0,
            z: (vertex.z - center.z) / extent * 2.0,
        })
        .collect();
    (normalized, indices)
}

fn rotate_y(v: Vec3, yaw: f32) -> Vec3 {
    Vec3 {
        x: v.x * yaw.cos() - v.z * yaw.sin(),
        y: v.y,
        z: v.x * yaw.sin() + v.z * yaw.cos(),
    }
}
fn rotate_x(v: Vec3, pitch: f32) -> Vec3 {
    Vec3 {
        x: v.x,
        y: v.y * pitch.cos() - v.z * pitch.sin(),
        z: v.y * pitch.sin() + v.z * pitch.cos(),
    }
}

fn project_point(
    vertex: Option<Vec3>,
    yaw: f32,
    pitch: f32,
    distance: f32,
    center: Point<Pixels>,
) -> Option<Point<Pixels>> {
    let mut v = vertex?;
    v = rotate_y(v, yaw);
    v = rotate_x(v, pitch);
    let depth = v.z + distance;
    if depth <= 0.1 {
        return None;
    }
    let scale = 220.0 / depth;
    Some(point(
        center.x + px(v.x * scale),
        center.y - px(v.y * scale),
    ))
}

fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}
