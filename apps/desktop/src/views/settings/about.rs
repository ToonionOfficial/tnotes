use super::SettingsView;
use super::components::{SettingsRow, SettingsSection};
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;
use gpui::*;

pub(super) fn render(_view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let theme = cx.theme().clone();

    let hero_card = div()
        .w_full()
        .rounded(px(10.))
        .bg(theme.card)
        .border_1()
        .border_color(theme.border)
        .p(px(20.))
        .flex()
        .items_center()
        .gap_4()
        .child(
            div()
                .w(px(48.))
                .h(px(48.))
                .rounded(px(12.))
                .bg(theme.primary)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::Zap)
                        .size(px(24.))
                        .color(theme.primary_foreground),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(18.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground)
                                .child("TNotes Desktop"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py(px(2.))
                                .rounded(px(4.))
                                .bg(theme.secondary)
                                .border_1()
                                .border_color(theme.border)
                                .text_size(px(11.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.primary)
                                .child(concat!("v", env!("CARGO_PKG_VERSION"))),
                        ),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(theme.muted_foreground)
                        .child("A local-first markdown notebook engineered for speed, offline reliability, and typography."),
                ),
        )
        .into_any_element();

    let project_section = SettingsSection::new("About")
        .subtitle("Source code repository and open source licensing")
        .child(
            SettingsRow::new("settings-about-github", "Source Code")
                .icon(IconName::Code)
                .subtitle("github.com/ToonionOfficial/tnotes")
                .value("GitHub")
                .on_press(|_, _| {
                    let _ = std::process::Command::new("xdg-open")
                        .arg("https://github.com/ToonionOfficial/tnotes")
                        .spawn();
                }),
        )
        .child(
            SettingsRow::new("settings-about-license", "License")
                .icon(IconName::Bookmark)
                .subtitle("Dual licensed under MIT and Apache-2.0")
                .value("MIT / Apache-2.0"),
        )
        .render_element(cx);

    let footer = div()
        .w_full()
        .pt(px(8.))
        .pb(px(20.))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(2.))
        .child(
            div()
                .text_size(px(11.))
                .text_color(theme.muted_foreground)
                .child(concat!("TNotes v", env!("CARGO_PKG_VERSION"))),
        );

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(hero_card)
        .child(project_section)
        .child(footer)
        .into_any_element()
}
