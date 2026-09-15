use gpui::*;
use crate::components::{Icon, IconName, NavButtons};
use crate::theme::ThemeExt;
use crate::views::SidebarView;

pub struct NoteView {
    sidebar: Entity<SidebarView>,
    _subscription: Subscription,
}

impl NoteView {
    pub fn new(sidebar: Entity<SidebarView>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&sidebar, |_, _, cx| {
            cx.notify();
        });
        Self {
            sidebar,
            _subscription: subscription,
        }
    }
}

impl Render for NoteView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (note, can_back, can_forward) = {
            let sidebar = self.sidebar.read(cx);
            (
                sidebar.selected_note().cloned(),
                sidebar.can_navigate_back(),
                sidebar.can_navigate_forward(),
            )
        };
        let theme = cx.theme().clone();

        let nav_buttons = NavButtons::new(can_back, can_forward)
            .on_back(cx.listener(|this, _, _window, cx| {
                this.sidebar.update(cx, |s, cx| {
                    s.navigate_back(cx);
                });
            }))
            .on_forward(cx.listener(|this, _, _window, cx| {
                this.sidebar.update(cx, |s, cx| {
                    s.navigate_forward(cx);
                });
            }));

        if let Some(note) = note {
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
                                .gap_3()
                                .child(nav_buttons)
                                .child(div().w(px(1.)).h(px(14.)).bg(theme.border))
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
                                ),
                        )
                        .child(
                            div()
                                .id("note-pin-btn")
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
                                .child(Icon::new(IconName::Star).size(px(14.)))
                                .on_click(cx.listener({
                                    let note_id = note.id.clone();
                                    move |this, _, _window, cx| {
                                        this.sidebar.update(cx, |s, cx| {
                                            s.toggle_note_pin(&note_id, cx);
                                        });
                                    }
                                })),
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
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .child(
                    div()
                        .w_full()
                        .h(px(48.))
                        .px(px(24.))
                        .flex()
                        .items_center()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(nav_buttons),
                )
                .child(
                    div()
                        .flex_1()
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
                        ),
                )
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use crate::theme::{ActiveTheme, Theme};

    #[test]
    fn note_view_renders_with_sidebar() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_view, cx) = cx.add_window_view(|_, cx| {
            let sidebar = cx.new(|cx| SidebarView::new(cx));
            NoteView::new(sidebar, cx)
        });
        cx.run_until_parked();
    }
}
