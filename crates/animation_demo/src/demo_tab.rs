use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString, Task,
    WeakEntity, Window, actions, prelude::*, px,
};
use smallvec::{SmallVec, smallvec};
use ui::{Divider, prelude::*};
use workspace::{
    Workspace, WorkspaceId,
    item::{Item, ItemEvent},
};

use crate::{
    DragConstraint, DragDropPreview, EasingCurve, FridayBorderPreview, HelloGlowPreview,
    MacOsDockPreview, Rgba, ScreenCarouselPreview, SidebarPreview, SpringConfig,
};

actions!(animation_demo, [OpenAnimationDemo]);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        workspace.register_action(|workspace, _: &OpenAnimationDemo, window, cx| {
            open_animation_demo_tab(workspace, window, cx);
        });

        let Some(window) = window else {
            return;
        };

        let workspace = cx.entity().downgrade();
        window.defer(cx, move |window, cx| {
            let Some(workspace) = workspace.upgrade() else {
                return;
            };
            let _ = workspace.update(cx, |workspace, cx| {
                open_animation_demo_tab(workspace, window, cx);
            });
        });
    })
    .detach();
}

fn open_animation_demo_tab(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let existing = workspace
        .active_pane()
        .read(cx)
        .items()
        .find_map(|item| item.downcast::<AnimationDemoTab>());

    if let Some(existing) = existing {
        workspace.activate_item(&existing, true, true, window, cx);
        return;
    }

    let demo_tab = cx.new(|cx| AnimationDemoTab::new(workspace.weak_handle(), cx));
    workspace.add_item_to_active_pane(Box::new(demo_tab), None, true, window, cx);
}

struct AnimationDemoTab {
    friday_preview: Entity<FridayBorderPreview>,
    hello_glow_preview: Entity<HelloGlowPreview>,
    dock_preview: Entity<MacOsDockPreview>,
    carousel_preview: Entity<ScreenCarouselPreview>,
    drag_drop_preview: Entity<DragDropPreview>,
    sidebar_preview: Entity<SidebarPreview>,
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
}

impl AnimationDemoTab {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            friday_preview: cx.new(FridayBorderPreview::new),
            hello_glow_preview: cx.new(HelloGlowPreview::new),
            dock_preview: cx.new(MacOsDockPreview::new),
            carousel_preview: cx.new(ScreenCarouselPreview::new),
            drag_drop_preview: cx.new(DragDropPreview::new),
            sidebar_preview: cx.new(SidebarPreview::new),
            workspace,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<ItemEvent> for AnimationDemoTab {}

impl Focusable for AnimationDemoTab {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for AnimationDemoTab {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Animation Demo".into()
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Animation Demo Opened")
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn can_split(&self) -> bool {
        true
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>> {
        let workspace = self.workspace.clone();
        Task::ready(Some(cx.new(|cx| Self::new(workspace, cx))))
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event)
    }
}

impl Render for AnimationDemoTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.bounds().size;
        let metrics: SmallVec<[(&'static str, String); 4]> = smallvec![
            (
                "Viewport",
                format!(
                    "{:.0}px x {:.0}px",
                    f32::from(viewport.width),
                    f32::from(viewport.height)
                )
            ),
            ("Target FPS", "120 FPS".to_string()),
            ("Frame Budget", "8.33 ms".to_string()),
            ("Preview Surface", "Video + 3D".to_string()),
        ];
        let default_spring = SpringConfig::default_apple();
        let snappy_spring = SpringConfig::snappy();
        let sample_constraint = DragConstraint {
            min_x: -120.0,
            max_x: 120.0,
            min_y: -48.0,
            max_y: 48.0,
            elasticity: 0.15,
        };
        let motion_primitives: SmallVec<[(&'static str, String); 4]> = smallvec![
            (
                "Default spring settle",
                format!("{:.2}s", default_spring.settling_duration())
            ),
            (
                "Snappy response",
                format!(
                    "{:.2}s @ {:.2} damping",
                    snappy_spring.response, snappy_spring.damping_fraction
                )
            ),
            (
                "Bezier midpoint",
                format!("{:.2}", EasingCurve::AppleDefault.evaluate(0.5))
            ),
            (
                "Gesture elasticity",
                format!(
                    "{:.2} within {}px",
                    sample_constraint.elasticity, sample_constraint.max_x as i32
                )
            ),
        ];
        let accent_color = Rgba::from_rgba_u32(0xFF7A3DFF).to_rgba_u32();

        h_flex()
            .id("animation-demo-tab")
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .track_focus(&self.focus_handle)
            .child(
                v_flex()
                    .w(px(280.))
                    .h_full()
                    .p_4()
                    .gap_3()
                    .border_r_1()
                    .border_color(cx.theme().colors().border)
                    .bg(cx.theme().colors().surface_background)
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Headline::new("Animation Demo").size(HeadlineSize::Small))
                            .child(
                                Label::new("Phase 1 workspace for animation validation")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(Divider::horizontal())
                    .child(
                        v_flex()
                            .gap_2()
                            .children(metrics.into_iter().map(|(label, value)| {
                                v_flex()
                                    .gap_0p5()
                                    .child(
                                        Label::new(label)
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    )
                                    .child(Label::new(value).size(LabelSize::Small))
                            })),
                    )
                    .child(Divider::horizontal())
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                Label::new("Upcoming slots")
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            )
                            .child(Label::new("Friday Border"))
                            .child(Label::new("Hello Glow"))
                            .child(Label::new("Sidebar / Carousel / Dock")),
                    ),
            )
            .child(
                div()
                    .id("animation-demo-scroll-area")
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .child(
                    v_flex()
                        .w_full()
                        .p_6()
                        .gap_4()
                        .child(
                            div()
                                .w_full()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().colors().border)
                                .bg(cx.theme().colors().surface_background)
                                .px_4()
                                .py_3()
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    Headline::new("Preview Stage")
                                                        .size(HeadlineSize::XSmall),
                                                )
                                                .child(
                                                    Label::new(
                                                        "Reserved for side-by-side GPUI animation validation.",
                                                    )
                                                    .size(LabelSize::Small)
                                                    .color(Color::Muted),
                                                ),
                                        )
                                        .child(
                                            Label::new(format!(
                                                "{:.0}px x {:.0}px",
                                                f32::from(viewport.width),
                                                f32::from(viewport.height)
                                            ))
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                        ),
                                ),
                        )
                        .child(
                            h_flex()
                                .gap_4()
                                .w_full()
                                .min_h(px(260.0))
                                .child(preview_panel(
                                    "Video Preview",
                                    "Planned stack: gstreamer or ffmpeg-next with hardware-accelerated MP4/WebM/MOV playback.",
                                    cx,
                                ))
                                .child(preview_panel(
                                    "3D Preview",
                                    "Planned stack: wgpu or three-d with realtime OBJ/GLTF/FBX rendering and camera controls.",
                                    cx,
                                )),
                        )
                        .child(
                            h_flex()
                                .gap_4()
                                .w_full()
                                .min_h(px(320.0))
                                .child(self.friday_preview.clone())
                                .child(self.hello_glow_preview.clone()),
                        )
                        .child(
                            h_flex()
                                .gap_4()
                                .w_full()
                                .min_h(px(320.0))
                                .child(self.sidebar_preview.clone())
                                .child(self.carousel_preview.clone()),
                        )
                        .child(self.dock_preview.clone())
                        .child(self.drag_drop_preview.clone())
                        .child(
                            div()
                                .w_full()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().colors().border_variant)
                                .bg(cx.theme().colors().element_background)
                                .p_4()
                                .child(
                                    Label::new(
                                        "Use this tab as the central verification surface for animation timing, dimensions, and performance as each phase lands.",
                                    )
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().colors().border)
                                .bg(cx.theme().colors().surface_background)
                                .p_4()
                                .child(
                                    v_flex()
                                        .gap_2()
                                        .child(
                                            Headline::new("Phase 2 Motion Toolkit")
                                                .size(HeadlineSize::XSmall),
                                        )
                                        .child(
                                            Label::new(format!(
                                                "Spring, easing, animator, transition, and gesture primitives are now wired into the crate. Accent seed: #{accent_color:08X}"
                                            ))
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                        )
                                        .children(motion_primitives.into_iter().map(|(label, value)| {
                                            h_flex()
                                                .justify_between()
                                                .gap_3()
                                                .child(
                                                    Label::new(label)
                                                        .size(LabelSize::XSmall)
                                                        .color(Color::Muted),
                                                )
                                                .child(Label::new(value).size(LabelSize::Small))
                                        })),
                                ),
                        ),
                    ),
            )
    }
}

fn preview_panel(title: &'static str, body: &'static str, cx: &App) -> impl IntoElement + use<> {
    v_flex()
        .flex_1()
        .h_full()
        .justify_between()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border)
        .bg(cx.theme().colors().surface_background)
        .p_5()
        .child(
            v_flex()
                .gap_1()
                .child(Headline::new(title).size(HeadlineSize::XSmall))
                .child(Label::new(body).size(LabelSize::Small).color(Color::Muted)),
        )
        .child(
            div()
                .w_full()
                .flex_1()
                .min_h(px(220.))
                .mt_4()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border_variant)
                .bg(cx.theme().colors().element_background)
                .items_center()
                .justify_center()
                .child(
                    Label::new("Preview surface")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
}
