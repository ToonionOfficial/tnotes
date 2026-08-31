use crate::components::icons::icon_settings;
use crate::keymap::{user_keymap_path, REGISTERED_ACTIONS};
use crate::state::AppState;
use gpui::*;

pub fn settings_modal(
    app_state: Entity<AppState>,
    on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let keymap_path_display = user_keymap_path().to_string_lossy().to_string();

    div()
        .id("settings-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(rgba(0x00000088))
        .flex()
        .items_center()
        .justify_center()
        .on_click(on_close)
        .child(
            div()
                .id("settings-modal-container")
                .w(px(640.0))
                .max_h(px(580.0))
                .flex()
                .flex_col()
                .bg(rgb(0x201f24))
                .border_1()
                .border_color(rgb(0x302e36))
                .rounded_xl()
                .shadow_xl()
                .overflow_hidden()
                .on_click(|_event, _window, cx| {
                    cx.stop_propagation();
                })
                .child(
                    // Header
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_6()
                        .py_4()
                        .border_b_1()
                        .border_color(rgb(0x302e36))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(icon_settings().size_5().text_color(rgb(0xcabeff)))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xe6e1e9))
                                        .child("Preferences & Keybindings"),
                                ),
                        )
                        .child(
                            div()
                                .id("close-settings-btn")
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .bg(rgb(0x2a2930))
                                .hover(|s| s.bg(rgb(0x36343b)))
                                .cursor_pointer()
                                .text_xs()
                                .text_color(rgb(0xe6e1e9))
                                .on_click({
                                    let app_state = app_state.clone();
                                    move |_event, _window, cx| {
                                        app_state.update(cx, |state, cx| {
                                            state.close_modal(cx);
                                        });
                                    }
                                })
                                .child("✕ Close"),
                        ),
                )
                .child(
                    // Body (Scrollable)
                    div()
                        .id("settings-modal-scroll-body")
                        .flex()
                        .flex_col()
                        .p_6()
                        .gap_4()
                        .overflow_y_scroll()
                        // 1. General & Storage
                        .child(section_title("STORAGE & SYNC"))
                        .child(setting_item(
                            "Sync Server Endpoint",
                            "Local or self-hosted TNotes server URL",
                            "http://localhost:3000",
                        ))
                        .child(setting_item(
                            "Storage Backend",
                            "High-performance local SQLite database with FTS5",
                            "~/.local/share/tnotes/tnotes.db",
                        ))
                        // 2. Keyboard Shortcuts Section
                        .child(section_title("KEYBOARD SHORTCUTS"))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1p5()
                                .p_3()
                                .rounded_lg()
                                .bg(rgb(0x141318))
                                .border_1()
                                .border_color(rgb(0x302e36))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xe6e1e9))
                                                .child("Custom Keymap Config File"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xa6e3a1))
                                                .child(keymap_path_display),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x79747e))
                                        .child("Edit your keymap.json to customize shortcuts. Overrides are automatically merged on launch."),
                                )
                                .child(div().h_px().bg(rgb(0x302e36)).my_1())
                                .children(REGISTERED_ACTIONS.iter().map(|action| {
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .py_1()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0xe6e1e9))
                                                        .child(action.label),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.0))
                                                        .text_color(rgb(0x79747e))
                                                        .child(action.id),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(rgb(0x2a2930))
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(rgb(0xcabeff))
                                                .child(action.default_shortcut),
                                        )
                                })),
                        ),
                ),
        )
}

fn section_title(label: &'static str) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0x79747e))
        .child(label)
}

fn setting_item(
    title: &'static str,
    description: &'static str,
    value: &'static str,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .p_3()
        .rounded_lg()
        .bg(rgb(0x141318))
        .border_1()
        .border_color(rgb(0x302e36))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xe6e1e9))
                        .child(title),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xcabeff))
                        .child(value),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x79747e))
                .child(description),
        )
}
