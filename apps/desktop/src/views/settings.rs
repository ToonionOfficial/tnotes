use crate::state::{ActiveScreen, AppState};
use gpui::*;

pub struct SettingsView {
    pub app_state: Entity<AppState>,
}

impl SettingsView {
    pub fn new(app_state: Entity<AppState>, _cx: &mut Context<Self>) -> Self {
        Self { app_state }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.clone();

        div()
            .id("settings-screen")
            .size_full()
            .bg(rgb(0x141318))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xe6e1e9))
                    .child("Settings"),
            )
            .child(
                div()
                    .id("back-to-notes-btn")
                    .mt_4()
                    .px_3p5()
                    .py_1p5()
                    .rounded_md()
                    .bg(rgb(0x2a2930))
                    .hover(|s| s.bg(rgb(0x36343b)))
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xe6e1e9))
                    .on_click(move |_event, _window, cx| {
                        app_state.update(cx, |state, cx| {
                            state.set_screen(ActiveScreen::Workspace, cx);
                        });
                    })
                    .child("← Back to Notes"),
            )
    }
}
