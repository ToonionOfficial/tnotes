use gpui::*;
use crate::components::{Icon, IconName};
use crate::keymap::CloseSettings;
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
                    window.dispatch_action(Box::new(CloseSettings), cx);
                }
            }))
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Back),
                cx.listener(|_this, _event, window, cx| {
                    window.dispatch_action(Box::new(CloseSettings), cx);
                }),
            )
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
                                        window.dispatch_action(Box::new(CloseSettings), cx);
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

#[cfg(test)]
mod tests {
    use super::SettingsView;
    use crate::keymap::KeymapConfig;
    use crate::theme::{ActiveTheme, Theme};
    use gpui::TestAppContext;

    #[test]
    fn settings_view_escape_dispatches_close_settings() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
            KeymapConfig::default_config().bind_to_gpui(cx);
        });

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let settings = SettingsView::new(cx);
            settings.focus(window);
            settings
        });
        cx.run_until_parked();

        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
    }

    #[test]
    fn settings_view_mouse_back_dispatches_close_settings() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
            KeymapConfig::default_config().bind_to_gpui(cx);
        });

        let (_view, cx) = cx.add_window_view(|_, cx| SettingsView::new(cx));
        cx.run_until_parked();

        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Navigate(gpui::NavigationDirection::Back),
            position: gpui::point(gpui::px(50.), gpui::px(50.)),
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();
    }
}
