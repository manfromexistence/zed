use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreviewKind {
    Video,
    Model3d,
    Latex,
    Pdf,
    Docx,
    Spreadsheet,
    Presentation,
    Markdown,
    Svg,
    Audio,
}

#[derive(Clone, Copy, Debug)]
pub struct PreviewHandler {
    pub kind: PreviewKind,
    pub label: &'static str,
    pub extensions: &'static [&'static str],
}

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "webm", "avi", "mov"];
const MODEL_EXTENSIONS: &[&str] = &["gltf", "glb", "obj", "fbx", "stl"];
const LATEX_EXTENSIONS: &[&str] = &["tex", "latex"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];
const DOCX_EXTENSIONS: &[&str] = &["docx"];
const SPREADSHEET_EXTENSIONS: &[&str] = &["xlsx", "csv"];
const PRESENTATION_EXTENSIONS: &[&str] = &["pptx"];
const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown"];
const SVG_EXTENSIONS: &[&str] = &["svg"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "ogg", "flac", "aac"];

pub const PREVIEW_HANDLERS: &[PreviewHandler] = &[
    PreviewHandler {
        kind: PreviewKind::Video,
        label: "Video",
        extensions: VIDEO_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Model3d,
        label: "3D Model",
        extensions: MODEL_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Latex,
        label: "LaTeX",
        extensions: LATEX_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Pdf,
        label: "PDF",
        extensions: PDF_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Docx,
        label: "Word",
        extensions: DOCX_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Spreadsheet,
        label: "Spreadsheet",
        extensions: SPREADSHEET_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Presentation,
        label: "Presentation",
        extensions: PRESENTATION_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Markdown,
        label: "Markdown",
        extensions: MARKDOWN_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Svg,
        label: "SVG",
        extensions: SVG_EXTENSIONS,
    },
    PreviewHandler {
        kind: PreviewKind::Audio,
        label: "Audio",
        extensions: AUDIO_EXTENSIONS,
    },
];

pub fn preview_handler_for_extension(ext: &str) -> Option<PreviewHandler> {
    PREVIEW_HANDLERS.iter().copied().find(|handler| {
        handler
            .extensions
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(ext))
    })
}

pub fn preview_handler_for_path(path: &Path) -> Option<PreviewHandler> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(preview_handler_for_extension)
}

impl PreviewKind {
    pub fn label(self) -> &'static str {
        PREVIEW_HANDLERS
            .iter()
            .find(|handler| handler.kind == self)
            .map(|handler| handler.label)
            .unwrap_or("Preview")
    }
}
