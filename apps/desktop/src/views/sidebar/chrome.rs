use super::SidebarView;
use crate::components::{
    Button, Icon, IconName, Input, SidebarContent, SidebarFooter, SidebarGroup, SidebarHeader,
    SidebarRail, SidebarRailItem, SidebarToggleButton,
};
use crate::theme::ThemeExt;
use gpui::*;

impl SidebarView {
    pub(super) fn render_rail(&self, cx: &mut Context<Self>) -> SidebarRail {
        SidebarRail::new("main-sidebar-collapsed")
            .top_item(
                SidebarRailItem::new("sidebar-expand-rail-btn", IconName::PanelLeft).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.toggle_collapsed(cx);
                    }),
                ),
            )
            .separator()
            .top_item(
                SidebarRailItem::new("sidebar-new-note-rail-btn", IconName::Plus).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.create_new_note(cx);
                    }),
                ),
            )
            .bottom_item(
                SidebarRailItem::new("sidebar-settings-rail-btn", IconName::Settings).on_click(
                    cx.listener(|_this, _, window, cx| {
                        window.dispatch_action(Box::new(crate::keymap::OpenSettings), cx);
                    }),
                ),
            )
    }

    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> SidebarHeader {
        let theme = cx.theme().clone();
        SidebarHeader::new()
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(20.))
                                    .h(px(20.))
                                    .rounded(px(5.))
                                    .bg(theme.primary)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        Icon::new(IconName::Zap)
                                            .size(px(12.))
                                            .color(theme.primary_foreground),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.foreground)
                                    .child("TNotes"),
                            ),
                    )
                    .child(
                        SidebarToggleButton::new("sidebar-collapse-btn").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.toggle_collapsed(cx);
                            },
                        )),
                    ),
            )
            .child(
                Button::primary("new-note-btn", "New Note")
                    .leading_icon(IconName::Plus)
                    .full_width(true)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.create_new_note(cx);
                    })),
            )
            .child(
                Input::sidebar_search("sidebar-search-input")
                    .state(&self.search_state)
                    .focus_handle(self.search_focus.clone())
                    .on_key_down(cx.listener(|this, event, window, cx| {
                        this.handle_search_key(event, window, cx);
                    }))
                    .on_clear(cx.listener(|this, _, _, cx| {
                        this.search_state.clear();
                        cx.notify();
                    })),
            )
    }

    pub(super) fn render_nav_group(
        &self,
        starred_count: usize,
        trash_count: usize,
        is_starred_active: bool,
        is_trash_active: bool,
        cx: &mut Context<Self>,
    ) -> SidebarGroup {
        let mut nav_group = SidebarGroup::new();

        // Starred is a virtual view over `WHERE pinned = 1`, not a folder:
        // no folder context menu here.
        nav_group = nav_group.child(
            Button::sidebar("nav-starred", "Starred")
                .leading_icon(IconName::Star)
                .count(starred_count)
                .active(is_starred_active)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.open_starred(cx);
                })),
        );

        nav_group = nav_group.child(
            Button::sidebar("nav-trash", "Trash")
                .leading_icon(IconName::Trash2)
                .count(trash_count)
                .active(is_trash_active)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.open_trash(cx);
                })),
        );

        nav_group
    }

    pub(super) fn render_footer(&self, cx: &mut Context<Self>) -> SidebarFooter {
        let theme = cx.theme().clone();
        SidebarFooter::new().child(
            div()
                .id("sidebar-footer-settings-btn")
                .w_full()
                .h(px(34.))
                .px_2()
                .rounded(px(6.))
                .flex()
                .items_center()
                .justify_between()
                .cursor_pointer()
                .hover(|s| s.bg(theme.secondary))
                .on_click(cx.listener(|_this, _, window, cx| {
                    window.dispatch_action(Box::new(crate::keymap::OpenSettings), cx);
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .w(px(22.))
                                .h(px(22.))
                                .rounded_full()
                                .bg(theme.muted)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(11.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground)
                                .child("A"),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground)
                                .child("Personal Vault"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(IconName::Settings).size(px(14.))),
                ),
        )
    }

    pub(super) fn render_content(&mut self, cx: &mut Context<Self>) -> SidebarContent {
        use crate::store::NavigationLocation;

        let (selected_note, active_location, starred_count, trash_count) = {
            let store = self.store.read(cx);
            (
                store.selected_note_id(),
                store.active_location().clone(),
                store.starred_note_count(),
                store.trashed_note_count(),
            )
        };
        let query = self.search_state.value().trim().to_string();

        let mut content = SidebarContent::new();

        if !query.is_empty() {
            content = content.child(self.render_search_results(selected_note, cx));
        } else {
            let is_starred_active = active_location == NavigationLocation::Starred;
            let is_trash_active = active_location == NavigationLocation::Trash;
            content = content.child(self.render_nav_group(
                starred_count,
                trash_count,
                is_starred_active,
                is_trash_active,
                cx,
            ));
            content = content.child(self.render_folder_tree(cx));
        }

        content
    }
}
