use super::SettingsView;
use super::components::{SettingsRow, SettingsSection, SettingsTelemetry};
use crate::components::IconName;
use gpui::*;

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let (username, vault_path, active_count, trashed_count, folder_count) = {
        let store = view.store();
        let store = store.read(cx);
        (
            store.user_id().to_string(),
            crate::paths::database_file(),
            store.active_note_count(),
            store.trashed_note_count(),
            store.folder_tree().len(),
        )
    };
    let cx_app: &mut App = cx;
    let vault_label = vault_path.to_string_lossy().to_string();
    let note_count = active_count + trashed_count;

    let db_size_str = match std::fs::metadata(&vault_path) {
        Ok(meta) => {
            let bytes = meta.len();
            if bytes < 1024 {
                format!("{bytes} B")
            } else if bytes < 1024 * 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else {
                format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
            }
        }
        Err(_) => "Local Storage".to_string(),
    };

    let parent_dir = vault_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| vault_path.clone());

    let telemetry = SettingsTelemetry::new("Personal Vault Console", vault_label.clone())
        .total_size(db_size_str)
        .active_notes(active_count, format!("{active_count} notes"))
        .trashed_notes(trashed_count, format!("{trashed_count} notes"))
        .on_reveal(move |_, _| {
            let _ = std::process::Command::new("xdg-open")
                .arg(&parent_dir)
                .spawn();
        });

    let section = SettingsSection::new("Account")
        .subtitle("Local credentials and physical storage location")
        .child(
            SettingsRow::new("settings-account-profile", "Local Account")
                .icon(IconName::User)
                .subtitle(format!("{note_count} notes • {folder_count} folders"))
                .value(username),
        )
        .child(
            SettingsRow::new("settings-account-vault", "Vault Location")
                .icon(IconName::Database)
                .subtitle("SQLite database file")
                .value(vault_label),
        )
        .child(
            SettingsRow::new("settings-account-mode", "Sync Status")
                .icon(IconName::RefreshCw)
                .subtitle("Notes stay on this device")
                .value("On-device"),
        )
        .render_element(cx_app);

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(telemetry)
        .child(section)
        .into_any_element()
}
