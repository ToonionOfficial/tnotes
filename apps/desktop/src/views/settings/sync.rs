use super::SettingsView;
use super::components::{SettingsRow, SettingsSection};
use crate::components::IconName;
use crate::store::format_relative_time;
use crate::theme::ThemeExt;
use gpui::*;

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let summary = view.store().read(cx).sync_summary();
    let theme = cx.theme().clone();
    let cx_app: &mut App = cx;

    let is_paired = summary.is_paired();
    let (status_label, status_subtitle) = match summary.server_url.clone() {
        Some(url) => ("Connected", url),
        None => (
            "Offline",
            "This vault operates locally without a remote server".to_string(),
        ),
    };
    let last_sync_label = match summary.last_sync_at {
        Some(ts) => format!("Last synced {}", format_relative_time(ts)),
        None => "Never synced to remote".to_string(),
    };

    let status_badge = div()
        .flex()
        .items_center()
        .gap_1p5()
        .rounded_full()
        .bg(theme.secondary)
        .px_2p5()
        .py_1()
        .child(div().w(px(7.)).h(px(7.)).rounded_full().bg(if is_paired {
            rgb(0x22c55e)
        } else {
            rgb(0xffc107)
        }))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground)
                .child(if is_paired { "Connected" } else { "Offline" }),
        );

    let status_banner = div()
        .w_full()
        .rounded(px(10.))
        .bg(theme.card)
        .border_1()
        .border_color(theme.border)
        .p(px(16.))
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .w(px(32.))
                        .h(px(32.))
                        .rounded(px(8.))
                        .bg(theme.secondary)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(if is_paired {
                            rgb(0x22c55e)
                        } else {
                            rgb(0xffc107)
                        })),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_size(px(14.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.foreground)
                                .child(format!("Sync Status: {status_label}")),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(theme.muted_foreground)
                                .child(status_subtitle.clone()),
                        ),
                ),
        )
        .child(status_badge)
        .into_any_element();

    let sync_section = SettingsSection::new("Sync")
        .subtitle("Cloud replication, multi-device syncing, and pairing")
        .child(
            SettingsRow::new("settings-sync-status", "Sync Status")
                .icon(IconName::RefreshCw)
                .subtitle(status_subtitle)
                .value(status_label),
        )
        .child(
            SettingsRow::new("settings-sync-last", "Last Sync")
                .icon(IconName::Check)
                .subtitle(last_sync_label)
                .value(if summary.is_paired() {
                    "Paired"
                } else {
                    "Local only"
                }),
        )
        .child(
            SettingsRow::new("settings-sync-auto", "Auto-Sync")
                .icon(IconName::Zap)
                .subtitle("Sync changes automatically when connected")
                .switch(false)
                .disabled(true),
        )
        .child(
            SettingsRow::new("settings-sync-now", "Sync Now")
                .icon(IconName::RefreshCw)
                .subtitle("Send & receive latest changes")
                .value("Not available")
                .disabled(true),
        )
        .child(
            SettingsRow::new("settings-sync-connect", "Connect Server")
                .icon(IconName::Plus)
                .subtitle("Pair this vault with a self-hosted or cloud sync server")
                .value("Connect")
                .disabled(true),
        )
        .child(
            SettingsRow::new("settings-sync-disconnect", "Disconnect Server")
                .icon(IconName::Trash2)
                .subtitle("Disconnect from remote sync server and retain local notes")
                .value("Disconnect")
                .destructive(true)
                .disabled(!is_paired),
        )
        .render_element(cx_app);

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(status_banner)
        .child(sync_section)
        .into_any_element()
}
