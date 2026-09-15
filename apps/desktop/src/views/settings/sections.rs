use gpui::*;
use super::components::SettingsSection;
use super::SettingsView;

pub(super) fn render_account(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let cx_app: &mut App = cx;
    SettingsSection::new("Account")
        .child(
            div()
                .px(px(16.))
                .py(px(12.))
                .text_size(px(13.))
                .child("Coming soon"),
        )
        .render_element(cx_app)
}

pub(super) fn render_sync(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let cx_app: &mut App = cx;
    SettingsSection::new("Sync")
        .child(
            div()
                .px(px(16.))
                .py(px(12.))
                .text_size(px(13.))
                .child("Coming soon"),
        )
        .render_element(cx_app)
}

pub(super) fn render_appearance(
    _view: &SettingsView,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let cx_app: &mut App = cx;
    SettingsSection::new("Appearance")
        .child(
            div()
                .px(px(16.))
                .py(px(12.))
                .text_size(px(13.))
                .child("Coming soon"),
        )
        .render_element(cx_app)
}

pub(super) fn render_storage(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let cx_app: &mut App = cx;
    SettingsSection::new("Storage")
        .child(
            div()
                .px(px(16.))
                .py(px(12.))
                .text_size(px(13.))
                .child("Coming soon"),
        )
        .render_element(cx_app)
}

pub(super) fn render_keybindings(
    _view: &SettingsView,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let cx_app: &mut App = cx;
    SettingsSection::new("Keybindings")
        .child(
            div()
                .px(px(16.))
                .py(px(12.))
                .text_size(px(13.))
                .child("Coming soon"),
        )
        .render_element(cx_app)
}

pub(super) fn render_developer(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
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

pub(super) fn render_about(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let cx_app: &mut App = cx;
    SettingsSection::new("About")
        .child(
            div()
                .px(px(16.))
                .py(px(12.))
                .text_size(px(13.))
                .child("Coming soon"),
        )
        .render_element(cx_app)
}
