use super::SettingsView;
use super::components::{SettingsRow, SettingsSection};
use crate::components::{Button, ButtonSize, ButtonVariant, Icon, IconName};
use crate::theme::ThemeExt;
use crate::updater::{ReleaseChannel, UpdateStatus};
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let theme = cx.theme().clone();

    let updater_entity = view.updater().clone();
    let updater_read = updater_entity.read(cx);
    let status = updater_read.status().clone();
    let settings = updater_read.settings().clone();

    let now_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let last_checked_str = match settings.last_checked {
        Some(timestamp) => {
            let diff = now_epoch.saturating_sub(timestamp);
            if diff < 60 {
                "Checked just now".to_string()
            } else if diff < 3600 {
                format!("Checked {} minutes ago", diff / 60)
            } else if diff < 86400 {
                format!("Checked {} hours ago", diff / 3600)
            } else {
                format!("Checked {} days ago", diff / 86400)
            }
        }
        None => "Never checked".to_string(),
    };

    // Hero / Status Card
    let updater_for_check = updater_entity.clone();
    let updater_for_download = updater_entity.clone();
    let updater_for_restart = updater_entity.clone();

    let (status_title, status_desc, is_error) = match &status {
        UpdateStatus::Idle => (
            "Up to Date",
            format!("TNotes is ready to check for updates. {last_checked_str}."),
            false,
        ),
        UpdateStatus::Checking => (
            "Checking for Updates",
            "Connecting to GitHub releases...".to_string(),
            false,
        ),
        UpdateStatus::UpToDate { .. } => (
            "You're Up to Date",
            format!("Running the latest release on the {} channel. {last_checked_str}.", settings.channel.title()),
            false,
        ),
        UpdateStatus::Available { version, size_bytes, .. } => {
            let mb = *size_bytes as f64 / (1024.0 * 1024.0);
            (
                "Update Available",
                format!("Version v{version} is available for download ({mb:.1} MB)."),
                false,
            )
        }
        UpdateStatus::Downloading { version, downloaded_bytes, total_bytes, .. } => {
            let dl_mb = *downloaded_bytes as f64 / (1024.0 * 1024.0);
            let tot_mb = *total_bytes as f64 / (1024.0 * 1024.0);
            (
                "Downloading Update",
                format!("Downloading v{version} ({dl_mb:.1} MB / {tot_mb:.1} MB)..."),
                false,
            )
        }
        UpdateStatus::Verifying => (
            "Verifying Update",
            "Validating SHA-256 package checksum integrity...".to_string(),
            false,
        ),
        UpdateStatus::ReadyToRestart { version, .. } => (
            "Update Ready",
            format!("Version v{version} has been verified and staged. Restart to complete update."),
            false,
        ),
        UpdateStatus::Error(msg) => (
            "Update Check Failed",
            msg.clone(),
            true,
        ),
    };

    let hero_icon = match &status {
        UpdateStatus::ReadyToRestart { .. } => IconName::Check,
        UpdateStatus::Available { .. } => IconName::Rocket,
        UpdateStatus::Downloading { .. } | UpdateStatus::Verifying => IconName::RefreshCw,
        UpdateStatus::Error(_) => IconName::CircleInfo,
        _ => IconName::Rocket,
    };

    let hero_icon_bg = if is_error {
        theme.destructive
    } else {
        theme.primary
    };

    let hero_card = div()
        .w_full()
        .rounded(px(10.))
        .bg(theme.card)
        .border_1()
        .border_color(if is_error { theme.destructive } else { theme.border })
        .p(px(20.))
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .w(px(48.))
                                .h(px(48.))
                                .rounded(px(12.))
                                .bg(hero_icon_bg)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    Icon::new(hero_icon)
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
                                                .child(status_title),
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
                                        .text_color(if is_error { theme.destructive } else { theme.muted_foreground })
                                        .child(status_desc),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(match &status {
                            UpdateStatus::Checking => {
                                Button::secondary("btn-checking", "Checking...")
                                    .size(ButtonSize::Sm)
                                    .disabled(true)
                                    .into_any_element()
                            }
                            UpdateStatus::Downloading { .. } => {
                                Button::secondary("btn-downloading", "Downloading...")
                                    .size(ButtonSize::Sm)
                                    .disabled(true)
                                    .into_any_element()
                            }
                            UpdateStatus::Verifying => {
                                Button::secondary("btn-verifying", "Verifying...")
                                    .size(ButtonSize::Sm)
                                    .disabled(true)
                                    .into_any_element()
                            }
                            UpdateStatus::Available { .. } => {
                                Button::primary("btn-download-update", "Download Update")
                                    .size(ButtonSize::Sm)
                                    .leading_icon(IconName::Rocket)
                                    .on_click(move |_, _, cx| {
                                        updater_for_download.update(cx, |manager, cx| {
                                            manager.start_download(cx);
                                        });
                                    })
                                    .into_any_element()
                            }
                            UpdateStatus::ReadyToRestart { .. } => {
                                Button::primary("btn-restart-update", "Restart & Install")
                                    .size(ButtonSize::Sm)
                                    .leading_icon(IconName::Check)
                                    .on_click(move |_, _, cx| {
                                        updater_for_restart.update(cx, |manager, cx| {
                                            manager.install_and_restart(cx);
                                        });
                                    })
                                    .into_any_element()
                            }
                            _ => {
                                Button::secondary("btn-check-updates", "Check for Updates")
                                    .size(ButtonSize::Sm)
                                    .leading_icon(IconName::RefreshCw)
                                    .on_click(move |_, _, cx| {
                                        updater_for_check.update(cx, |manager, cx| {
                                            manager.check_for_updates(false, cx);
                                        });
                                    })
                                    .into_any_element()
                            }
                        }),
                ),
        )
        .when_some(
            match &status {
                UpdateStatus::Downloading { progress, .. } => Some(*progress),
                _ => None,
            },
            |this, progress| {
                let percent = (progress * 100.0).round() as u32;
                this.child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .w_full()
                                .h(px(6.))
                                .rounded_full()
                                .bg(theme.secondary)
                                .overflow_hidden()
                                .child(
                                    div()
                                        .h_full()
                                        .w(DefiniteLength::Fraction(progress.clamp(0.01, 1.0)))
                                        .bg(theme.primary)
                                        .rounded_full(),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(theme.muted_foreground)
                                .child(format!("{percent}% complete")),
                        ),
                )
            },
        )
        .when_some(
            match &status {
                UpdateStatus::Available { changelog_url, .. }
                | UpdateStatus::ReadyToRestart { changelog_url, .. } => changelog_url.clone(),
                _ => None,
            },
            |this, url| {
                let release_url = url.clone();
                this.child(
                    div()
                        .pt_2()
                        .border_t_1()
                        .border_color(theme.border)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(theme.muted_foreground)
                                .child("View the release notes and changelog on GitHub"),
                        )
                        .child(
                            Button::new("btn-view-changelog")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .label("Release Notes")
                                .trailing_icon(IconName::ChevronRight)
                                .on_click(move |_, _, _| {
                                    let _ = std::process::Command::new("xdg-open")
                                        .arg(&release_url)
                                        .spawn();
                                }),
                        ),
                )
            },
        );

    // Release Channel Selection Card
    let mut channel_section = SettingsSection::new("Release Channel")
        .subtitle("Select which update stream your installation subscribes to");

    for ch in ReleaseChannel::ALL {
        let is_selected = settings.channel == ch;
        let updater_ch = updater_entity.clone();

        let channel_badge = match ch {
            ReleaseChannel::Stable => "Recommended",
            ReleaseChannel::Beta => "Preview",
            ReleaseChannel::Alpha => "Bleeding Edge",
        };

        let row = div()
            .id(SharedString::from(format!("channel-row-{}", ch.title().to_lowercase())))
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .flex()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .hover(|s| s.bg(theme.secondary))
            .when(is_selected, |s| s.bg(theme.secondary.opacity(0.6)))
            .on_click(move |_, _, cx| {
                updater_ch.update(cx, |manager, cx| {
                    manager.set_channel(ch, cx);
                });
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(20.))
                            .h(px(20.))
                            .rounded_full()
                            .border_1()
                            .border_color(if is_selected { theme.primary } else { theme.muted_foreground })
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .w(px(10.))
                                    .h(px(10.))
                                    .rounded_full()
                                    .when(is_selected, |s| s.bg(theme.primary)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(1.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(13.5))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.foreground)
                                            .child(ch.title()),
                                    )
                                    .child(
                                        div()
                                            .px_1p5()
                                            .py(px(1.))
                                            .rounded(px(3.))
                                            .bg(theme.muted)
                                            .text_size(px(10.5))
                                            .font_weight(FontWeight::NORMAL)
                                            .text_color(theme.muted_foreground)
                                            .child(channel_badge),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(theme.muted_foreground)
                                    .child(ch.description()),
                            ),
                    ),
            )
            .child(
                div()
                    .when(is_selected, |this| {
                        this.child(
                            Icon::new(IconName::Check)
                                .size(px(16.))
                                .color(theme.primary),
                        )
                    }),
            );

        channel_section = channel_section.child(row);
    }

    let channel_element = channel_section.render_element(cx);

    // Automation Preferences Section
    let updater_for_launch_toggle = updater_entity.clone();
    let updater_for_download_toggle = updater_entity.clone();

    let auto_section = SettingsSection::new("Automation Preferences")
        .subtitle("Configure startup checks and automatic download behavior")
        .child(
            SettingsRow::new("settings-update-launch", "Check for Updates on Launch")
                .icon(IconName::RefreshCw)
                .subtitle("Automatically query release feeds on startup (cooldown: 4 hours)")
                .switch(settings.check_on_launch)
                .on_toggle(move |new_val, _, cx| {
                    updater_for_launch_toggle.update(cx, |manager, cx| {
                        manager.set_check_on_launch(new_val, cx);
                    });
                }),
        )
        .child(
            SettingsRow::new("settings-update-autodownload", "Automatically Download Updates")
                .icon(IconName::Rocket)
                .subtitle("Download verified update packages in the background when discovered")
                .switch(settings.auto_download)
                .on_toggle(move |new_val, _, cx| {
                    updater_for_download_toggle.update(cx, |manager, cx| {
                        manager.set_auto_download(new_val, cx);
                    });
                }),
        )
        .render_element(cx);

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(hero_card)
        .child(channel_element)
        .child(auto_section)
        .into_any_element()
}
