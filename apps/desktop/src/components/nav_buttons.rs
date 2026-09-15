use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;

#[derive(IntoElement)]
pub struct NavButtons {
    can_back: bool,
    can_forward: bool,
    on_back: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_forward: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl NavButtons {
    pub fn new(can_back: bool, can_forward: bool) -> Self {
        Self {
            can_back,
            can_forward,
            on_back: None,
            on_forward: None,
        }
    }

    pub fn on_back(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_back = Some(Box::new(handler));
        self
    }

    pub fn on_forward(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_forward = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for NavButtons {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_back = self.on_back;
        let on_forward = self.on_forward;

        div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .id("note-nav-back-btn")
                    .w(px(24.))
                    .h(px(24.))
                    .rounded(px(4.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.muted_foreground)
                    .when(self.can_back, |this| {
                        let mut el = this
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground));
                        if let Some(on_back) = on_back {
                            el = el.on_click(move |ev, window, cx| on_back(ev, window, cx));
                        }
                        el
                    })
                    .when(!self.can_back, |this| this.opacity(0.35).cursor_default())
                    .child(Icon::new(IconName::ArrowLeft).size(px(13.))),
            )
            .child(
                div()
                    .id("note-nav-forward-btn")
                    .w(px(24.))
                    .h(px(24.))
                    .rounded(px(4.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.muted_foreground)
                    .when(self.can_forward, |this| {
                        let mut el = this
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground));
                        if let Some(on_forward) = on_forward {
                            el = el.on_click(move |ev, window, cx| on_forward(ev, window, cx));
                        }
                        el
                    })
                    .when(!self.can_forward, |this| this.opacity(0.35).cursor_default())
                    .child(Icon::new(IconName::ArrowRight).size(px(13.))),
            )
    }
}
