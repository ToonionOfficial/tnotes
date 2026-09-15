use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSectionId {
    Account,
    Sync,
    Appearance,
    Storage,
    Keybindings,
    Developer,
    About,
}

impl SettingsSectionId {
    #[allow(dead_code)]
    pub const ALL: [Self; 7] = [
        Self::Account,
        Self::Sync,
        Self::Appearance,
        Self::Storage,
        Self::Keybindings,
        Self::Developer,
        Self::About,
    ];

    pub const WORKSPACE: [Self; 3] = [
        Self::Appearance,
        Self::Account,
        Self::Keybindings,
    ];

    pub const SYSTEM: [Self; 4] = [
        Self::Sync,
        Self::Storage,
        Self::Developer,
        Self::About,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Sync => "Sync",
            Self::Appearance => "Appearance",
            Self::Storage => "Storage",
            Self::Keybindings => "Keybindings",
            Self::Developer => "Developer",
            Self::About => "About",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Appearance => "Customize theme mode, editor typography, and document reading comfort.",
            Self::Account => "Personal vault identity, filesystem location, and storage overview.",
            Self::Keybindings => "Keyboard shortcuts, quick search, command navigation, and key capture.",
            Self::Sync => "Pairing status, sync server connection, and synchronization engine.",
            Self::Storage => "Local database metrics, index maintenance, and vault exports.",
            Self::Developer => "GPUI engine diagnostics, frame budgets, and runtime telemetry.",
            Self::About => "Version information, system architecture, and project repository.",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Self::Account => IconName::User,
            Self::Sync => IconName::RefreshCw,
            Self::Appearance => IconName::Palette,
            Self::Storage => IconName::Database,
            Self::Keybindings => IconName::Keyboard,
            Self::Developer => IconName::Code,
            Self::About => IconName::CircleInfo,
        }
    }
}

pub struct SettingsSection {
    title: SharedString,
    subtitle: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl SettingsSection {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            children: Vec::new(),
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub(crate) fn render_element(self, cx: &mut App) -> AnyElement {
        let theme = cx.theme().clone();
        let has_subtitle = self.subtitle.is_some();

        let mut body = div()
            .w_full()
            .rounded(px(10.))
            .bg(theme.card)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .px(px(16.))
                    .pt(px(14.))
                    .pb(if has_subtitle { px(4.) } else { px(8.) })
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground)
                            .child(self.title),
                    )
                    .children(self.subtitle.map(|sub| {
                        div()
                            .text_size(px(12.))
                            .text_color(theme.muted_foreground)
                            .child(sub)
                    })),
            );

        for (idx, child) in self.children.into_iter().enumerate() {
            body = body.child(
                div()
                    .when(idx > 0, |this| this.border_t_1().border_color(theme.border))
                    .child(child),
            );
        }

        body.into_any_element()
    }
}

impl RenderOnce for SettingsSection {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.render_element(cx)
    }
}

#[derive(IntoElement)]
pub struct SettingsRow {
    id: ElementId,
    icon: Option<IconName>,
    title: SharedString,
    subtitle: Option<SharedString>,
    value: Option<SharedString>,
    destructive: bool,
    disabled: bool,
    switch_on: Option<bool>,
    custom_trailing: Option<AnyElement>,
    on_toggle: Option<ToggleHandler>,
    on_press: Option<PressHandler>,
}

type ToggleHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;
type PressHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;

impl SettingsRow {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            title: title.into(),
            subtitle: None,
            value: None,
            destructive: false,
            disabled: false,
            switch_on: None,
            custom_trailing: None,
            on_toggle: None,
            on_press: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn switch(mut self, on: bool) -> Self {
        self.switch_on = Some(on);
        self
    }

    pub fn custom_trailing(mut self, el: impl IntoElement) -> Self {
        self.custom_trailing = Some(el.into_any_element());
        self
    }

    #[allow(dead_code)]
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_press(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SettingsRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let title_color = if self.destructive {
            theme.destructive
        } else if self.disabled {
            theme.muted_foreground
        } else {
            theme.foreground
        };

        let is_clickable = !self.disabled && self.on_press.is_some();

        let mut row = div()
            .id(self.id)
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .when(is_clickable, |this| {
                this.cursor_pointer().hover(|s| s.bg(theme.secondary))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .flex_1()
                    .overflow_hidden()
                    .children(self.icon.map(|icon| {
                        div()
                            .w(px(32.))
                            .h(px(32.))
                            .rounded(px(8.))
                            .bg(theme.secondary)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(icon)
                                    .size(px(15.))
                                    .color(theme.muted_foreground),
                            )
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .flex_1()
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(title_color)
                                    .child(self.title),
                            )
                            .children(self.subtitle.map(|subtitle| {
                                div()
                                    .text_size(px(12.))
                                    .text_color(theme.muted_foreground)
                                    .child(subtitle)
                            })),
                    ),
            );

        if let Some(trailing) = self.custom_trailing {
            row = row.child(trailing);
        } else if let Some(on) = self.switch_on {
            let on_toggle = self.on_toggle;
            row = row.child(
                div()
                    .id(SharedString::from(format!(
                        "settings-toggle-{}",
                        on_toggle.is_some()
                    )))
                    .w(px(40.))
                    .h(px(22.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .px(px(3.))
                    .justify_start()
                    .when(on, |this| this.justify_end())
                    .bg(if on { theme.primary } else { theme.muted })
                    .cursor_pointer()
                    .when(on_toggle.is_some(), |this| {
                        this.on_click(move |_, window, cx| {
                            if let Some(handler) = on_toggle.as_ref() {
                                handler(!on, window, cx);
                            }
                        })
                    })
                    .child(
                        div()
                            .w(px(16.))
                            .h(px(16.))
                            .rounded_full()
                            .bg(theme.primary_foreground),
                    ),
            );
        } else {
            row = row
                .children(self.value.map(|value| {
                    div()
                        .text_size(px(12.5))
                        .text_color(theme.muted_foreground)
                        .child(value)
                }))
                .children(self.on_press.is_some().then(|| {
                    Icon::new(IconName::ChevronRight)
                        .size(px(14.))
                        .color(theme.muted_foreground)
                }));
        }

        if !self.disabled
            && let Some(on_press) = self.on_press
        {
            row = row.cursor_pointer().on_click(move |_, window, cx| {
                on_press(window, cx);
            });
        }

        row
    }
}

#[derive(IntoElement)]
pub struct SettingsTelemetry {
    title: SharedString,
    path: SharedString,
    total_size: SharedString,
    active_count: usize,
    active_size: SharedString,
    trashed_count: usize,
    trashed_size: SharedString,
    on_reveal: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl SettingsTelemetry {
    pub fn new(title: impl Into<SharedString>, path: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            path: path.into(),
            total_size: "0 B".into(),
            active_count: 0,
            active_size: "0 B".into(),
            trashed_count: 0,
            trashed_size: "0 B".into(),
            on_reveal: None,
        }
    }

    pub fn total_size(mut self, size: impl Into<SharedString>) -> Self {
        self.total_size = size.into();
        self
    }

    pub fn active_notes(mut self, count: usize, size: impl Into<SharedString>) -> Self {
        self.active_count = count;
        self.active_size = size.into();
        self
    }

    pub fn trashed_notes(mut self, count: usize, size: impl Into<SharedString>) -> Self {
        self.trashed_count = count;
        self.trashed_size = size.into();
        self
    }

    pub fn on_reveal(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reveal = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SettingsTelemetry {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .w_full()
            .rounded(px(10.))
            .bg(theme.card)
            .border_1()
            .border_color(theme.border)
            .p(px(16.))
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
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(24.))
                                    .h(px(24.))
                                    .rounded(px(6.))
                                    .bg(theme.secondary)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        Icon::new(IconName::Database)
                                            .size(px(13.))
                                            .color(theme.primary),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.foreground)
                                    .child(self.title),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
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
                                    .child("SQLite WAL"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.foreground)
                                    .child(self.total_size),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .px(px(10.))
                    .py(px(6.))
                    .rounded(px(6.))
                    .bg(theme.background)
                    .border_1()
                    .border_color(theme.border)
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.muted_foreground)
                            .overflow_hidden()
                            .child(self.path),
                    )
                    .children(self.on_reveal.map(|handler| {
                        div()
                            .id("telemetry-reveal-btn")
                            .px_2()
                            .py(px(3.))
                            .rounded(px(4.))
                            .bg(theme.secondary)
                            .hover(|s| s.bg(theme.card))
                            .cursor_pointer()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground)
                            .child("Open Directory")
                            .on_click(move |_, window, cx| {
                                handler(window, cx);
                            })
                    })),
            )
            .child(
                div()
                    .w_full()
                    .h(px(6.))
                    .rounded_full()
                    .bg(theme.secondary)
                    .overflow_hidden()
                    .flex()
                    .child(div().flex_grow().bg(theme.primary))
                    .when(self.trashed_count > 0, |this| {
                        this.child(div().w(px(20.)).bg(theme.destructive))
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .text_size(px(11.5))
                    .text_color(theme.muted_foreground)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(5.))
                            .child(div().w(px(6.)).h(px(6.)).rounded_full().bg(theme.primary))
                            .child(format!(
                                "{} active notes ({})",
                                self.active_count, self.active_size
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(5.))
                            .child(div().w(px(6.)).h(px(6.)).rounded_full().bg(theme.destructive))
                            .child(format!(
                                "{} in trash ({})",
                                self.trashed_count, self.trashed_size
                            )),
                    ),
            )
    }
}

#[derive(IntoElement)]
pub struct SettingsSegmented {
    id: ElementId,
    options: Vec<SettingsSegmentedOption>,
}

pub struct SettingsSegmentedOption {
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub active: bool,
    pub on_click: Box<dyn Fn(&mut Window, &mut App) + 'static>,
}

impl SettingsSegmented {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            options: Vec::new(),
        }
    }

    pub fn option(
        mut self,
        label: impl Into<SharedString>,
        icon: Option<IconName>,
        active: bool,
        on_click: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.options.push(SettingsSegmentedOption {
            label: label.into(),
            icon,
            active,
            on_click: Box::new(on_click),
        });
        self
    }
}

impl RenderOnce for SettingsSegmented {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .id(self.id)
            .p(px(3.))
            .rounded(px(8.))
            .bg(theme.secondary)
            .border_1()
            .border_color(theme.border)
            .flex()
            .items_center()
            .gap(px(2.))
            .children(self.options.into_iter().enumerate().map(|(idx, opt)| {
                let active = opt.active;
                let on_click = opt.on_click;
                div()
                    .id(SharedString::from(format!("segmented-opt-{}", idx)))
                    .px_3()
                    .py(px(5.))
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .bg(if active {
                        theme.card
                    } else {
                        gpui::transparent_black()
                    })
                    .border_1()
                    .border_color(if active {
                        theme.border
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(|s| {
                        if !active {
                            s.bg(theme.muted)
                        } else {
                            s
                        }
                    })
                    .text_size(px(12.5))
                    .font_weight(if active {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(if active {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    })
                    .children(opt.icon.map(|icon| {
                        Icon::new(icon)
                            .size(px(14.))
                            .color(if active {
                                theme.primary
                            } else {
                                theme.muted_foreground
                            })
                    }))
                    .child(opt.label)
                    .on_click(move |_, window, cx| {
                        on_click(window, cx);
                    })
            }))
    }
}
