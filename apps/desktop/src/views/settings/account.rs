use gpui::*;
use crate::components::IconName;
use super::components::{SettingsRow, SettingsSection};
use super::SettingsView;

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let (username, vault_path, note_count, folder_count) = {
        let store = view.store();
        let store = store.read(cx);
        (
            store.user_id().to_string(),
            crate::paths::database_file(),
            store.active_notes().len() + store.trashed_notes().len(),
            store.folder_tree().len(),
        )
    };
    let cx_app: &mut App = cx;
    let vault_label = vault_path.to_string_lossy().to_string();
    SettingsSection::new("Account")
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
        .render_element(cx_app)
}
