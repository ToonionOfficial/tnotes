use gpui::prelude::FluentBuilder;
use gpui::*;
use tnotes_core::models::note::Note;
use crate::components::{Icon, IconName, NavButtons};
use crate::store::{NoteStore, format_relative_time, snippet_from_body};
use crate::theme::ThemeExt;

pub struct StarredView {
    store: Entity<NoteStore>,
    _subscription: Subscription,
}

impl StarredView {
    pub fn new(store: Entity<NoteStore>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&store, |_, _, cx| {
            cx.notify();
        });
        Self {
            store,
            _subscription: subscription,
        }
    }
}

impl Render for StarredView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (starred_notes, can_back, can_forward, folder_names) = {
            let store = self.store.read(cx);
            let notes = store.starred_notes();
            let names: Vec<Option<String>> = notes
                .iter()
                .map(|n| {
                    store
                        .folder_name(n.folder_id.as_deref())
                        .map(|s| s.to_string())
                })
                .collect();
            (
                notes,
                store.can_navigate_back(),
                store.can_navigate_forward(),
            names,
            )
        };
        let theme = cx.theme().clone();
        let count = starred_notes.len();

        let nav_buttons = NavButtons::new(can_back, can_forward)
            .on_back(cx.listener(|this, _, _window, cx| {
                this.store.update(cx, |s, cx| {
                    s.navigate_back(cx);
                });
            }))
            .on_forward(cx.listener(|this, _, _window, cx| {
                this.store.update(cx, |s, cx| {
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
                                        .children(starred_notes.iter().zip(folder_names.iter()).map(|(note, folder)| {
                                            render_starred_card(cx, &self.store, note, folder.clone(), &theme)
                                        })),
                                )
                            }),
                    ),
            )
    }
}

fn render_starred_card(
    cx: &mut Context<StarredView>,
    store: &Entity<NoteStore>,
    note: &Note,
    folder: Option<String>,
    theme: &crate::theme::Theme,
) -> gpui::AnyElement {
    let note_id = note.id.clone();
    let snippet = snippet_from_body(&note.searchable_text, 160);
    let updated = format_relative_time(note.updated_at);
    let store = store.clone();
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
        .on_click(cx.listener(move |this, _, _window, cx| {
            let _ = this;
            let note_id = note_id.clone();
            store.update(cx, |s, cx| {
                s.select_note(&note_id, cx);
            });
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
                        .children(folder.as_ref().map(|f| {
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
                        .child(snippet),
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
                                .child(format!("Last edited {updated}")),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(theme.primary)
                                .child("Open note →"),
                        ),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::AppContext;
    use crate::theme::{ActiveTheme, Theme};

    #[test]
    fn starred_view_renders_with_sidebar() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            StarredView::new(store, cx)
        });
        cx.run_until_parked();
    }
}
