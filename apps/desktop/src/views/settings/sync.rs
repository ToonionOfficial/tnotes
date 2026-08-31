use crate::components::icons::*;
use crate::state::AppState;
use crate::state::AuthStatus;
use gpui::*;

pub fn sync_tab(app_state: Entity<AppState>, cx: &mut App) -> impl IntoElement {
    let app_state_read = app_state.read(cx);
    let auth_store = app_state_read.auth_store.read(cx);
    let sync_store = app_state_read.sync_store.read(cx);

    let status = auth_store.status.clone();
    let auto_sync = sync_store.auto_sync;
    let sync_store_entity = app_state_read.sync_store.clone();
    let auth_store_entity = app_state_read.auth_store.clone();

    let (server_url, is_connected, status_dot_color, status_label) = match status {
        AuthStatus::Connected {
            ref server_url,
            ..
        } => (
            server_url.clone(),
            true,
            rgb(0xa6e3a1),
            "Connected",
        ),
        AuthStatus::Connecting => (
            "Connecting...".to_string(),
            false,
            rgb(0xfacc15),
            "Connecting",
        ),
        AuthStatus::Error(ref err) => (
            format!("Error: {err}"),
            false,
            rgb(0xffb4ab),
            "Error",
        ),
        AuthStatus::LocalOnly => (
            "No Server Connected".to_string(),
            false,
            rgb(0xfacc15),
            "Offline",
        ),
    };

    div()
        .id("settings-sync-screen")
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
                // 1. Sync Settings Section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(section_title("SYNC"))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_4()
                                // Row 1: Sync Status
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
                                                .child(icon_cloud().size_5().text_color(rgb(0xcabeff)))
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
                                                                .child("Sync Status"),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0x938f99))
                                                                .child(if is_connected {
                                                                    server_url
                                                                } else {
                                                                    "Offline / On-Device".to_string()
                                                                }),
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
                                )
                                .child(div().h_px().bg(rgb(0x201f24)))
                                // Row 2: Auto-Sync Toggle
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
                                                .child(icon_sync().size_5().text_color(rgb(0xcabeff)))
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
                                                                .child("Auto-Sync"),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0x938f99))
                                                                .child("Sync changes automatically when connected"),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("btn-toggle-autosync")
                                                .flex()
                                                .items_center()
                                                .px_2p5()
                                                .py_1()
                                                .rounded_full()
                                                .border_1()
                                                .border_color(if auto_sync {
                                                    rgb(0xa6e3a1)
                                                } else {
                                                    rgb(0x302e36)
                                                })
                                                .bg(if auto_sync {
                                                    rgb(0x1e3a2f)
                                                } else {
                                                    rgb(0x201f24)
                                                })
                                                .cursor_pointer()
                                                .on_click({
                                                    let store = sync_store_entity.clone();
                                                    move |_e, _w, cx| {
                                                        store.update(cx, |s, cx| s.toggle_auto_sync(cx));
                                                    }
                                                })
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(if auto_sync {
                                                            rgb(0xa6e3a1)
                                                        } else {
                                                            rgb(0x79747e)
                                                        })
                                                        .child(if auto_sync { "Enabled" } else { "Disabled" }),
                                                ),
                                        ),
                                )
                                .child(div().h_px().bg(rgb(0x201f24)))
                                // Row 3: Connect Server (when offline) OR Sync Now + Disconnect (when connected)
                                .child(if !is_connected {
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
                                                .child(icon_globe().size_5().text_color(rgb(0xcabeff)))
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
                                                                .child("Connect Server"),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0x938f99))
                                                                .child("Pair this device with a self-hosted or cloud sync server"),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("btn-connect-server")
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .bg(rgb(0x201f24))
                                                .hover(|s| s.bg(rgb(0x2a2930)))
                                                .cursor_pointer()
                                                .on_click({
                                                    let auth = auth_store_entity.clone();
                                                    move |_e, _w, cx| {
                                                        auth.update(cx, |a, cx| {
                                                            a.set_connected(
                                                                "http://localhost:3000".to_string(),
                                                                "local_user".to_string(),
                                                                "local@tnotes.internal".to_string(),
                                                                cx,
                                                            );
                                                        });
                                                    }
                                                })
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(rgb(0xcabeff))
                                                        .child("Connect (Localhost:3000)"),
                                                ),
                                        )
                                        .into_any_element()
                                } else {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_4()
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
                                                        .child(icon_sync().size_5().text_color(rgb(0xcabeff)))
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
                                                                        .child("Sync Now"),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(rgb(0x938f99))
                                                                        .child("Send and receive latest changes"),
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .id("btn-sync-now")
                                                        .px_3()
                                                        .py_1p5()
                                                        .rounded_md()
                                                        .bg(rgb(0x201f24))
                                                        .hover(|s| s.bg(rgb(0x2a2930)))
                                                        .cursor_pointer()
                                                        .on_click({
                                                            let sync = sync_store_entity.clone();
                                                            move |_e, _w, cx| {
                                                                sync.update(cx, |s, cx| {
                                                                    s.set_status(crate::state::sync_store::SyncStatus::Synced {
                                                                        last_synced_at: tnotes_core::models::current_time_ms(),
                                                                    }, cx);
                                                                });
                                                            }
                                                        })
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                .text_color(rgb(0xcabeff))
                                                                .child("Sync Now"),
                                                        ),
                                                ),
                                        )
                                        .child(div().h_px().bg(rgb(0x201f24)))
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
                                                        .child(icon_log_out().size_5().text_color(rgb(0xffb4ab)))
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_0p5()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .font_weight(FontWeight::SEMIBOLD)
                                                                        .text_color(rgb(0xffb4ab))
                                                                        .child("Disconnect Server"),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(rgb(0x938f99))
                                                                        .child("Unlink from server while keeping local notes on this device"),
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .id("btn-disconnect-server")
                                                        .px_3()
                                                        .py_1p5()
                                                        .rounded_md()
                                                        .bg(rgb(0x2d1a1e))
                                                        .hover(|s| s.bg(rgb(0x3e2329)))
                                                        .cursor_pointer()
                                                        .on_click({
                                                            let auth = auth_store_entity.clone();
                                                            move |_e, _w, cx| {
                                                                auth.update(cx, |a, cx| {
                                                                    a.set_local_only(cx);
                                                                });
                                                            }
                                                        })
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                .text_color(rgb(0xffb4ab))
                                                                .child("Disconnect"),
                                                        ),
                                                ),
                                        )
                                        .into_any_element()
                                }),
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
