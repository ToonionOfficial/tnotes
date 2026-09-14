use gpui::*;
use crate::assets::DesktopAssets;
use crate::components::{Button, IconName, Input};
use crate::theme::{ActiveTheme, Theme, ThemeExt};

#[allow(dead_code)]
pub struct Tnotes {
    text: SharedString,
}

impl Render for Tnotes {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                div()
                    .w(px(260.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Input::sidebar_search("search-input"))
                    .child(
                        Button::sidebar("notes-btn", "All Notes")
                            .leading_icon(IconName::FileText)
                            .count(42)
                            .active(true),
                    )
                    .child(
                        Button::sidebar("starred-btn", "Starred")
                            .leading_icon(IconName::Star)
                            .count(5),
                    )
                    .child(
                        Button::primary("create-btn", "New Note")
                            .leading_icon(IconName::Plus)
                            .full_width(true),
                    ),
            )
    }
}

impl Tnotes {
    pub fn run_app() {
        Application::new()
            .with_assets(DesktopAssets)
            .run(|cx: &mut App| {
                cx.set_global(ActiveTheme(Theme::dark()));

                let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);

                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_cx| Tnotes {
                            text: "TNotes".into(),
                        })
                    },
                )
                .unwrap();

                cx.activate(true);
            });
    }
}
