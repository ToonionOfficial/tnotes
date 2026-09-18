use super::SidebarView;
use super::model::FolderIconPickerState;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;
use gpui::*;

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
    #[allow(dead_code)]
    pub fn toggle_icon_picker(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        self.toggle_icon_picker_at(folder_id, None, cx);
    }

    pub fn toggle_icon_picker_at(
        &mut self,
        folder_id: &str,
        position: Option<Point<Pixels>>,
        cx: &mut Context<Self>,
    ) {
        if self.picking_icon_for.as_ref().map(|s| s.folder_id.as_str()) == Some(folder_id) {
            self.picking_icon_for = None;
        } else {
            self.picking_icon_for = Some(FolderIconPickerState {
                folder_id: folder_id.to_string(),
                position: position.unwrap_or_else(|| point(px(80.), px(120.))),
            });
            self.renaming = None;
        }
        self.context_menu = None;
        cx.notify();
    }

    pub fn pick_folder_icon(&mut self, folder_id: &str, icon: IconName, cx: &mut Context<Self>) {
        let folder_id = folder_id.to_string();
        let name = folder_icon_name(icon);
        self.store
            .update(cx, |store, cx| store.set_folder_icon(&folder_id, name, cx));
        self.picking_icon_for = None;
        cx.notify();
    }

    pub(super) fn icon_picker_element(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let picker_state = self.picking_icon_for.as_ref()?;
        let folder_id = picker_state.folder_id.clone();
        let theme = cx.theme().clone();
        let current_icon = self
            .store
            .read(cx)
            .folder_tree()
            .iter()
            .find(|n| n.folder.id == folder_id)
            .map(|n| n.folder.icon.clone())
            .unwrap_or_else(|| "folder".to_string());

        let viewport = window.viewport_size();
        let dropdown_width = px(216.);
        let dropdown_height = px(220.);
        let position = if viewport.width > px(0.) && viewport.height > px(0.) {
            let max_x = (viewport.width - dropdown_width - px(16.)).max(px(0.));
            let max_y = (viewport.height - dropdown_height - px(16.)).max(px(0.));
            point(
                picker_state.position.x.min(max_x).max(px(8.)),
                picker_state.position.y.min(max_y).max(px(8.)),
            )
        } else {
            picker_state.position
        };

        let entity = cx.entity();
        let dismiss_entity = entity.clone();
        let dismiss_right = entity.clone();
        let dismiss_escape = entity.clone();

        let dropdown = div()
            .id("folder-icon-dropdown")
            .occlude()
            .track_focus(&self.context_menu_focus)
            .absolute()
            .left(position.x)
            .top(position.y)
            .w(dropdown_width)
            .bg(theme.popover)
            .border_1()
            .border_color(theme.border)
            .rounded(px(8.))
            .shadow_lg()
            .p_2()
            .flex()
            .flex_col()
            .gap_1p5()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down_out(move |event: &MouseDownEvent, _window, cx| {
                if event.button == MouseButton::Left || event.button == MouseButton::Middle {
                    dismiss_entity.update(cx, |this, cx| {
                        this.picking_icon_for = None;
                        cx.notify();
                    });
                }
            })
            .on_mouse_down(MouseButton::Right, move |_, _window, cx| {
                cx.stop_propagation();
                dismiss_right.update(cx, |this, cx| {
                    this.picking_icon_for = None;
                    cx.notify();
                });
            })
            .on_key_down(move |event, _window, cx| {
                if event.keystroke.key == "escape" {
                    dismiss_escape.update(cx, |this, cx| {
                        this.picking_icon_for = None;
                        cx.notify();
                    });
                }
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .pb_1()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.muted_foreground)
                            .child("Folder Icon"),
                    )
                    .child(
                        div()
                            .id("close-icon-picker-btn")
                            .cursor_pointer()
                            .text_color(theme.muted_foreground)
                            .hover(|s| s.text_color(theme.foreground))
                            .child(Icon::new(IconName::X).size(px(12.)))
                            .on_mouse_down(MouseButton::Left, {
                                let entity = entity.clone();
                                move |_, _window, cx| {
                                    cx.stop_propagation();
                                    entity.update(cx, |this, cx| {
                                        this.picking_icon_for = None;
                                        cx.notify();
                                    });
                                }
                            }),
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
                            .w(px(34.))
                            .h(px(34.))
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
                                theme.border.opacity(0.4)
                            })
                            .hover(|s| s.bg(theme.secondary))
                            .text_color(if selected {
                                theme.foreground
                            } else {
                                theme.muted_foreground
                            })
                            .child(Icon::new(icon).size(px(16.)))
                            .on_mouse_down(MouseButton::Left, move |_, _window, cx| {
                                cx.stop_propagation();
                                entity.update(cx, |this, cx| {
                                    this.pick_folder_icon(&folder_id, icon, cx);
                                });
                            })
                    })),
            );

        Some(deferred(dropdown).into_any_element())
    }
}
