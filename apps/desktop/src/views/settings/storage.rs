use super::SettingsView;
use super::components::{SettingsRow, SettingsSection, SettingsTelemetry};
use crate::components::IconName;
use gpui::*;

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let (vault_path, active_count, trashed_count, folder_count) = {
        let store = view.store();
        let store = store.read(cx);
        (
            crate::paths::database_file(),
            store.active_note_count(),
            store.trashed_note_count(),
            store.folder_tree().len(),
        )
    };
    let cx_app: &mut App = cx;
    let vault_label = vault_path.to_string_lossy().to_string();

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
        Err(_) => "Local SQLite".to_string(),
    };

    let parent_dir = vault_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| vault_path.clone());

    let telemetry = SettingsTelemetry::new("Storage Footprint", vault_label)
        .total_size(db_size_str.clone())
        .active_notes(active_count, format!("{active_count} notes"))
        .trashed_notes(trashed_count, format!("{trashed_count} notes"))
        .on_reveal(move |_, _| {
            let _ = std::process::Command::new("xdg-open")
                .arg(&parent_dir)
                .spawn();
        });

    let store_for_export = view.store();
    let store_for_export_md = view.store();
    let store_for_export_json = view.store();

    let storage_section = SettingsSection::new("Storage")
        .subtitle("Local SQLite database metrics and note export")
        .child(
            SettingsRow::new("settings-storage-db", "Local Database")
                .icon(IconName::Database)
                .subtitle(format!("{active_count} notes • {folder_count} folders"))
                .value(db_size_str),
        )
        .child(
            SettingsRow::new("settings-storage-export", "Export Notes")
                .icon(IconName::FileText)
                .subtitle("Export all notes as Markdown and JSON backup")
                .value("Markdown / JSON")
                .on_press(move |_, cx| {
                    let notes = store_for_export.read(cx).active_notes();
                    let export_dir = crate::paths::data_dir().join("exports");
                    let md_dir = export_dir.join("markdown");
                    if std::fs::create_dir_all(&md_dir).is_ok() {
                        for note in &notes {
                            let safe_title = note.title.replace('/', "-").replace('\\', "-");
                            let file_name = if safe_title.trim().is_empty() {
                                format!("{}.md", note.id)
                            } else {
                                format!("{safe_title}.md")
                            };
                            let content = format!(
                                "---\ntitle: {}\nid: {}\n---\n\n{}",
                                note.title, note.id, note.searchable_text
                            );
                            let _ = std::fs::write(md_dir.join(file_name), content);
                        }
                        if let Ok(json) = serde_json::to_string_pretty(&notes) {
                            let _ = std::fs::write(export_dir.join("vault_backup.json"), json);
                        }
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&export_dir)
                            .spawn();
                    }
                }),
        )
        .render_element(cx_app);

    let store_for_vacuum = view.store();
    let store_for_empty = view.store();

    let maintenance_section = SettingsSection::new("Database Maintenance")
        .subtitle("Optimize disk performance and reclaim deleted file storage")
        .child(
            SettingsRow::new("settings-storage-vacuum", "Vacuum Database")
                .icon(IconName::Database)
                .subtitle("Defragment SQLite b-trees and reclaim free pages")
                .value("Execute Vacuum")
                .on_press(move |_, cx| {
                    store_for_vacuum.update(cx, |s, cx| {
                        let _ = s.vacuum();
                        cx.notify();
                    });
                }),
        )
        .child(
            SettingsRow::new("settings-storage-empty-trash", "Empty Trash")
                .icon(IconName::Trash2)
                .subtitle("Permanently remove soft-deleted notes from disk")
                .value(if trashed_count > 0 {
                    format!("{trashed_count} notes")
                } else {
                    "Trash is empty".to_string()
                })
                .destructive(trashed_count > 0)
                .disabled(trashed_count == 0)
                .on_press(move |_, cx| {
                    store_for_empty.update(cx, |s, cx| {
                        s.empty_trash(cx);
                    });
                }),
        )
        .render_element(cx_app);

    let export_section = SettingsSection::new("Individual Exports")
        .subtitle("Targeted file export for Markdown or JSON")
        .child(
            SettingsRow::new("settings-storage-export-md", "Export Markdown Archive")
                .icon(IconName::FileText)
                .subtitle("Write active notes as individual .md files with frontmatter")
                .value("Export Files")
                .on_press(move |_, cx| {
                    let notes = store_for_export_md.read(cx).active_notes();
                    let export_dir = crate::paths::data_dir().join("exports").join("markdown");
                    if std::fs::create_dir_all(&export_dir).is_ok() {
                        for note in notes {
                            let safe_title = note.title.replace('/', "-").replace('\\', "-");
                            let file_name = if safe_title.trim().is_empty() {
                                format!("{}.md", note.id)
                            } else {
                                format!("{safe_title}.md")
                            };
                            let content = format!(
                                "---\ntitle: {}\nid: {}\n---\n\n{}",
                                note.title, note.id, note.searchable_text
                            );
                            let _ = std::fs::write(export_dir.join(file_name), content);
                        }
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&export_dir)
                            .spawn();
                    }
                }),
        )
        .child(
            SettingsRow::new("settings-storage-export-json", "Export Vault JSON")
                .icon(IconName::Copy)
                .subtitle("Single-file JSON snapshot with folders, notes, and metadata")
                .value("Export JSON")
                .on_press(move |_, cx| {
                    let notes = store_for_export_json.read(cx).active_notes();
                    let export_dir = crate::paths::data_dir().join("exports");
                    if std::fs::create_dir_all(&export_dir).is_ok() {
                        if let Ok(json) = serde_json::to_string_pretty(&notes) {
                            let file_path = export_dir.join("vault_backup.json");
                            let _ = std::fs::write(&file_path, json);
                            let _ = std::process::Command::new("xdg-open")
                                .arg(&export_dir)
                                .spawn();
                        }
                    }
                }),
        )
        .render_element(cx_app);

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(telemetry)
        .child(storage_section)
        .child(maintenance_section)
        .child(export_section)
        .into_any_element()
}
