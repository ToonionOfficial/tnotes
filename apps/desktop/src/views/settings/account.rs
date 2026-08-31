use crate::components::icons::*;
use crate::state::AppState;
use crate::state::AuthStatus;
use gpui::*;

pub fn account_tab(app_state: Entity<AppState>, cx: &mut App) -> impl IntoElement {
    let auth_store = app_state.read(cx).auth_store.read(cx);
    let user_id = auth_store.user_id.clone();
    let device_id = auth_store.device_id.clone();
    let status = auth_store.status.clone();

    let (status_label, status_dot_color, status_desc) = match status {
        AuthStatus::Connected {
            ref username,
            ref server_url,
            ..
        } => (
            "Connected",
            rgb(0xa6e3a1),
            format!("Synced with server: {server_url} ({username})"),
        ),
        AuthStatus::Connecting => (
            "Connecting...",
            rgb(0xfacc15),
            "Attempting connection to sync server".to_string(),
        ),
        AuthStatus::Error(ref err) => (
            "Sync Error",
            rgb(0xffb4ab),
            format!("Connection failed: {err}"),
        ),
        AuthStatus::LocalOnly => (
            "On-Device",
            rgb(0xfacc15),
            "Stored locally on this device in SQLite".to_string(),
        ),
    };

    div()
        .id("settings-account-screen")
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
                .gap_6()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xe6e1e9))
                                .child("Account & Profile"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x938f99))
                                .child("Manage your user identity, active device identifier, and sync connection."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .p_5()
                        .rounded_xl()
                        .bg(rgb(0x201f24))
                        .border_1()
                        .border_color(rgb(0x302e36))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_4()
                                .child(
                                    div()
                                        .w_12()
                                        .h_12()
                                        .rounded_2xl()
                                        .bg(rgb(0x2a2930))
                                        .border_1()
                                        .border_color(rgb(0x36343b))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(icon_user().size_6().text_color(rgb(0xcabeff))),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xe6e1e9))
                                                .child(user_id),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x938f99))
                                                .child(status_desc),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .py_1p5()
                                .rounded_full()
                                .bg(rgb(0x2a2930))
                                .border_1()
                                .border_color(rgb(0x302e36))
                                .child(div().w_2().h_2().rounded_full().bg(status_dot_color))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0xe6e1e9))
                                        .child(status_label),
                                ),
                        ),
                )
                .child(account_section_title("DEVICE IDENTIFIER"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .rounded_xl()
                        .bg(rgb(0x201f24))
                        .border_1()
                        .border_color(rgb(0x302e36))
                        .child(account_info_row("Device ULID", device_id))
                        .child(div().h_px().bg(rgb(0x302e36)))
                        .child(account_info_row("Client Type", "Desktop Native (Rust + GPUI)"))
                        .child(div().h_px().bg(rgb(0x302e36)))
                        .child(account_info_row("Renderer", "GPU Blade (Vulkan / Metal / Direct3D)"))
                        .child(div().h_px().bg(rgb(0x302e36)))
                        .child(account_info_row("Local Storage", "SQLite 3 with WAL & FTS5")),
                )
                .child(account_section_title("SECURITY & ISOLATION"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_4()
                        .rounded_xl()
                        .bg(rgb(0x201f24))
                        .border_1()
                        .border_color(rgb(0x302e36))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(icon_key().size_5().text_color(rgb(0xcabeff)))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xe6e1e9))
                                                .child("Local-First Isolation"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x79747e))
                                                .child("Your notes and folders are always saved on this device first. Sync operations replicate delta mutations with conflict-free resolution."),
                                        ),
                                ),
                        ),
                ),
        )
}

fn account_section_title(label: &'static str) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0x79747e))
        .child(label)
}

fn account_info_row(label: &'static str, value: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x938f99))
                .child(label),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xe6e1e9))
                .child(value.into()),
        )
}
