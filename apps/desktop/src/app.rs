use gpui::*;
use crate::assets::DesktopAssets;
use crate::theme::{ActiveTheme, Theme, ThemeExt};
use crate::views::SidebarView;

pub struct Tnotes {
    sidebar: Entity<SidebarView>,
}

impl Render for Tnotes {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .size_full()
            .flex()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.sidebar.clone())
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(24.))
                            .font_weight(FontWeight::BOLD)
                            .child("TNotes"),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(theme.muted_foreground)
                            .child("Select or create a note to begin"),
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

                let bounds = Bounds::centered(None, size(px(1000.), px(700.)), cx);

                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |_, cx| {
                        let sidebar = cx.new(|cx| SidebarView::new(cx));
                        cx.new(|_| Tnotes { sidebar })
                    },
                )
                .unwrap();

                cx.activate(true);
            });
    }
}
