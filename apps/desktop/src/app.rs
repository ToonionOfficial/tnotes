use gpui::*;
use crate::theme::{ActiveTheme, Theme, ThemeExt};

pub struct Tnotes {
    text: SharedString,
}

impl Render for Tnotes {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.text.clone())
    }
}

impl Tnotes {
    pub fn run_app() {
        Application::new().run(|cx: &mut App| {
            cx.set_global(ActiveTheme(Theme::dark()));

            let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_cx| Tnotes {
                        text: "Hello, World!".into(),
                    })
                },
            )
            .unwrap();

            cx.activate(true);
        });
    }
}
