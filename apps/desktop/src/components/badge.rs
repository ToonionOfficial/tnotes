use gpui::*;
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
}

#[allow(dead_code)]
impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let (bg, text_color, border_color) = match self.variant {
            BadgeVariant::Default => (theme.primary, theme.primary_foreground, theme.primary),
            BadgeVariant::Secondary => (theme.secondary, theme.foreground, theme.border),
            BadgeVariant::Outline => (
                gpui::transparent_black(),
                theme.muted_foreground,
                theme.border,
            ),
        };

        div()
            .flex()
            .items_center()
            .px_2()
            .py_0p5()
            .rounded(px(4.))
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .text_size(px(11.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(text_color)
            .child(self.label)
    }
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct KbdBadge {
    label: SharedString,
}

#[allow(dead_code)]
impl KbdBadge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl RenderOnce for KbdBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .justify_center()
            .px_1p5()
            .py_0p5()
            .rounded(px(4.))
            .bg(theme.card)
            .border_1()
            .border_color(theme.border)
            .text_size(px(10.5))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.muted_foreground)
            .child(self.label)
    }
}
