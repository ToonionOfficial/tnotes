use gpui::*;
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconName {
    Folder,
    FolderOpen,
    FileText,
    Search,
    Settings,
    Plus,
    ChevronRight,
    ChevronDown,
    Star,
    Check,
    Trash2,
    PanelLeft,
    X,
    RefreshCw,
}

impl IconName {
    pub fn path(self) -> SharedString {
        match self {
            Self::Folder => "icons/folder.svg",
            Self::FolderOpen => "icons/folder-open.svg",
            Self::FileText => "icons/file-text.svg",
            Self::Search => "icons/search.svg",
            Self::Settings => "icons/settings.svg",
            Self::Plus => "icons/plus.svg",
            Self::ChevronRight => "icons/chevron-right.svg",
            Self::ChevronDown => "icons/chevron-down.svg",
            Self::Star => "icons/star.svg",
            Self::Check => "icons/check.svg",
            Self::Trash2 => "icons/trash-2.svg",
            Self::PanelLeft => "icons/panel-left.svg",
            Self::X => "icons/x.svg",
            Self::RefreshCw => "icons/refresh-cw.svg",
        }
        .into()
    }
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: Pixels,
    color: Option<Hsla>,
}

#[allow(dead_code)]
impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: px(16.),
            color: None,
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or_else(|| cx.theme().foreground);
        svg()
            .path(self.name.path())
            .size(self.size)
            .text_color(color)
    }
}
