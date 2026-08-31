use crate::components::icons::*;
use crate::state::AppState;
use crate::state::AuthStatus;
use gpui::*;

pub fn account_tab(app_state: Entity<AppState>, cx: &mut App) -> impl IntoElement {
    let auth_store = app_state.read(cx).auth_store.read(cx);
    let status = auth_store.status.clone();

    let (username, server_subtitle, is_connected, status_dot_color, status_label) = match status {
        AuthStatus::Connected {
            ref username,
            ref server_url,
            ..
        } => (
            username.clone(),
            server_url.clone(),
            true,
            rgb(0xa6e3a1),
            "Connected",
        ),
        AuthStatus::Connecting => (
            "Connecting...".to_string(),
            "Attempting connection to sync server".to_string(),
            false,
            rgb(0xfacc15),
            "Connecting",
        ),
        AuthStatus::Error(ref err) => (
            "Sync Error".to_string(),
            format!("Connection failed: {err}"),
            false,
            rgb(0xffb4ab),
            "Error",
        ),
        AuthStatus::LocalOnly => (
            "Local Account".to_string(),
            "Offline / On-Device".to_string(),
            false,
            rgb(0xfacc15),
            "Offline",
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
                .gap_8()
                // 1. Profile Section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(section_title("PROFILE"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
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
                                                .border_1()
                                                .border_color(rgb(0x302e36))
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
                                                        .child(username),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x938f99))
                                                        .child(server_subtitle),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .px_3()
                                        .py_1()
                                        .rounded_full()
                                        .border_1()
                                        .border_color(rgb(0x302e36))
                                        .child(div().w_2().h_2().rounded_full().bg(status_dot_color))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xe6e1e9))
                                                .child(status_label),
                                        ),
                                ),
                        ),
                )
                // 2. Security Section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(section_title("SECURITY"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .py_1()
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
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(rgb(0xe6e1e9))
                                                        .child("Change Password"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x938f99))
                                                        .child("Update your master account password"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x79747e))
                                        .child("Web Dashboard"),
                                ),
                        ),
                )
                // 3. Connected Devices Section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(section_title("CONNECTED DEVICES"))
                        .child(if is_connected {
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .py_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_3()
                                        .child(icon_monitor().size_5().text_color(rgb(0xcabeff)))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(rgb(0xe6e1e9))
                                                        .child("Linux Desktop"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x938f99))
                                                        .child("Active now"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_2p5()
                                        .py_0p5()
                                        .rounded_full()
                                        .border_1()
                                        .border_color(rgb(0x302e36))
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xa6e3a1))
                                        .child("This Device"),
                                )
                        } else {
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py_6()
                                .gap_2()
                                .child(icon_monitor().size_8().text_color(rgb(0x79747e)))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0xe6e1e9))
                                        .child("No Server Connected"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x938f99))
                                        .child("Connect this device to a sync server in Settings to view and manage linked devices."),
                                )
                        }),
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
