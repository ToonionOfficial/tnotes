use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;

/// Section identifier for the settings nav + content area.
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
    pub const ALL: [Self; 7] = [
        Self::Account,
        Self::Sync,
        Self::Appearance,
        Self::Storage,
        Self::Keybindings,
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

/// Titled card wrapper for a group of settings rows.
pub struct SettingsSection {
    title: SharedString,
    children: Vec<AnyElement>,
}

impl SettingsSection {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub(crate) fn render_element(self, cx: &mut App) -> AnyElement {
        let theme = cx.theme().clone();
        div()
            .w_full()
            .rounded(px(10.))
            .bg(theme.card)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .px(px(16.))
                    .pt(px(14.))
                    .pb(px(8.))
                    .text_size(px(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.muted_foreground)
                    .child(self.title),
            )
            .children(self.children)
            .into_any_element()
    }
}

impl RenderOnce for SettingsSection {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .w_full()
            .rounded(px(10.))
            .bg(theme.card)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .px(px(16.))
                    .pt(px(14.))
                    .pb(px(8.))
                    .text_size(px(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.muted_foreground)
                    .child(self.title),
            )
            .children(self.children)
    }
}

/// One settings row: icon + title/subtitle on the left, and a value, chevron,
/// or toggle switch on the right.
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

        let mut row = div()
            .id(self.id)
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
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

        if let Some(on) = self.switch_on {
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
