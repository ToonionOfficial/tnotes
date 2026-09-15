use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::components::{Icon, IconName, NavButtons};
use crate::theme::ThemeExt;
use crate::views::{NoteItem, SidebarView};

pub struct StarredView {
    sidebar: Entity<SidebarView>,
    _subscription: Subscription,
}

impl StarredView {
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

impl Render for StarredView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (starred_notes, can_back, can_forward) = {
            let sidebar = self.sidebar.read(cx);
            (
                sidebar.starred_notes().into_iter().cloned().collect::<Vec<NoteItem>>(),
                sidebar.can_navigate_back(),
                sidebar.can_navigate_forward(),
            )
        };
        let theme = cx.theme().clone();
        let count = starred_notes.len();

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
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .text_color(theme.foreground)
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(Icon::new(IconName::Star).size(px(13.)).color(theme.primary))
                                            .child("Starred"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .px_2()
                            .py(px(2.))
                            .rounded(px(10.))
                            .bg(theme.secondary)
                            .text_size(px(11.))
                            .text_color(theme.muted_foreground)
                            .child(format!("{count} note{}", if count == 1 { "" } else { "s" })),
                    ),
            )
            .child(
                div()
                    .id("starred-pane-scroll")
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
                            .gap_5()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .w(px(36.))
                                            .h(px(36.))
                                            .rounded(px(8.))
                                            .bg(theme.secondary)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(Icon::new(IconName::Star).size(px(18.)).color(theme.primary)),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_size(px(24.))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme.foreground)
                                                    .child("Starred Notes"),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(13.))
                                                    .text_color(theme.muted_foreground)
                                                    .child("Quick access to your important and favorite notes"),
                                            ),
                                    ),
                            )
                            .child(div().w_full().h(px(1.)).bg(theme.border))
                            .when(starred_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py(px(64.))
                                        .gap_3()
                                        .child(
                                            Icon::new(IconName::Star)
                                                .size(px(40.))
                                                .color(theme.muted_foreground),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme.foreground)
                                                .child("No starred notes yet"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(13.))
                                                .text_color(theme.muted_foreground)
                                                .child("Star any note to quickly find it here."),
                                        ),
                                )
                            })
                            .when(!starred_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2p5()
                                        .children(starred_notes.iter().map(|note| {
                                            let note_id = note.id.clone();
                                            div()
                                                .id(SharedString::from(format!("starred-card-{}", note.id)))
                                                .w_full()
                                                .p(px(14.))
                                                .rounded(px(8.))
                                                .bg(theme.card)
                                                .border_1()
                                                .border_color(theme.border)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(theme.secondary).border_color(theme.ring))
                                                .on_click(cx.listener({
                                                    let note_id = note_id.clone();
                                                    move |this, _, _window, cx| {
                                                        this.sidebar.update(cx, |s, cx| {
                                                            s.select_note(&note_id, cx);
                                                        });
                                                    }
                                                }))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_1p5()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_2()
                                                                        .child(Icon::new(IconName::Star).size(px(13.)).color(theme.primary))
                                                                        .child(
                                                                            div()
                                                                                .text_size(px(14.))
                                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                                .text_color(theme.foreground)
                                                                                .child(note.title.clone()),
                                                                        ),
                                                                )
                                                                .children(note.folder_name.as_ref().map(|f| {
                                                                    div()
                                                                        .px_2()
                                                                        .py(px(1.5))
                                                                        .rounded(px(4.))
                                                                        .bg(theme.muted)
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.muted_foreground)
                                                                        .child(f.clone())
                                                                })),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(12.5))
                                                                .text_color(theme.muted_foreground)
                                                                .line_clamp(2)
                                                                .child(note.snippet.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .pt_1()
                                                                .child(
                                                                    div()
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.muted_foreground)
                                                                        .child(format!("Last edited {}", note.updated_at)),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.primary)
                                                                        .child("Open note →"),
                                                                ),
                                                        ),
                                                )
                                        })),
                                )
                            }),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use crate::theme::{ActiveTheme, Theme};

    #[test]
    fn starred_view_renders_with_sidebar() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_view, cx) = cx.add_window_view(|_, cx| {
            let sidebar = cx.new(|cx| SidebarView::new(cx));
            StarredView::new(sidebar, cx)
        });
        cx.run_until_parked();
    }
}
