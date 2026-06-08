#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClipboardOperationMode {
    Copy,
    Move,
}

impl ClipboardOperationMode {
    pub(crate) fn ready_label(self) -> &'static str {
        match self {
            Self::Copy => "Ready to copy",
            Self::Move => "Ready to move",
        }
    }

    pub(crate) fn paste_tooltip(self) -> &'static str {
        match self {
            Self::Copy => "Copy clipboard items here",
            Self::Move => "Move clipboard items here",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ClipboardOperationSummary {
    pub mode: ClipboardOperationMode,
    pub item_count: usize,
}

impl ClipboardOperationSummary {
    pub(crate) fn new(mode: ClipboardOperationMode, item_count: usize) -> Option<Self> {
        (item_count > 0).then_some(Self { mode, item_count })
    }

    pub(crate) fn status_label(self) -> String {
        format!("{} {}", self.mode.ready_label(), self.item_count_label())
    }

    pub(crate) fn item_count_label(self) -> String {
        if self.item_count == 1 {
            "1 item".to_string()
        } else {
            format!("{} items", self.item_count)
        }
    }
}
