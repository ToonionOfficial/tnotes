use gpui::*;
use crate::components::{Icon, IconName};
use crate::keymap::ToggleFps;
use crate::theme::ThemeExt;
use super::components::{SettingsRow, SettingsSection};
use super::SettingsView;

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let (active_notes, folder_count, db_path) = {
        let store = view.store();
        let store = store.read(cx);
        (
            store.active_note_count(),
            store.folder_tree().len(),
            crate::paths::database_file(),
        )
    };

    let theme = cx.theme().clone();
    let cx_app: &mut App = cx;

    let store_for_b100 = view.store();
    let store_for_b500 = view.store();
    let store_for_b1000 = view.store();
    let store_for_b5000 = view.store();
    let store_for_bdel = view.store();

    let benchmark_section = SettingsSection::new("Flags & Performance Benchmark")
        .subtitle("Generate local real notes across benchmark folders for load testing (never synced)")
        .child(
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    Icon::new(IconName::Target)
                                        .size(px(15.))
                                        .color(theme.primary),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.5))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.foreground)
                                        .child("Generate Benchmark Notes"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .id("btn-bench-100")
                                        .px_2p5()
                                        .py(px(4.))
                                        .rounded(px(6.))
                                        .bg(theme.secondary)
                                        .hover(|s| s.bg(theme.primary).text_color(theme.primary_foreground))
                                        .cursor_pointer()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground)
                                        .child("+100")
                                        .on_click(move |_, _, cx| {
                                            store_for_b100.update(cx, |s, cx| {
                                                s.create_benchmark_notes(100, cx);
                                            });
                                        }),
                                )
                                .child(
                                    div()
                                        .id("btn-bench-500")
                                        .px_2p5()
                                        .py(px(4.))
                                        .rounded(px(6.))
                                        .bg(theme.secondary)
                                        .hover(|s| s.bg(theme.primary).text_color(theme.primary_foreground))
                                        .cursor_pointer()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground)
                                        .child("+500")
                                        .on_click(move |_, _, cx| {
                                            store_for_b500.update(cx, |s, cx| {
                                                s.create_benchmark_notes(500, cx);
                                            });
                                        }),
                                )
                                .child(
                                    div()
                                        .id("btn-bench-1k")
                                        .px_2p5()
                                        .py(px(4.))
                                        .rounded(px(6.))
                                        .bg(theme.secondary)
                                        .hover(|s| s.bg(theme.primary).text_color(theme.primary_foreground))
                                        .cursor_pointer()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground)
                                        .child("+1k")
                                        .on_click(move |_, _, cx| {
                                            store_for_b1000.update(cx, |s, cx| {
                                                s.create_benchmark_notes(1000, cx);
                                            });
                                        }),
                                )
                                .child(
                                    div()
                                        .id("btn-bench-5k")
                                        .px_2p5()
                                        .py(px(4.))
                                        .rounded(px(6.))
                                        .bg(theme.secondary)
                                        .hover(|s| s.bg(theme.primary).text_color(theme.primary_foreground))
                                        .cursor_pointer()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground)
                                        .child("+5k")
                                        .on_click(move |_, _, cx| {
                                            store_for_b5000.update(cx, |s, cx| {
                                                s.create_benchmark_notes(5000, cx);
                                            });
                                        }),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("btn-bench-delete")
                        .w_full()
                        .py(px(6.))
                        .rounded(px(6.))
                        .bg(rgb(0xdc2626))
                        .hover(|s| s.bg(rgb(0xb91c1c)))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .child(Icon::new(IconName::Trash2).size(px(13.)).color(rgb(0xffffff).into()))
                        .child("Delete all benchmark notes")
                        .on_click(move |_, _, cx| {
                            store_for_bdel.update(cx, |s, cx| {
                                s.delete_benchmark_notes(cx);
                            });
                        }),
                ),
        )
        .render_element(cx_app);

    let engine_section = SettingsSection::new("Engine & Display Telemetry")
        .subtitle("GPU acceleration, display frame pacing, and graphics backend")
        .child(
            SettingsRow::new("settings-dev-fps", "Performance HUD Overlay")
                .icon(IconName::Zap)
                .subtitle("Display live frame rate counter and microsecond render times")
                .value("Toggle HUD (F3)")
                .on_press(|window, cx| {
                    window.dispatch_action(Box::new(ToggleFps), cx);
                }),
        )
        .child(
            SettingsRow::new("settings-dev-framework", "UI Framework")
                .icon(IconName::Code)
                .subtitle("Zed GPUI 0.2 retained/immediate hybrid rendering pipeline")
                .value("GPUI (Rust)"),
        )
        .child(
            SettingsRow::new("settings-dev-budget", "Target Frame Budget")
                .icon(IconName::Target)
                .subtitle("Display frame budget threshold for 144 Hz refresh rate")
                .value("6.94 ms / frame"),
        )
        .render_element(cx_app);

    let db_section = SettingsSection::new("Database Diagnostics")
        .subtitle("SQLite storage engine, schema migrations, and connection pooling")
        .child(
            SettingsRow::new("settings-dev-sqlite", "Database Driver")
                .icon(IconName::Database)
                .subtitle("Embedded C SQLite library compiled into native binary")
                .value("Rusqlite (SQLite 3)"),
        )
        .child(
            SettingsRow::new("settings-dev-schema", "Schema Migration Version")
                .icon(IconName::Bookmark)
                .subtitle("Database structure migration ledger level")
                .value("Migration 002 (changes)"),
        )
        .child(
            SettingsRow::new("settings-dev-journal", "Journal Mode")
                .icon(IconName::RefreshCw)
                .subtitle("Concurrent read operations without writer lock starvation")
                .value("WAL Mode"),
        )
        .render_element(cx_app);

    let store_for_debug = view.store();

    let utils_section = SettingsSection::new("Debug Utilities")
        .subtitle("Developer actions and diagnostic clipboard export")
        .child(
            SettingsRow::new("settings-dev-report", "Copy Diagnostic Report")
                .icon(IconName::Copy)
                .subtitle("Copy system specs, vault size, and architecture report")
                .value("Copy to Clipboard")
                .on_press(move |_, cx| {
                    let report = format!(
                        "TNotes Diagnostic Report\nOS: Linux\nDatabase: {}\nActive Notes: {}\nFolders: {}\nEngine: GPUI 0.2\nArch: x86_64\n",
                        db_path.display(),
                        active_notes,
                        folder_count
                    );
                    cx.write_to_clipboard(ClipboardItem::new_string(report));
                }),
        )
        .child(
            SettingsRow::new("settings-dev-clear-history", "Reset Navigation History")
                .icon(IconName::Trash2)
                .subtitle("Clear the session back and forward navigation stack")
                .value("Reset Stack")
                .on_press(move |_, cx| {
                    store_for_debug.update(cx, |s, cx| {
                        s.navigate_back(cx);
                    });
                }),
        )
        .render_element(cx_app);

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(benchmark_section)
        .child(engine_section)
        .child(db_section)
        .child(utils_section)
        .into_any_element()
}
