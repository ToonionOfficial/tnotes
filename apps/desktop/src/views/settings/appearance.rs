use gpui::*;
use crate::components::IconName;
use crate::theme::{ActiveTheme, Theme, ThemeExt};
use super::components::{SettingsRow, SettingsSection, SettingsSegmented};
use super::SettingsView;

pub(super) fn render(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let theme = cx.theme().clone();
    let is_dark = theme.is_dark();
    let cx_app: &mut App = cx;

    let theme_segmented = SettingsSegmented::new("settings-theme-segmented")
        .option(
            "Dark Theme",
            Some(IconName::Palette),
            is_dark,
            |_, cx| {
                cx.set_global(ActiveTheme(Theme::dark()));
                cx.refresh_windows();
            },
        )
        .option(
            "Light Theme",
            Some(IconName::Lightbulb),
            !is_dark,
            |_, cx| {
                cx.set_global(ActiveTheme(Theme::light()));
                cx.refresh_windows();
            },
        );

    let theme_section = SettingsSection::new("Appearance")
        .subtitle("Select between deep obsidian dark mode and high-clarity light mode")
        .child(
            SettingsRow::new("settings-appearance-dark-mode", "Dark Mode")
                .icon(IconName::Palette)
                .subtitle(if is_dark {
                    "Dark theme enabled"
                } else {
                    "Light theme enabled"
                })
                .switch(is_dark)
                .on_toggle(|on, _, cx| {
                    if on {
                        cx.set_global(ActiveTheme(Theme::dark()));
                    } else {
                        cx.set_global(ActiveTheme(Theme::light()));
                    }
                    cx.refresh_windows();
                }),
        )
        .child(
            div()
                .p(px(16.))
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_size(px(13.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground)
                                .child("Theme Palette"),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(theme.muted_foreground)
                                .child(if is_dark {
                                    "Obsidian background with lavender quartz accents"
                                } else {
                                    "Porcelain background with slate-violet accents"
                                }),
                        ),
                )
                .child(theme_segmented),
        )
        .render_element(cx_app);

    let typography_section = SettingsSection::new("Typography")
        .subtitle("Text scale and typographic rhythm for reading and editing")
        .child(
            SettingsRow::new("settings-font-family", "Typography")
                .icon(IconName::Bookmark)
                .subtitle("Proportional typography with monospace code blocks")
                .value("System"),
        )
        .child(
            SettingsRow::new("settings-font-scale", "Body Font Scale")
                .icon(IconName::Pencil)
                .subtitle("Base font sizing for the editor and note reader")
                .value("14 px (Standard)"),
        )
        .child(
            SettingsRow::new("settings-line-height", "Line Height")
                .icon(IconName::FileText)
                .subtitle("Vertical spacing for editorial breathing room")
                .value("1.6x (Relaxed)"),
        )
        .render_element(cx_app);

    let preview_box = div()
        .w_full()
        .rounded(px(10.))
        .bg(theme.card)
        .border_1()
        .border_color(theme.border)
        .p(px(18.))
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.muted_foreground)
                        .child("Live Document Preview"),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(2.))
                        .rounded(px(4.))
                        .bg(theme.secondary)
                        .text_size(px(11.))
                        .text_color(theme.primary)
                        .child("Realtime Renderer"),
                ),
        )
        .child(
            div()
                .p(px(14.))
                .rounded(px(8.))
                .bg(theme.background)
                .border_1()
                .border_color(theme.border)
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(16.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.foreground)
                        .child("Architecture of a Fast Local Vault"),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(20.))
                        .text_color(theme.foreground)
                        .child("Notes are persisted to an embedded SQLite database using write-ahead logging. GPU rendering guarantees smooth 120+ fps frame pacing regardless of note length."),
                )
                .child(
                    div()
                        .p(px(8.))
                        .rounded(px(6.))
                        .bg(theme.secondary)
                        .text_size(px(12.))
                        .text_color(theme.primary)
                        .child("fn open_vault() -> Result<NoteStore>"),
                ),
        )
        .into_any_element();

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(theme_section)
        .child(typography_section)
        .child(preview_box)
        .into_any_element()
}
