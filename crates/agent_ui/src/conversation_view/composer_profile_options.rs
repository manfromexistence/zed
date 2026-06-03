use ui::IconName;

#[derive(Clone, Copy)]
pub(super) enum ComposerProfileKind {
    Ask,
    Agents,
    Media,
    Search,
    Study,
}

#[derive(Clone, Copy)]
pub(super) struct ComposerOptionSlot {
    pub(super) id: &'static str,
    pub(super) icon: IconName,
    pub(super) label: &'static str,
    pub(super) tooltip: &'static str,
    pub(super) options: &'static [ComposerOptionEntry],
}

#[derive(Clone, Copy)]
pub(super) struct ComposerOptionEntry {
    pub(super) id: &'static str,
    pub(super) icon: IconName,
    pub(super) label: &'static str,
    pub(super) detail: &'static str,
}

const fn option(
    id: &'static str,
    icon: IconName,
    label: &'static str,
    detail: &'static str,
) -> ComposerOptionEntry {
    ComposerOptionEntry {
        id,
        icon,
        label,
        detail,
    }
}

const fn slot(
    id: &'static str,
    icon: IconName,
    label: &'static str,
    tooltip: &'static str,
    options: &'static [ComposerOptionEntry],
) -> ComposerOptionSlot {
    ComposerOptionSlot {
        id,
        icon,
        label,
        tooltip,
        options,
    }
}

static ASK_MODEL_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "ask-models-multiple",
        IconName::AiOpenAi,
        "Multiple answers",
        "Compare several model responses before choosing a direction.",
    ),
    option(
        "ask-models-consensus",
        IconName::CheckDouble,
        "Consensus pass",
        "Prefer agreement across strong models for higher confidence.",
    ),
    option(
        "ask-models-single",
        IconName::ZedAssistant,
        "Single answer",
        "Use the selected model for one focused answer.",
    ),
];

static ASK_SPEED_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "ask-speed-fast",
        IconName::FastForward,
        "Fast",
        "Prioritize a short, low-latency answer.",
    ),
    option(
        "ask-speed-balanced",
        IconName::SignalMedium,
        "Balanced",
        "Balance speed, context, and explanation depth.",
    ),
    option(
        "ask-speed-source-aware",
        IconName::ToolSearch,
        "Source aware",
        "Prefer answers that can cite concrete local or web evidence.",
    ),
];

static ASK_REASON_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "ask-reasoning-light",
        IconName::ThinkingModeOff,
        "Light",
        "Use minimal reasoning for simple questions.",
    ),
    option(
        "ask-reasoning-deep",
        IconName::ThinkingMode,
        "Deep",
        "Spend more reasoning on tradeoffs and correctness.",
    ),
    option(
        "ask-reasoning-exhaustive",
        IconName::Crosshair,
        "Exhaustive",
        "Use the strongest reasoning path for hard decisions.",
    ),
];

static AGENT_WORK_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "agents-work-workspace-edits",
        IconName::ToolHammer,
        "Workspace edits",
        "Let the assistant change files through the normal tool path.",
    ),
    option(
        "agents-work-queued",
        IconName::QueueMessage,
        "Queued work",
        "Stage follow-up prompts while the current turn is running.",
    ),
    option(
        "agents-work-diagnostics",
        IconName::ToolDiagnostics,
        "Diagnostics",
        "Prefer inspection and focused checks before runtime proof.",
    ),
];

static AGENT_PLAN_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "agents-plan-mode",
        IconName::ListTodo,
        "Plan mode",
        "Turn intent into a clear implementation path.",
    ),
    option(
        "agents-plan-checkpoints",
        IconName::TodoProgress,
        "Checkpoints",
        "Keep long work split into visible, reviewable milestones.",
    ),
    option(
        "agents-plan-handoff",
        IconName::FileTextOutlined,
        "Handoff",
        "Preserve decisions and verification for the next pass.",
    ),
];

static AGENT_WORKER_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "agents-workers-parallel",
        IconName::UserGroup,
        "Parallel workers",
        "Split independent work into bounded lanes when available.",
    ),
    option(
        "agents-workers-review",
        IconName::Eye,
        "Review lane",
        "Use a focused pass to catch regressions and dummy wiring.",
    ),
    option(
        "agents-workers-handoff",
        IconName::GitBranch,
        "Lane handoff",
        "Keep branch and ownership boundaries explicit.",
    ),
];

static MEDIA_OUTPUT_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-output-image",
        IconName::Image,
        "Image",
        "Tune prompt, ratio, and quality for still images.",
    ),
    option(
        "media-output-video",
        IconName::Screen,
        "Video",
        "Tune scene length, motion, and preview frames.",
    ),
    option(
        "media-output-audio",
        IconName::AudioOn,
        "Audio",
        "Tune voice, music, timing, and transcript details.",
    ),
];

static MEDIA_FRAME_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-frame-square",
        IconName::SquareDot,
        "Square",
        "Use square output for cards, posts, and thumbnails.",
    ),
    option(
        "media-frame-wide",
        IconName::Screen,
        "Wide",
        "Use wide output for previews, videos, and hero media.",
    ),
    option(
        "media-frame-tall",
        IconName::ExpandVertical,
        "Tall",
        "Use vertical output for mobile and story formats.",
    ),
];

static MEDIA_TIME_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-time-short",
        IconName::Clock,
        "Short",
        "Generate compact clips or samples.",
    ),
    option(
        "media-time-loop",
        IconName::HistoryRerun,
        "Loop",
        "Prefer seamless motion or audio loops.",
    ),
    option(
        "media-time-scene",
        IconName::CountdownTimer,
        "Scene",
        "Use a longer scene with richer timing controls.",
    ),
];

static MEDIA_QUALITY_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-quality-draft",
        IconName::Pencil,
        "Draft",
        "Explore quickly before spending more generation budget.",
    ),
    option(
        "media-quality-high",
        IconName::Sparkle,
        "High",
        "Raise quality for assets that may ship.",
    ),
    option(
        "media-quality-production",
        IconName::CheckDouble,
        "Production",
        "Prefer final-pass output and stricter review.",
    ),
];

static SEARCH_SCOPE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "search-scope-web",
        IconName::Public,
        "Web",
        "Search current public sources.",
    ),
    option(
        "search-scope-workspace",
        IconName::FileTree,
        "Workspace",
        "Search local project context first.",
    ),
    option(
        "search-scope-both",
        IconName::Blocks,
        "Both",
        "Blend web and workspace evidence.",
    ),
];

static SEARCH_FRESHNESS_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "search-freshness-latest",
        IconName::Clock,
        "Latest",
        "Prefer fresh sources when the topic may have changed.",
    ),
    option(
        "search-freshness-stable",
        IconName::Library,
        "Stable",
        "Prefer canonical documentation and durable references.",
    ),
    option(
        "search-freshness-archive",
        IconName::Archive,
        "Archive",
        "Include older records when history matters.",
    ),
];

static SEARCH_SOURCE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "search-sources-primary",
        IconName::Check,
        "Primary",
        "Favor official docs, papers, and first-party sources.",
    ),
    option(
        "search-sources-community",
        IconName::UserGroup,
        "Community",
        "Include reputable community reports when useful.",
    ),
    option(
        "search-sources-media",
        IconName::Image,
        "Media",
        "Include image, video, or visual source results.",
    ),
];

static STUDY_SOURCE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "study-sources-notebook",
        IconName::Book,
        "Notebook sources",
        "Use attached notes, docs, and saved material.",
    ),
    option(
        "study-sources-extracts",
        IconName::FileTextOutlined,
        "Extracts",
        "Pull key passages into the study flow.",
    ),
    option(
        "study-sources-tables",
        IconName::DatabaseZap,
        "Tables",
        "Organize facts into structured study tables.",
    ),
];

static STUDY_NOTE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "study-notes-summary",
        IconName::Notepad,
        "Summary",
        "Condense sources into clean notes.",
    ),
    option(
        "study-notes-outline",
        IconName::ListTree,
        "Outline",
        "Build a lesson structure from the source set.",
    ),
    option(
        "study-notes-citations",
        IconName::Quote,
        "Citations",
        "Keep important references visible while studying.",
    ),
];

static STUDY_PRACTICE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "study-practice-drills",
        IconName::Crosshair,
        "Practice",
        "Turn material into questions and drills.",
    ),
    option(
        "study-practice-recall",
        IconName::TodoComplete,
        "Recall",
        "Check retention with short active-recall prompts.",
    ),
    option(
        "study-practice-progress",
        IconName::TodoProgress,
        "Progress",
        "Track what is understood and what needs review.",
    ),
];

static ASK_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "ask-models",
        IconName::AiOpenAi,
        "Models",
        "Choose how many model answers Ask should compare",
        &ASK_MODEL_OPTIONS,
    ),
    slot(
        "ask-speed",
        IconName::FastForward,
        "Speed",
        "Choose the Ask response speed preference",
        &ASK_SPEED_OPTIONS,
    ),
    slot(
        "ask-reasoning",
        IconName::ThinkingMode,
        "Reasoning",
        "Choose the Ask reasoning depth",
        &ASK_REASON_OPTIONS,
    ),
];

static AGENTS_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "agents-work",
        IconName::ZedAgent,
        "Work",
        "Choose how Agents should handle workspace work",
        &AGENT_WORK_OPTIONS,
    ),
    slot(
        "agents-plan",
        IconName::ListTodo,
        "Plan",
        "Choose the Agents planning preference",
        &AGENT_PLAN_OPTIONS,
    ),
    slot(
        "agents-workers",
        IconName::UserGroup,
        "Workers",
        "Choose the Agents worker preference",
        &AGENT_WORKER_OPTIONS,
    ),
];

static MEDIA_COMPOSER_SLOTS: [ComposerOptionSlot; 4] = [
    slot(
        "media-output",
        IconName::Image,
        "Output",
        "Choose the Media output family",
        &MEDIA_OUTPUT_OPTIONS,
    ),
    slot(
        "media-frame",
        IconName::Screen,
        "Frame",
        "Choose ratio and frame shape for Media",
        &MEDIA_FRAME_OPTIONS,
    ),
    slot(
        "media-time",
        IconName::Clock,
        "Time",
        "Choose duration behavior for Media",
        &MEDIA_TIME_OPTIONS,
    ),
    slot(
        "media-quality",
        IconName::Sliders,
        "Quality",
        "Choose the Media generation quality target",
        &MEDIA_QUALITY_OPTIONS,
    ),
];

static SEARCH_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "search-scope",
        IconName::MagnifyingGlass,
        "Scope",
        "Choose where Search should look",
        &SEARCH_SCOPE_OPTIONS,
    ),
    slot(
        "search-freshness",
        IconName::Clock,
        "Freshness",
        "Choose the Search freshness preference",
        &SEARCH_FRESHNESS_OPTIONS,
    ),
    slot(
        "search-sources",
        IconName::Public,
        "Sources",
        "Choose the Search source mix",
        &SEARCH_SOURCE_OPTIONS,
    ),
];

static STUDY_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "study-sources",
        IconName::Book,
        "Sources",
        "Choose the Study source behavior",
        &STUDY_SOURCE_OPTIONS,
    ),
    slot(
        "study-notes",
        IconName::Notepad,
        "Notes",
        "Choose the Study note style",
        &STUDY_NOTE_OPTIONS,
    ),
    slot(
        "study-practice",
        IconName::Crosshair,
        "Practice",
        "Choose the Study practice behavior",
        &STUDY_PRACTICE_OPTIONS,
    ),
];

impl ComposerProfileKind {
    pub(super) fn slots(self) -> &'static [ComposerOptionSlot] {
        match self {
            ComposerProfileKind::Ask => &ASK_COMPOSER_SLOTS,
            ComposerProfileKind::Agents => &AGENTS_COMPOSER_SLOTS,
            ComposerProfileKind::Media => &MEDIA_COMPOSER_SLOTS,
            ComposerProfileKind::Search => &SEARCH_COMPOSER_SLOTS,
            ComposerProfileKind::Study => &STUDY_COMPOSER_SLOTS,
        }
    }
}
