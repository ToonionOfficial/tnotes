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
    focus_handle: Option<FocusHandle>,
    leading_icon: Option<IconName>,
    trailing_kbd: Option<SharedString>,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_key_down: Option<Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>>,
    on_clear: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
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
            focus_handle: None,
            leading_icon: None,
            trailing_kbd: None,
            disabled: false,
            on_click: None,
            on_key_down: None,
            on_clear: None,
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

    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
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

    pub fn on_key_down(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_key_down = Some(Box::new(handler));
        self
    }

    pub fn on_clear(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_clear = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let (height, padding_x, font_size, icon_size, radius) = match self.size {
            InputSize::Sm => (px(28.), px(8.), px(12.5), px(14.), px(6.)),
            InputSize::Default => (px(32.), px(10.), px(13.5), px(15.), px(6.)),
            InputSize::Lg => (px(40.), px(12.), px(15.0), px(18.), px(8.)),
        };

        let is_focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(false);

        let (bg, border_color) = match self.variant {
            InputVariant::Default => (theme.card, if is_focused { theme.ring } else { theme.border }),
            InputVariant::Sidebar => (theme.card, if is_focused { theme.ring } else { theme.border }),
        };

        let is_empty = self.value.is_empty();

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
            .text_size(font_size);

        if self.disabled {
            el = el.opacity(0.5).cursor_not_allowed();
        } else {
            el = el
                .cursor_text()
                .hover(|s| s.border_color(theme.ring))
                .focus(|s| s.border_color(theme.ring));
        }

        if let Some(ref focus_handle) = self.focus_handle {
            el = el.track_focus(focus_handle);
        }

        if !self.disabled {
            let focus_handle = self.focus_handle.clone();
            let on_click = self.on_click;
            if focus_handle.is_some() || on_click.is_some() {
                el = el.on_click(move |ev, window, cx| {
                    if let Some(ref h) = focus_handle {
                        h.focus(window);
                    }
                    if let Some(ref click) = on_click {
                        click(ev, window, cx);
                    }
                });
            }
        }

        if let Some(on_key_down) = self.on_key_down {
            el = el.on_key_down(move |event, window, cx| {
                on_key_down(event, window, cx);
            });
        }

        if is_focused {
            el = el.on_mouse_down_out(move |_event, window, _| {
                window.blur();
            });
        }

        let cursor = div()
            .w(px(1.5))
            .h(px(14.))
            .bg(theme.primary);

        let mut text_container = div().flex().items_center().overflow_hidden();

        if is_empty {
            if is_focused {
                text_container = text_container.child(cursor.mr(px(2.)));
            }
            text_container = text_container.child(
                div()
                    .text_color(theme.muted_foreground)
                    .line_clamp(1)
                    .child(self.placeholder.clone()),
            );
        } else {
            text_container = text_container.child(
                div()
                    .text_color(theme.foreground)
                    .line_clamp(1)
                    .child(self.value.clone()),
            );
            if is_focused {
                text_container = text_container.child(cursor.ml(px(1.)));
            }
        }

        let leading = div()
            .flex()
            .items_center()
            .gap_2()
            .flex_1()
            .overflow_hidden()
            .children(self.leading_icon.map(|name| {
                Icon::new(name)
                    .size(icon_size)
                    .color(if is_focused { theme.primary } else { theme.muted_foreground })
            }))
            .child(text_container);

        el = el.child(leading);

        if !is_empty && self.on_clear.is_some() {
            let on_clear = self.on_clear.unwrap();
            el = el.child(
                div()
                    .id("input-clear-btn")
                    .w(px(16.))
                    .h(px(16.))
                    .rounded(px(3.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                    .text_color(theme.muted_foreground)
                    .on_click(move |ev, window, cx| {
                        cx.stop_propagation();
                        on_clear(ev, window, cx);
                    })
                    .child(Icon::new(IconName::X).size(px(11.))),
            );
        } else if let Some(kbd) = self.trailing_kbd {
            el = el.child(KbdBadge::new(kbd));
        }

        el
    }
}
