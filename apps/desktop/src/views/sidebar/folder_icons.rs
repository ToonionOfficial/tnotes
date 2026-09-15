use gpui::*;
use crate::components::IconName;
use super::SidebarView;

/// Folder icon options (mobile parity) with their stored lowercase names.
pub const FOLDER_ICON_OPTIONS: &[(&str, IconName)] = &[
    ("folder", IconName::Folder),
    ("briefcase", IconName::Briefcase),
    ("lightbulb", IconName::Lightbulb),
    ("file-text", IconName::FileText),
    ("rocket", IconName::Rocket),
    ("target", IconName::Target),
    ("graduation-cap", IconName::GraduationCap),
    ("palette", IconName::Palette),
    ("home", IconName::Home),
    ("wallet", IconName::Wallet),
    ("star", IconName::Star),
    ("heart", IconName::Heart),
    ("bookmark", IconName::Bookmark),
    ("code", IconName::Code),
    ("music", IconName::Music),
    ("zap", IconName::Zap),
    ("shopping-cart", IconName::ShoppingCart),
];

pub fn folder_icon_name(icon: IconName) -> &'static str {
    FOLDER_ICON_OPTIONS
        .iter()
        .find(|(_, candidate)| *candidate == icon)
        .map(|(name, _)| *name)
        .unwrap_or("folder")
}

pub fn folder_icon_from_name(name: &str) -> Option<IconName> {
    FOLDER_ICON_OPTIONS
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name.trim()))
        .map(|(_, icon)| *icon)
}

/// Resolve a stored `Folder.icon` value to a renderable icon, falling back to
/// the open/closed folder glyph for legacy (emoji) or unknown values.
pub fn folder_icon_for(stored: &str, expanded: bool) -> IconName {
    folder_icon_from_name(stored).unwrap_or(if expanded {
        IconName::FolderOpen
    } else {
        IconName::Folder
    })
}

impl SidebarView {
    pub fn toggle_icon_picker(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        if self.picking_icon_for.as_deref() == Some(folder_id) {
            self.picking_icon_for = None;
        } else {
            self.picking_icon_for = Some(folder_id.to_string());
            self.renaming = None;
        }
        self.context_menu = None;
        cx.notify();
    }

    pub fn pick_folder_icon(&mut self, folder_id: &str, icon: IconName, cx: &mut Context<Self>) {
        let folder_id = folder_id.to_string();
        let name = folder_icon_name(icon);
        self.store.update(cx, |store, cx| {
            store.set_folder_icon(&folder_id, name, cx)
        });
        self.picking_icon_for = None;
        cx.notify();
    }
}
