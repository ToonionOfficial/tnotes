use gpui::*;
use super::components::SettingsSection;
use super::SettingsView;

pub(super) fn render(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let cx_app: &mut App = cx;
    SettingsSection::new("Developer")
        .child(
            div()
                .px(px(16.))
                .py(px(12.))
                .text_size(px(13.))
                .child("Coming soon"),
        )
        .render_element(cx_app)
}
