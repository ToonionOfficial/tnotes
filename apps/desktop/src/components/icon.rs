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
    Pencil,
    Copy,
    FolderPlus,
    FilePlus,
    PanelLeft,
    X,
    RefreshCw,
    Briefcase,
    Lightbulb,
    Rocket,
    Target,
    GraduationCap,
    Palette,
    Home,
    Wallet,
    Heart,
    Bookmark,
    Code,
    Music,
    Zap,
    ShoppingCart,
    ArrowLeft,
    ArrowRight,
    User,
    Database,
    Keyboard,
    CircleInfo,
}

#[allow(dead_code)]
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
            Self::Pencil => "icons/pencil.svg",
            Self::Copy => "icons/copy.svg",
            Self::FolderPlus => "icons/folder-plus.svg",
            Self::FilePlus => "icons/file-plus.svg",
            Self::PanelLeft => "icons/panel-left.svg",
            Self::X => "icons/x.svg",
            Self::RefreshCw => "icons/refresh-cw.svg",
            Self::Briefcase => "icons/briefcase.svg",
            Self::Lightbulb => "icons/lightbulb.svg",
            Self::Rocket => "icons/rocket.svg",
            Self::Target => "icons/target.svg",
            Self::GraduationCap => "icons/graduation-cap.svg",
            Self::Palette => "icons/palette.svg",
            Self::Home => "icons/home.svg",
            Self::Wallet => "icons/wallet.svg",
            Self::Heart => "icons/heart.svg",
            Self::Bookmark => "icons/bookmark.svg",
            Self::Code => "icons/code.svg",
            Self::Music => "icons/music.svg",
            Self::Zap => "icons/zap.svg",
            Self::ShoppingCart => "icons/shopping-cart.svg",
            Self::ArrowLeft => "icons/arrow-left.svg",
            Self::ArrowRight => "icons/arrow-right.svg",
            Self::User => "icons/user.svg",
            Self::Database => "icons/database.svg",
            Self::Keyboard => "icons/keyboard.svg",
            Self::CircleInfo => "icons/info.svg",
        }
        .into()
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "arrow-left" => Some(Self::ArrowLeft),
            "arrow-right" => Some(Self::ArrowRight),
            "folder" => Some(Self::Folder),
            "folder-open" => Some(Self::FolderOpen),
            "file-text" => Some(Self::FileText),
            "search" => Some(Self::Search),
            "settings" => Some(Self::Settings),
            "plus" => Some(Self::Plus),
            "chevron-right" => Some(Self::ChevronRight),
            "chevron-down" => Some(Self::ChevronDown),
            "star" => Some(Self::Star),
            "check" => Some(Self::Check),
            "trash" | "trash-2" => Some(Self::Trash2),
            "pencil" => Some(Self::Pencil),
            "copy" => Some(Self::Copy),
            "folder-plus" => Some(Self::FolderPlus),
            "file-plus" => Some(Self::FilePlus),
            "panel-left" => Some(Self::PanelLeft),
            "x" => Some(Self::X),
            "refresh-cw" => Some(Self::RefreshCw),
            "briefcase" => Some(Self::Briefcase),
            "lightbulb" => Some(Self::Lightbulb),
            "rocket" => Some(Self::Rocket),
            "target" => Some(Self::Target),
            "graduation-cap" => Some(Self::GraduationCap),
            "palette" => Some(Self::Palette),
            "home" => Some(Self::Home),
            "wallet" => Some(Self::Wallet),
            "heart" => Some(Self::Heart),
            "bookmark" => Some(Self::Bookmark),
            "code" => Some(Self::Code),
            "music" => Some(Self::Music),
            "zap" => Some(Self::Zap),
            "shopping-cart" => Some(Self::ShoppingCart),
            "user" => Some(Self::User),
            "database" => Some(Self::Database),
            "keyboard" => Some(Self::Keyboard),
            "info" | "circle-info" => Some(Self::CircleInfo),
            _ => None,
        }
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
