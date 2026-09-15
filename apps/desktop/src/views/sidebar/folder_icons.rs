use gpui::*;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;
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

    pub(super) fn icon_picker_element(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let folder_id = self.picking_icon_for.clone()?;
        let theme = cx.theme().clone();
        let current_icon = self
            .store
            .read(cx)
            .folder_tree()
            .iter()
            .find(|n| n.folder.id == folder_id)
            .map(|n| n.folder.icon.clone())
            .unwrap_or_else(|| "folder".to_string());

        let entity = cx.entity();
        Some(
            div()
                .id("icon-picker-overlay")
                .absolute()
                .inset_0()
                .bg(gpui::rgba(0x00000040))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.picking_icon_for = None;
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .id("icon-picker-dialog")
                        .w(px(240.))
                        .p_3()
                        .rounded(px(8.))
                        .bg(theme.background)
                        .border_1()
                        .border_color(theme.border)
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .on_mouse_down(MouseButton::Left, |_, _, _| {})
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground)
                                        .child("Choose Folder Icon"),
                                )
                                .child(
                                    div()
                                        .id("close-icon-picker-btn")
                                        .cursor_pointer()
                                        .text_color(theme.muted_foreground)
                                        .hover(|s| s.text_color(theme.foreground))
                                        .child(Icon::new(IconName::X).size(px(14.)))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.picking_icon_for = None;
                                            cx.notify();
                                        })),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_1()
                                .children(FOLDER_ICON_OPTIONS.iter().map(|(name, icon)| {
                                    let selected = current_icon.eq_ignore_ascii_case(name);
                                    let icon = *icon;
                                    let folder_id = folder_id.clone();
                                    let entity = entity.clone();
                                    div()
                                        .id(SharedString::from(format!("dialog-icon-{name}")))
                                        .w(px(32.))
                                        .h(px(32.))
                                        .rounded(px(6.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .bg(if selected {
                                            theme.secondary
                                        } else {
                                            gpui::transparent_black()
                                        })
                                        .border_1()
                                        .border_color(if selected {
                                            theme.ring
                                        } else {
                                            theme.border
                                        })
                                        .hover(|s| s.bg(theme.secondary))
                                        .text_color(if selected {
                                            theme.foreground
                                        } else {
                                            theme.muted_foreground
                                        })
                                        .child(Icon::new(icon).size(px(16.)))
                                        .on_click(move |_, _, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.pick_folder_icon(&folder_id, icon, cx);
                                            });
                                        })
                                })),
                        ),
                )
                .into_any_element(),
        )
    }
}
