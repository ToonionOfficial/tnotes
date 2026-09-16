use crate::components::{Icon, IconName, NavButtons};
use crate::store::{NoteStore, format_relative_time, snippet_from_body};
use crate::theme::ThemeExt;
use gpui::prelude::FluentBuilder;
use gpui::*;
use tnotes_core::models::note::Note;

pub struct StarredView {
    store: Entity<NoteStore>,
    scroll_handle: UniformListScrollHandle,
    starred_indices: Vec<usize>,
    dirty: bool,
    _subscription: Subscription,
}

impl StarredView {
    pub fn new(store: Entity<NoteStore>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&store, |this, _, cx| {
            this.dirty = true;
            cx.notify();
        });
        Self {
            store,
            scroll_handle: UniformListScrollHandle::new(),
            starred_indices: Vec::new(),
            dirty: true,
            _subscription: subscription,
        }
    }
}

impl Render for StarredView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.dirty {
            let store = self.store.read(cx);
            self.starred_indices = store
                .notes()
                .iter()
                .enumerate()
                .filter_map(|(ix, n)| {
                    if n.pinned && !n.trashed {
                        Some(ix)
                    } else {
                        None
                    }
                })
                .collect();
            self.dirty = false;
        }

        let (can_back, can_forward) = {
            let store = self.store.read(cx);
            (store.can_navigate_back(), store.can_navigate_forward())
        };
        let theme = cx.theme().clone();
        let count = self.starred_indices.len();

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
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .px(px(48.))
                    .pt(px(36.))
                    .pb(px(16.))
                    .child(
                        div()
                            .max_w(px(760.))
                            .w_full()
                            .h_full()
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
                            .when(count == 0, |this| {
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
                            .when(count > 0, |this| {
                                this.child(
                                    uniform_list("starred-notes-list", count, {
                                        cx.processor(|this: &mut StarredView, range: std::ops::Range<usize>, _window: &mut Window, cx: &mut Context<StarredView>| {
                                            let theme = cx.theme().clone();
                                            let note_data: Vec<(Note, Option<String>)> = {
                                                let store = this.store.read(cx);
                                                let all_notes = store.notes();
                                                range.clone().filter_map(|i| {
                                                    let &note_idx = this.starred_indices.get(i)?;
                                                    let note = all_notes.get(note_idx)?;
                                                    let folder = store.folder_name(note.folder_id.as_deref()).map(|s| s.to_string());
                                                    Some((note.clone(), folder))
                                                }).collect()
                                            };
                                            note_data.into_iter().map(|(note, folder)| {
                                                div()
                                                    .w_full()
                                                    .h(px(120.))
                                                    .pb(px(10.))
                                                    .child(render_starred_card(cx, &this.store, &note, folder, &theme))
                                                    .into_any_element()
                                            }).collect::<Vec<_>>()
                                        })
                                    })
                                    .track_scroll(self.scroll_handle.clone())
                                    .flex_1()
                                    .min_h_0()
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
        .h_full()
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
                .h_full()
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .min_w_0()
                                .flex_1()
                                .child(Icon::new(IconName::Star).size(px(13.)).color(theme.primary))
                                .child(
                                    div()
                                        .text_size(px(14.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground)
                                        .truncate()
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
                                .flex_shrink_0()
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
    use crate::theme::{ActiveTheme, Theme};
    use core::prelude::v1::test;
    use gpui::AppContext;

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
