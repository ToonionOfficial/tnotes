use gpui::*;
use crate::components::badge::KbdBadge;
use crate::components::icon::{Icon, IconName};
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InputVariant {
    #[default]
    Default,
    Sidebar,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InputSize {
    Sm,
    #[default]
    Default,
    Lg,
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct Input {
    id: ElementId,
    variant: InputVariant,
    size: InputSize,
    placeholder: SharedString,
    value: SharedString,
    leading_icon: Option<IconName>,
    trailing_kbd: Option<SharedString>,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl Input {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: InputVariant::Default,
            size: InputSize::Default,
            placeholder: "".into(),
            value: "".into(),
            leading_icon: None,
            trailing_kbd: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn sidebar_search(id: impl Into<ElementId>) -> Self {
        Self::new(id)
            .variant(InputVariant::Sidebar)
            .size(InputSize::Sm)
            .placeholder("Search notes...")
            .leading_icon(IconName::Search)
            .trailing_kbd("⌘K")
    }

    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_kbd(mut self, kbd: impl Into<SharedString>) -> Self {
        self.trailing_kbd = Some(kbd.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for Input {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let (height, padding_x, font_size, icon_size, radius) = match self.size {
            InputSize::Sm => (px(28.), px(8.), px(12.5), px(14.), px(6.)),
            InputSize::Default => (px(32.), px(10.), px(13.5), px(15.), px(6.)),
            InputSize::Lg => (px(40.), px(12.), px(15.0), px(18.), px(8.)),
        };

        let (bg, border_color) = match self.variant {
            InputVariant::Default => (theme.card, theme.border),
            InputVariant::Sidebar => (theme.card, theme.border),
        };

        let is_empty = self.value.is_empty();
        let display_text = if is_empty {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };
        let text_color = if is_empty {
            theme.muted_foreground
        } else {
            theme.foreground
        };

        let mut el = div()
            .id(self.id)
            .w_full()
            .h(height)
            .px(padding_x)
            .rounded(radius)
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .flex()
            .items_center()
            .justify_between()
            .text_size(font_size)
            .text_color(text_color);

        if self.disabled {
            el = el.opacity(0.5).cursor_not_allowed();
        } else {
            el = el
                .cursor_text()
                .hover(|s| s.border_color(theme.ring));
        }

        let leading = div()
            .flex()
            .items_center()
            .gap_2()
            .children(self.leading_icon.map(|name| {
                Icon::new(name)
                    .size(icon_size)
                    .color(theme.muted_foreground)
            }))
            .child(div().line_clamp(1).child(display_text));

        el = el.child(leading);

        if let Some(kbd) = self.trailing_kbd {
            el = el.child(KbdBadge::new(kbd));
        }

        if let Some(on_click) = self.on_click {
            if !self.disabled {
                el = el.on_click(move |ev, window, cx| on_click(ev, window, cx));
            }
        }

        el
    }
}
