use agent_settings::{AgentProfile, DxAiProfileKind};
use ui::{DxUiIcon, IconName, dx_icon};

#[derive(Clone, Copy)]
pub(super) enum ComposerProfileKind {
    Ask,
    Agents,
    Media,
    Search,
    Study,
}

#[derive(Clone, Copy)]
pub(super) enum ComposerSlotControlState {
    DisplayOnly,
    BackendPending,
}

#[derive(Clone, Copy)]
pub(super) struct ComposerSlotContract {
    pub(super) backing: &'static str,
    pub(super) control_state: ComposerSlotControlState,
}

#[derive(Clone, Copy)]
pub(super) struct ComposerOptionSlot {
    pub(super) id: &'static str,
    pub(super) icon: ComposerOptionIcon,
    pub(super) label: &'static str,
    pub(super) tooltip: &'static str,
    pub(super) contract: ComposerSlotContract,
    pub(super) options: &'static [ComposerOptionEntry],
}

#[derive(Clone, Copy)]
pub(super) struct ComposerOptionEntry {
    pub(super) id: &'static str,
    pub(super) icon: ComposerOptionIcon,
    pub(super) label: &'static str,
    pub(super) detail: &'static str,
}

#[derive(Clone, Copy)]
pub(super) enum ComposerOptionIcon {
    Static(IconName),
    Dx(DxUiIcon),
}

impl ComposerOptionIcon {
    pub(super) fn icon_name(self) -> IconName {
        match self {
            ComposerOptionIcon::Static(icon) => icon,
            ComposerOptionIcon::Dx(icon) => dx_icon(icon),
        }
    }
}

const fn composer_icon(icon: IconName) -> ComposerOptionIcon {
    ComposerOptionIcon::Static(icon)
}

const fn option(
    id: &'static str,
    icon: ComposerOptionIcon,
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
    icon: ComposerOptionIcon,
    label: &'static str,
    tooltip: &'static str,
    contract: ComposerSlotContract,
    options: &'static [ComposerOptionEntry],
) -> ComposerOptionSlot {
    ComposerOptionSlot {
        id,
        icon,
        label,
        tooltip,
        contract,
        options,
    }
}

const ASK_CONTRACT: ComposerSlotContract = ComposerSlotContract {
    backing: "Ask profile",
    control_state: ComposerSlotControlState::DisplayOnly,
};

const AGENTS_CONTRACT: ComposerSlotContract = ComposerSlotContract {
    backing: "Agent tools",
    control_state: ComposerSlotControlState::DisplayOnly,
};

const SEARCH_CONTRACT: ComposerSlotContract = ComposerSlotContract {
    backing: "DX MetaSearch and Web Preview evidence",
    control_state: ComposerSlotControlState::DisplayOnly,
};

const STUDY_CONTRACT: ComposerSlotContract = ComposerSlotContract {
    backing: "Study source receipts",
    control_state: ComposerSlotControlState::DisplayOnly,
};

const MEDIA_RECEIPT_CONTRACT: ComposerSlotContract = ComposerSlotContract {
    backing: "Media receipts",
    control_state: ComposerSlotControlState::DisplayOnly,
};

const MEDIA_PROVIDER_CONTRACT: ComposerSlotContract = ComposerSlotContract {
    backing: "Provider controls",
    control_state: ComposerSlotControlState::BackendPending,
};

static ASK_MODEL_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "ask-models-multiple",
        composer_icon(IconName::AiOpenAi),
        "Multiple answers",
        "Compare several model responses before choosing a direction.",
    ),
    option(
        "ask-models-consensus",
        composer_icon(IconName::CheckDouble),
        "Consensus pass",
        "Prefer agreement across strong models for higher confidence.",
    ),
    option(
        "ask-models-single",
        composer_icon(IconName::ZedAssistant),
        "Single answer",
        "Use the selected model for one focused answer.",
    ),
];

static ASK_SPEED_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "ask-speed-fast",
        composer_icon(IconName::FastForward),
        "Fast",
        "Prioritize a short, low-latency answer.",
    ),
    option(
        "ask-speed-balanced",
        composer_icon(IconName::SignalMedium),
        "Balanced",
        "Balance speed, context, and explanation depth.",
    ),
    option(
        "ask-speed-source-aware",
        ComposerOptionIcon::Dx(DxUiIcon::Search),
        "Source aware",
        "Prefer answers that can cite concrete local or web evidence.",
    ),
];

static ASK_REASON_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "ask-reasoning-light",
        composer_icon(IconName::ThinkingModeOff),
        "Light",
        "Use minimal reasoning for simple questions.",
    ),
    option(
        "ask-reasoning-deep",
        composer_icon(IconName::ThinkingMode),
        "Deep",
        "Spend more reasoning on tradeoffs and correctness.",
    ),
    option(
        "ask-reasoning-exhaustive",
        composer_icon(IconName::Crosshair),
        "Exhaustive",
        "Use the strongest reasoning path for hard decisions.",
    ),
];

static AGENT_WORK_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "agents-work-workspace-edits",
        composer_icon(IconName::ToolHammer),
        "Workspace edits",
        "Let the assistant change files through the normal tool path.",
    ),
    option(
        "agents-work-queued",
        composer_icon(IconName::QueueMessage),
        "Queued work",
        "Stage follow-up prompts while the current turn is running.",
    ),
    option(
        "agents-work-diagnostics",
        composer_icon(IconName::ToolDiagnostics),
        "Diagnostics",
        "Prefer inspection and focused checks before runtime proof.",
    ),
];

static AGENT_PLAN_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "agents-plan-mode",
        composer_icon(IconName::ListTodo),
        "Plan mode",
        "Turn intent into a clear implementation path.",
    ),
    option(
        "agents-plan-checkpoints",
        composer_icon(IconName::TodoProgress),
        "Checkpoints",
        "Keep long work split into visible, reviewable milestones.",
    ),
    option(
        "agents-plan-handoff",
        composer_icon(IconName::FileTextOutlined),
        "Handoff",
        "Preserve decisions and verification for the next pass.",
    ),
];

static AGENT_WORKER_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "agents-workers-parallel",
        composer_icon(IconName::UserGroup),
        "Parallel workers",
        "Split independent work into bounded lanes when available.",
    ),
    option(
        "agents-workers-review",
        composer_icon(IconName::Eye),
        "Review lane",
        "Use a focused pass to catch regressions and unverified wiring.",
    ),
    option(
        "agents-workers-handoff",
        composer_icon(IconName::GitBranch),
        "Lane handoff",
        "Keep branch and ownership boundaries explicit.",
    ),
];

static MEDIA_OUTPUT_OPTIONS: [ComposerOptionEntry; 6] = [
    option(
        "media-output-image",
        ComposerOptionIcon::Dx(DxUiIcon::Media),
        "Image",
        "Tune prompt, ratio, and quality for still images.",
    ),
    option(
        "media-output-video",
        composer_icon(IconName::Screen),
        "Video",
        "Tune scene length, motion, and preview frames.",
    ),
    option(
        "media-output-audio",
        composer_icon(IconName::AudioOn),
        "Audio",
        "Tune voice, music, timing, and transcript details.",
    ),
    option(
        "media-output-music",
        composer_icon(IconName::PlayOutlined),
        "Music",
        "Tune music style, structure, stems, and loops.",
    ),
    option(
        "media-output-3d",
        composer_icon(IconName::Box),
        "3D",
        "Tune model, material, scene, and export constraints.",
    ),
    option(
        "media-output-docs",
        composer_icon(IconName::FileDoc),
        "Docs",
        "Tune document format, source inputs, and review path.",
    ),
];

static MEDIA_FRAME_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-frame-square",
        composer_icon(IconName::SquareDot),
        "Square",
        "Use square output for cards, posts, and thumbnails.",
    ),
    option(
        "media-frame-wide",
        composer_icon(IconName::Screen),
        "Wide",
        "Use wide output for previews, videos, and hero media.",
    ),
    option(
        "media-frame-tall",
        composer_icon(IconName::ExpandVertical),
        "Tall",
        "Use vertical output for mobile and story formats.",
    ),
];

static MEDIA_TIME_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-time-short",
        composer_icon(IconName::Clock),
        "Short",
        "Generate compact clips or samples.",
    ),
    option(
        "media-time-loop",
        composer_icon(IconName::HistoryRerun),
        "Loop",
        "Prefer seamless motion or audio loops.",
    ),
    option(
        "media-time-scene",
        composer_icon(IconName::CountdownTimer),
        "Scene",
        "Use a longer scene with richer timing controls.",
    ),
];

static MEDIA_QUALITY_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-quality-draft",
        composer_icon(IconName::Pencil),
        "Draft",
        "Explore quickly before spending more generation budget.",
    ),
    option(
        "media-quality-high",
        composer_icon(IconName::Sparkle),
        "High",
        "Raise quality for assets that may ship.",
    ),
    option(
        "media-quality-production",
        composer_icon(IconName::CheckDouble),
        "Production",
        "Prefer final-pass output and stricter review.",
    ),
];

static MEDIA_PROVIDER_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "media-provider-readiness",
        composer_icon(IconName::Warning),
        "Readiness",
        "Check credentials, local runners, and provider health before generation.",
    ),
    option(
        "media-provider-receipts",
        composer_icon(IconName::FileTextOutlined),
        "Receipts",
        "Use approved media plan and runner receipts before execution.",
    ),
    option(
        "media-provider-budget",
        ComposerOptionIcon::Dx(DxUiIcon::Gateway),
        "Budget",
        "Keep provider cost, quality, and safety gates explicit.",
    ),
];

static SEARCH_SCOPE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "search-scope-web",
        composer_icon(IconName::Public),
        "Web",
        "Search current public sources.",
    ),
    option(
        "search-scope-workspace",
        composer_icon(IconName::FileTree),
        "Workspace",
        "Search local project context first.",
    ),
    option(
        "search-scope-both",
        composer_icon(IconName::Blocks),
        "Both",
        "Blend web and workspace evidence.",
    ),
];

static SEARCH_FRESHNESS_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "search-freshness-latest",
        composer_icon(IconName::Clock),
        "Latest",
        "Prefer fresh sources when the topic may have changed.",
    ),
    option(
        "search-freshness-stable",
        composer_icon(IconName::Library),
        "Stable",
        "Prefer canonical documentation and durable references.",
    ),
    option(
        "search-freshness-archive",
        composer_icon(IconName::Archive),
        "Archive",
        "Include older records when history matters.",
    ),
];

static SEARCH_SOURCE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "search-sources-primary",
        composer_icon(IconName::Check),
        "Primary",
        "Favor official docs, papers, and first-party sources.",
    ),
    option(
        "search-sources-community",
        composer_icon(IconName::UserGroup),
        "Community",
        "Include reputable community reports when useful.",
    ),
    option(
        "search-sources-media",
        ComposerOptionIcon::Dx(DxUiIcon::Media),
        "Media",
        "Include image, video, or visual source results.",
    ),
];

static STUDY_SOURCE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "study-sources-attached",
        composer_icon(IconName::Book),
        "Study sources",
        "Use attached notes, docs, and saved material.",
    ),
    option(
        "study-sources-extracts",
        composer_icon(IconName::FileTextOutlined),
        "Extracts",
        "Pull key passages into the study flow.",
    ),
    option(
        "study-sources-tables",
        composer_icon(IconName::DatabaseZap),
        "Tables",
        "Organize facts into structured study tables.",
    ),
];

static STUDY_NOTE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "study-notes-summary",
        composer_icon(IconName::Notepad),
        "Summary",
        "Condense sources into clean notes.",
    ),
    option(
        "study-notes-outline",
        composer_icon(IconName::ListTree),
        "Outline",
        "Build a lesson structure from the source set.",
    ),
    option(
        "study-notes-citations",
        composer_icon(IconName::Quote),
        "Citations",
        "Keep important references visible while studying.",
    ),
];

static STUDY_PRACTICE_OPTIONS: [ComposerOptionEntry; 3] = [
    option(
        "study-practice-drills",
        composer_icon(IconName::Crosshair),
        "Practice",
        "Turn material into questions and drills.",
    ),
    option(
        "study-practice-recall",
        composer_icon(IconName::TodoComplete),
        "Recall",
        "Check retention with short active-recall prompts.",
    ),
    option(
        "study-practice-progress",
        composer_icon(IconName::TodoProgress),
        "Progress",
        "Track what is understood and what needs review.",
    ),
];

static ASK_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "ask-models",
        composer_icon(IconName::AiOpenAi),
        "Models",
        "View model comparison guidance for Ask",
        ASK_CONTRACT,
        &ASK_MODEL_OPTIONS,
    ),
    slot(
        "ask-speed",
        composer_icon(IconName::FastForward),
        "Speed",
        "View response speed guidance for Ask",
        ASK_CONTRACT,
        &ASK_SPEED_OPTIONS,
    ),
    slot(
        "ask-reasoning",
        composer_icon(IconName::ThinkingMode),
        "Reasoning",
        "View reasoning depth guidance for Ask",
        ASK_CONTRACT,
        &ASK_REASON_OPTIONS,
    ),
];

static AGENTS_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "agents-work",
        composer_icon(IconName::ZedAgent),
        "Work",
        "View workspace-work guidance for Agents",
        AGENTS_CONTRACT,
        &AGENT_WORK_OPTIONS,
    ),
    slot(
        "agents-plan",
        composer_icon(IconName::ListTodo),
        "Plan",
        "View planning guidance for Agents",
        AGENTS_CONTRACT,
        &AGENT_PLAN_OPTIONS,
    ),
    slot(
        "agents-workers",
        composer_icon(IconName::UserGroup),
        "Workers",
        "View worker-lane guidance for Agents",
        AGENTS_CONTRACT,
        &AGENT_WORKER_OPTIONS,
    ),
];

static MEDIA_COMPOSER_SLOTS: [ComposerOptionSlot; 5] = [
    slot(
        "media-output",
        ComposerOptionIcon::Dx(DxUiIcon::Media),
        "Output",
        "View Media output guidance",
        MEDIA_PROVIDER_CONTRACT,
        &MEDIA_OUTPUT_OPTIONS,
    ),
    slot(
        "media-frame",
        composer_icon(IconName::Screen),
        "Frame",
        "View Media frame guidance",
        MEDIA_PROVIDER_CONTRACT,
        &MEDIA_FRAME_OPTIONS,
    ),
    slot(
        "media-time",
        composer_icon(IconName::Clock),
        "Time",
        "View Media timing guidance",
        MEDIA_PROVIDER_CONTRACT,
        &MEDIA_TIME_OPTIONS,
    ),
    slot(
        "media-quality",
        ComposerOptionIcon::Dx(DxUiIcon::Settings),
        "Quality",
        "View Media quality guidance",
        MEDIA_RECEIPT_CONTRACT,
        &MEDIA_QUALITY_OPTIONS,
    ),
    slot(
        "media-provider",
        ComposerOptionIcon::Dx(DxUiIcon::Gateway),
        "Provider",
        "View Media provider readiness",
        MEDIA_PROVIDER_CONTRACT,
        &MEDIA_PROVIDER_OPTIONS,
    ),
];

static SEARCH_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "search-scope",
        ComposerOptionIcon::Dx(DxUiIcon::Search),
        "Scope",
        "View Search scope guidance",
        SEARCH_CONTRACT,
        &SEARCH_SCOPE_OPTIONS,
    ),
    slot(
        "search-freshness",
        composer_icon(IconName::Clock),
        "Freshness",
        "View Search freshness guidance",
        SEARCH_CONTRACT,
        &SEARCH_FRESHNESS_OPTIONS,
    ),
    slot(
        "search-sources",
        composer_icon(IconName::Public),
        "Sources",
        "View Search source guidance",
        SEARCH_CONTRACT,
        &SEARCH_SOURCE_OPTIONS,
    ),
];

static STUDY_COMPOSER_SLOTS: [ComposerOptionSlot; 3] = [
    slot(
        "study-sources",
        composer_icon(IconName::Book),
        "Sources",
        "View Study source guidance",
        STUDY_CONTRACT,
        &STUDY_SOURCE_OPTIONS,
    ),
    slot(
        "study-notes",
        composer_icon(IconName::Notepad),
        "Notes",
        "View Study note guidance",
        STUDY_CONTRACT,
        &STUDY_NOTE_OPTIONS,
    ),
    slot(
        "study-practice",
        composer_icon(IconName::Crosshair),
        "Practice",
        "View Study practice guidance",
        STUDY_CONTRACT,
        &STUDY_PRACTICE_OPTIONS,
    ),
];

impl ComposerProfileKind {
    pub(super) fn for_profile_id(profile_id: &str) -> Option<Self> {
        let metadata = AgentProfile::dx_builtin_metadata_for_id(profile_id)?;
        Some(Self::from_dx_kind(metadata.kind))
    }

    fn from_dx_kind(kind: DxAiProfileKind) -> Self {
        match kind {
            DxAiProfileKind::Ask => ComposerProfileKind::Ask,
            DxAiProfileKind::Agents => ComposerProfileKind::Agents,
            DxAiProfileKind::Search => ComposerProfileKind::Search,
            DxAiProfileKind::Study => ComposerProfileKind::Study,
            DxAiProfileKind::Media => ComposerProfileKind::Media,
        }
    }

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
