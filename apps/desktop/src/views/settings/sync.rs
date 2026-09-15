use gpui::*;
use crate::components::IconName;
use crate::store::format_relative_time;
use super::components::{SettingsRow, SettingsSection};
use super::SettingsView;

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let summary = view.store().read(cx).sync_summary();
    let cx_app: &mut App = cx;

    let (status_label, status_subtitle) = match summary.server_url.clone() {
        Some(url) => ("Connected".to_string(), url),
        None => (
            "Offline".to_string(),
            "This vault has never been paired with a server".to_string(),
        ),
    };
    let last_sync_label = match summary.last_sync_at {
        Some(ts) => format!("Last synced {}", format_relative_time(ts)),
        None => "Never synced".to_string(),
    };

    SettingsSection::new("Sync")
        .child(
            SettingsRow::new("settings-sync-status", format!("Status: {status_label}"))
                .icon(IconName::RefreshCw)
                .subtitle(status_subtitle),
        )
        .child(
            SettingsRow::new("settings-sync-last", "Last Sync")
                .icon(IconName::Check)
                .subtitle(last_sync_label)
                .value(if summary.is_paired() { "Paired" } else { "Local only" }),
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
                .subtitle("Pair with a sync server")
                .value("Not available")
                .disabled(true),
        )
        .render_element(cx_app)
}
