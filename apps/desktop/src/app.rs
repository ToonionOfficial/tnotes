use gpui::*;
use crate::assets::DesktopAssets;
use crate::components::{Icon, IconName};
use crate::theme::{ActiveTheme, Theme, ThemeExt};
use crate::views::SidebarView;

pub struct Tnotes {
    sidebar: Entity<SidebarView>,
    _subscriptions: Vec<Subscription>,
}

impl Render for Tnotes {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let right_pane = if let Some(note) = self.sidebar.read(cx).selected_note() {
            let folder_label = note
                .folder_name
                .clone()
                .unwrap_or_else(|| "Notes".to_string());

            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .bg(theme.background)
                .child(
                    div()
                        .w_full()
                        .h(px(48.))
                        .px(px(24.))
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_size(px(12.))
                                .text_color(theme.muted_foreground)
                                .child("TNotes")
                                .child(div().text_size(px(10.)).child("/"))
                                .child(folder_label)
                                .child(div().text_size(px(10.)).child("/"))
                                .child(
                                    div()
                                        .text_color(theme.foreground)
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(note.title.clone()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .w(px(28.))
                                        .h(px(28.))
                                        .rounded(px(5.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                                        .text_color(if note.is_pinned {
                                            theme.primary
                                        } else {
                                            theme.muted_foreground
                                        })
                                        .child(Icon::new(IconName::Star).size(px(14.))),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("note-reader-scroll")
                        .flex_1()
                        .overflow_y_scroll()
                        .px(px(48.))
                        .py(px(36.))
                        .child(
                            div()
                                .max_w(px(760.))
                                .w_full()
                                .mx_auto()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(
                                    div()
                                        .text_size(px(28.))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.foreground)
                                        .child(note.title.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_size(px(12.))
                                        .text_color(theme.muted_foreground)
                                        .child(format!("Last edited {}", note.updated_at)),
                                )
                                .child(div().w_full().h(px(1.)).bg(theme.border))
                                .child(
                                    div()
                                        .text_size(px(15.))
                                        .text_color(theme.foreground)
                                        .line_height(px(24.))
                                        .child(note.snippet.clone()),
                                ),
                        ),
                )
        } else {
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .bg(theme.background)
                .child(
                    Icon::new(IconName::FileText)
                        .size(px(36.))
                        .color(theme.muted_foreground),
                )
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("No note selected"),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.muted_foreground)
                        .child("Select a note from the sidebar or click 'New Note' to create one"),
                )
        };

        div()
            .size_full()
            .flex()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.sidebar.clone())
            .child(right_pane)
    }
}

impl Tnotes {
    pub fn run_app() {
        Application::new()
            .with_assets(DesktopAssets)
            .run(|cx: &mut App| {
                cx.set_global(ActiveTheme(Theme::dark()));

                let bounds = Bounds::centered(None, size(px(1080.), px(720.)), cx);

                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |_, cx| {
                        let sidebar = cx.new(|cx| SidebarView::new(cx));

                        cx.new(|cx| {
                            let mut subscriptions = Vec::new();

                            subscriptions.push(cx.observe(&sidebar, |_, _, cx| {
                                cx.notify();
                            }));

                            Tnotes {
                                sidebar,
                                _subscriptions: subscriptions,
                            }
                        })
                    },
                )
                .unwrap();

                cx.activate(true);
            });
    }
}
