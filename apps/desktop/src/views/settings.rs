use gpui::*;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;

pub struct SettingsView {
    focus_handle: FocusHandle,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn focus(&self, window: &mut Window) {
        window.focus(&self.focus_handle);
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id("settings-screen")
            .size_full()
            .flex()
            .bg(theme.background)
            .text_color(theme.foreground)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|_this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key.eq_ignore_ascii_case("escape") {
                    window.dispatch_action(Box::new(crate::keymap::CloseSettings), cx);
                }
            }))
            .child(
                div()
                    .id("settings-sidebar-placeholder")
                    .w(px(240.))
                    .h_full()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .bg(theme.card)
                    .border_r_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .id("settings-sidebar-nav")
                            .flex_1()
                            .p_2(),
                    )
                    .child(
                        div()
                            .id("settings-sidebar-footer")
                            .p_2()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .id("settings-back-btn")
                                    .w_full()
                                    .h(px(34.))
                                    .px_2()
                                    .rounded(px(6.))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme.secondary))
                                    .on_click(cx.listener(|_this, _, window, cx| {
                                        window.dispatch_action(Box::new(crate::keymap::CloseSettings), cx);
                                    }))
                                    .child(
                                        Icon::new(IconName::ArrowLeft)
                                            .size(px(14.))
                                            .color(theme.muted_foreground),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.foreground)
                                            .child("Back to Notes"),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .id("settings-content-area")
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .bg(theme.background)
                    .child(
                        Icon::new(IconName::Settings)
                            .size(px(36.))
                            .color(theme.muted_foreground),
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Settings"),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(theme.muted_foreground)
                            .child("Settings screen with dedicated sidebar coming soon"),
                    ),
            )
    }
}
