pub mod account;
pub mod appearance;
pub mod sidebar;
pub mod sync;

use crate::state::{ActiveScreen, AppState, SettingsTab};
use account::account_tab;
use appearance::appearance_tab;
use sidebar::settings_sidebar;
use sync::sync_tab;
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state_read = self.app_state.read(cx);
        let settings_tab = app_state_read.settings_tab;
        let app_state_entity = self.app_state.clone();

        div()
            .id("settings-view-root")
            .size_full()
            .bg(rgb(0x141318))
            .flex()
            .flex_row()
            .child(settings_sidebar(
                app_state_entity.clone(),
                settings_tab,
                {
                    let app_state = app_state_entity.clone();
                    move |_event, _window, cx| {
                        app_state.update(cx, |state, cx| {
                            state.set_screen(ActiveScreen::Workspace, cx);
                        });
                    }
                },
            ))
            .child(match settings_tab {
                SettingsTab::Account => account_tab(app_state_entity, cx).into_any_element(),
                SettingsTab::Sync => sync_tab(app_state_entity, cx).into_any_element(),
                SettingsTab::Appearance => appearance_tab(app_state_entity, cx).into_any_element(),
                SettingsTab::Storage => placeholder_screen("Storage & Database", "Database statistics, local SQLite path, and note exports.").into_any_element(),
                SettingsTab::Keyboard => placeholder_screen("Keyboard Shortcuts", "Keymap configuration and customizable shortcut bindings.").into_any_element(),
                SettingsTab::Developer => placeholder_screen("Developer Tools & Flags", "FPS monitor HUD, benchmark stress tests, and SQLite housekeeping.").into_any_element(),
                SettingsTab::About => placeholder_screen("About TNotes", "Version info, license, and repository links.").into_any_element(),
            })
    }
}

fn placeholder_screen(title: &'static str, description: &'static str) -> impl IntoElement {
    div()
        .id("settings-placeholder-screen")
        .flex_1()
        .h_full()
        .overflow_y_scroll()
        .p_8()
        .flex()
        .flex_col()
        .items_center()
        .child(
            div()
                .w_full()
                .max_w(px(680.0))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xe6e1e9))
                        .child(title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x938f99))
                        .child(description),
                ),
        )
}
