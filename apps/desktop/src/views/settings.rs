use gpui::*;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;

pub type OnBackCallback = Box<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct SettingsView {
    focus_handle: FocusHandle,
    on_back: Option<OnBackCallback>,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            on_back: None,
        }
    }

    pub fn set_on_back(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_back = Some(Box::new(handler));
    }

    pub fn trigger_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ref on_back) = self.on_back {
            on_back(window, cx);
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
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key.eq_ignore_ascii_case("escape") {
                    this.trigger_back(window, cx);
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
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.trigger_back(window, cx);
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
    use crate::theme::{ActiveTheme, Theme};
    use gpui::TestAppContext;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn settings_view_trigger_back_invokes_on_back_callback() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let (view, cx) = cx.add_window_view(|_, cx| {
            let mut settings = SettingsView::new(cx);
            settings.set_on_back(move |_, _| {
                called_clone.store(true, Ordering::SeqCst);
            });
            settings
        });
        cx.run_until_parked();

        cx.update(|window, cx| {
            view.update(cx, |v, cx| {
                v.trigger_back(window, cx);
            });
        });
        cx.run_until_parked();

        assert!(called.load(Ordering::SeqCst), "trigger_back should invoke on_back");
    }

    #[test]
    fn settings_view_escape_key_invokes_on_back() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let mut settings = SettingsView::new(cx);
            settings.set_on_back(move |_, _| {
                called_clone.store(true, Ordering::SeqCst);
            });
            settings.focus(window);
            settings
        });
        cx.run_until_parked();

        cx.simulate_keystrokes("escape");
        cx.run_until_parked();

        assert!(called.load(Ordering::SeqCst), "escape keystroke should invoke on_back");
    }
}
