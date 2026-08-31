use crate::components::icons::*;
use crate::state::{AppState, SettingsTab};
use gpui::prelude::FluentBuilder;
use gpui::*;

pub fn settings_sidebar(
    app_state: Entity<AppState>,
    current_tab: SettingsTab,
    on_back: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .w_80()
        .h_full()
        .flex_shrink_0()
        .bg(rgb(0x141318))
        .border_r_1()
        .border_color(rgb(0x302e36))
        .p_2p5()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .id("settings-sidebar-header")
                .flex()
                .items_center()
                .justify_between()
                .px_2()
                .py_1p5()
                .child(
                    div()
                        .id("settings-back-btn")
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(0x201f24))
                        .hover(|s| s.bg(rgb(0x2a2930)))
                        .cursor_pointer()
                        .on_click(on_back)
                        .child(icon_arrow_left().size_4().text_color(rgb(0xcabeff)))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xe6e1e9))
                                .child("Back to Notes"),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x79747e))
                        .child("SETTINGS"),
                ),
        )
        .child(
            div()
                .id("settings-nav-tree")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(settings_nav_row(
                    "nav-account",
                    icon_user().size_4(),
                    "Account & Profile",
                    current_tab == SettingsTab::Account,
                    {
                        let state = app_state.clone();
                        move |_e, _w, cx| state.update(cx, |s, cx| s.set_settings_tab(SettingsTab::Account, cx))
                    },
                ))
                .child(settings_nav_row(
                    "nav-sync",
                    icon_sync().size_4(),
                    "Sync & Server",
                    current_tab == SettingsTab::Sync,
                    {
                        let state = app_state.clone();
                        move |_e, _w, cx| state.update(cx, |s, cx| s.set_settings_tab(SettingsTab::Sync, cx))
                    },
                ))
                .child(settings_nav_row(
                    "nav-appearance",
                    icon_palette().size_4(),
                    "Appearance & Theme",
                    current_tab == SettingsTab::Appearance,
                    {
                        let state = app_state.clone();
                        move |_e, _w, cx| state.update(cx, |s, cx| s.set_settings_tab(SettingsTab::Appearance, cx))
                    },
                ))
                .child(settings_nav_row(
                    "nav-storage",
                    icon_database().size_4(),
                    "Storage & Database",
                    current_tab == SettingsTab::Storage,
                    {
                        let state = app_state.clone();
                        move |_e, _w, cx| state.update(cx, |s, cx| s.set_settings_tab(SettingsTab::Storage, cx))
                    },
                ))
                .child(settings_nav_row(
                    "nav-keyboard",
                    icon_keyboard().size_4(),
                    "Keyboard Shortcuts",
                    current_tab == SettingsTab::Keyboard,
                    {
                        let state = app_state.clone();
                        move |_e, _w, cx| state.update(cx, |s, cx| s.set_settings_tab(SettingsTab::Keyboard, cx))
                    },
                ))
                .child(settings_nav_row(
                    "nav-developer",
                    icon_cpu().size_4(),
                    "Developer Tools",
                    current_tab == SettingsTab::Developer,
                    {
                        let state = app_state.clone();
                        move |_e, _w, cx| state.update(cx, |s, cx| s.set_settings_tab(SettingsTab::Developer, cx))
                    },
                ))
                .child(settings_nav_row(
                    "nav-about",
                    icon_info().size_4(),
                    "About TNotes",
                    current_tab == SettingsTab::About,
                    {
                        let state = app_state.clone();
                        move |_e, _w, cx| state.update(cx, |s, cx| s.set_settings_tab(SettingsTab::About, cx))
                    },
                )),
        )
}

fn settings_nav_row(
    id: &'static str,
    icon: Svg,
    label: impl Into<SharedString>,
    is_active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let label = label.into();
    let icon_color = if is_active {
        rgb(0xcabeff)
    } else {
        rgb(0x938f99)
    };

    div()
        .id(id)
        .flex()
        .items_center()
        .gap_2p5()
        .px_2()
        .py_1p5()
        .rounded_md()
        .when(is_active, |s| {
            s.bg(rgb(0x201f24))
        })
        .when(!is_active, |s| {
            s.hover(|s| s.bg(rgb(0x201f24)))
        })
        .cursor_pointer()
        .on_click(on_click)
        .child(
            div()
                .w_4()
                .h_4()
                .flex()
                .items_center()
                .justify_center()
                .child(icon.text_color(icon_color)),
        )
        .child(
            div()
                .text_xs()
                .font_weight(if is_active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(if is_active {
                    rgb(0xe6e1e9)
                } else {
                    rgb(0xc6c2cd)
                })
                .child(label),
        )
}
