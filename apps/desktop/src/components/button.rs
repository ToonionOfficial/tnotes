use gpui::*;
use crate::components::icon::{Icon, IconName};
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Sidebar,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Sm,
    #[default]
    Default,
    Lg,
    Icon,
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    variant: ButtonVariant,
    size: ButtonSize,
    label: Option<SharedString>,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    count: Option<usize>,
    active: bool,
    disabled: bool,
    full_width: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            label: None,
            leading_icon: None,
            trailing_icon: None,
            count: None,
            active: false,
            disabled: false,
            full_width: false,
            on_click: None,
        }
    }

    pub fn primary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .variant(ButtonVariant::Default)
            .label(label)
    }

    pub fn secondary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .variant(ButtonVariant::Secondary)
            .label(label)
    }

    pub fn ghost(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .variant(ButtonVariant::Ghost)
            .label(label)
    }

    pub fn outline(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .variant(ButtonVariant::Outline)
            .label(label)
    }

    pub fn icon(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self::new(id)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Icon)
            .leading_icon(icon)
    }

    pub fn sidebar(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .variant(ButtonVariant::Sidebar)
            .size(ButtonSize::Sm)
            .full_width(true)
            .label(label)
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let (height, padding_x, font_size, icon_size, radius) = match self.size {
            ButtonSize::Sm => (px(28.), px(8.), px(12.5), px(14.), px(6.)),
            ButtonSize::Default => (px(32.), px(12.), px(13.5), px(16.), px(6.)),
            ButtonSize::Lg => (px(40.), px(16.), px(15.0), px(18.), px(8.)),
            ButtonSize::Icon => (px(28.), px(0.), px(13.5), px(16.), px(6.)),
        };

        let (bg, hover_bg, text_color, hover_text_color, border_color) = match self.variant {
            ButtonVariant::Default => {
                let mut hover = theme.primary;
                hover.l = (hover.l + 0.06).min(1.0);
                (
                    theme.primary,
                    hover,
                    theme.primary_foreground,
                    theme.primary_foreground,
                    theme.primary,
                )
            }
            ButtonVariant::Secondary => {
                let bg = if self.active {
                    theme.secondary
                } else {
                    theme.card
                };
                (
                    bg,
                    theme.secondary,
                    theme.foreground,
                    theme.foreground,
                    theme.border,
                )
            }
            ButtonVariant::Outline => {
                let bg = if self.active {
                    theme.secondary
                } else {
                    gpui::transparent_black()
                };
                (
                    bg,
                    theme.secondary,
                    theme.foreground,
                    theme.foreground,
                    theme.border,
                )
            }
            ButtonVariant::Ghost => {
                let bg = if self.active {
                    theme.secondary
                } else {
                    gpui::transparent_black()
                };
                (
                    bg,
                    theme.secondary,
                    theme.foreground,
                    theme.foreground,
                    gpui::transparent_black(),
                )
            }
            ButtonVariant::Destructive => {
                let mut hover = theme.destructive;
                hover.l = (hover.l + 0.05).min(1.0);
                (
                    theme.destructive,
                    hover,
                    theme.destructive_foreground,
                    theme.destructive_foreground,
                    theme.destructive,
                )
            }
            ButtonVariant::Sidebar => {
                let bg = if self.active {
                    theme.secondary
                } else {
                    gpui::transparent_black()
                };
                let text = if self.active {
                    theme.foreground
                } else {
                    theme.muted_foreground
                };
                (
                    bg,
                    theme.secondary,
                    text,
                    theme.foreground,
                    gpui::transparent_black(),
                )
            }
        };

        let mut el = div()
            .id(self.id)
            .h(height)
            .rounded(radius)
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .text_size(font_size)
            .text_color(text_color);

        if self.size == ButtonSize::Icon {
            el = el
                .w(height)
                .flex()
                .items_center()
                .justify_center();
        } else {
            el = el
                .px(padding_x)
                .flex()
                .items_center();

            if self.variant == ButtonVariant::Sidebar {
                el = el.justify_between();
            } else {
                el = el.justify_center();
            }
        }

        if self.full_width {
            el = el.w_full();
        }

        if self.disabled {
            el = el.opacity(0.5).cursor_not_allowed();
        } else {
            el = el
                .cursor_pointer()
                .hover(|s| s.bg(hover_bg).border_color(hover_bg).text_color(hover_text_color));
        }

        let content = div()
            .flex()
            .items_center()
            .gap_2()
            .children(self.leading_icon.map(|name| {
                let icon_color = if self.active && self.variant == ButtonVariant::Sidebar {
                    theme.primary
                } else {
                    text_color
                };
                Icon::new(name).size(icon_size).color(icon_color)
            }))
            .children(self.label.map(|label| div().child(label)))
            .children(self.trailing_icon.map(|name| {
                Icon::new(name).size(icon_size).color(text_color)
            }));

        el = el.child(content);

        if let Some(count) = self.count {
            el = el.child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.muted_foreground)
                    .child(count.to_string()),
            );
        }

        if let Some(on_click) = self.on_click {
            if !self.disabled {
                el = el.on_click(move |ev, window, cx| on_click(ev, window, cx));
            }
        }

        el
    }
}
