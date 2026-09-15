use gpui::prelude::FluentBuilder;
use gpui::*;
use tnotes_core::models::note::Note;
use crate::components::{Icon, IconName, NavButtons};
use crate::store::{NoteStore, format_relative_time, snippet_from_body};
use crate::theme::ThemeExt;

pub struct TrashView {
    store: Entity<NoteStore>,
    _subscription: Subscription,
}

impl TrashView {
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

impl Render for TrashView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (trash_notes, can_back, can_forward, folder_names) = {
            let store = self.store.read(cx);
            let notes = store.trash_notes();
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
        let count = trash_notes.len();

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
                                            .child(Icon::new(IconName::Trash2).size(px(13.)).color(theme.muted_foreground))
                                            .child("Trash"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when(!trash_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .id("empty-trash-header-btn")
                                        .px_2p5()
                                        .py_1()
                                        .rounded(px(5.))
                                        .bg(gpui::transparent_black())
                                        .border_1()
                                        .border_color(theme.border)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(theme.destructive).border_color(theme.destructive).text_color(theme.destructive_foreground))
                                        .text_size(px(11.5))
                                        .text_color(theme.muted_foreground)
                                        .child("Empty Trash")
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.store.update(cx, |s, cx| {
                                                s.empty_trash(cx);
                                            });
                                        })),
                                )
                            })
                            .child(
                                div()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded(px(10.))
                                    .bg(theme.secondary)
                                    .text_size(px(11.))
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{count} item{}", if count == 1 { "" } else { "s" })),
                            ),
                    ),
            )
            .child(
                div()
                    .id("trash-pane-scroll")
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
                                            .child(Icon::new(IconName::Trash2).size(px(18.)).color(theme.muted_foreground)),
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
                                                    .child("Trash"),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(13.))
                                                    .text_color(theme.muted_foreground)
                                                    .child("Deleted notes remain here until permanently removed"),
                                            ),
                                    ),
                            )
                            .child(div().w_full().h(px(1.)).bg(theme.border))
                            .when(trash_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py(px(64.))
                                        .gap_3()
                                        .child(
                                            Icon::new(IconName::Trash2)
                                                .size(px(40.))
                                                .color(theme.muted_foreground),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme.foreground)
                                                .child("Trash is empty"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(13.))
                                                .text_color(theme.muted_foreground)
                                                .child("Notes you delete will appear here."),
                                        ),
                                )
                            })
                            .when(!trash_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2p5()
                                        .children(trash_notes.iter().zip(folder_names.iter()).map(|(note, folder)| {
                                            render_trash_card(cx, &self.store, note, folder.clone(), &theme)
                                        })),
                                )
                            }),
                    ),
            )
    }
}

fn render_trash_card(
    cx: &mut Context<TrashView>,
    store: &Entity<NoteStore>,
    note: &Note,
    folder: Option<String>,
    theme: &crate::theme::Theme,
) -> gpui::AnyElement {
    let snippet = snippet_from_body(&note.searchable_text, 160);
    let updated = format_relative_time(note.updated_at);
    let restore_store = store.clone();
    let delete_store = store.clone();
    let restore_id = note.id.clone();
    let delete_id = note.id.clone();
    div()
        .id(SharedString::from(format!("trash-card-{}", note.id)))
        .w_full()
        .p(px(14.))
        .rounded(px(8.))
        .bg(theme.card)
        .border_1()
        .border_color(theme.border)
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
                                .child(Icon::new(IconName::FileText).size(px(13.)).color(theme.muted_foreground))
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
                                .child(format!("Deleted • {updated}")),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .id(SharedString::from(format!("restore-btn-{}", note.id)))
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(theme.foreground))
                                        .text_size(px(12.))
                                        .text_color(theme.primary)
                                        .child("Restore")
                                        .on_click(cx.listener(move |this, _, _window, cx| {
                                            let _ = this;
                                            let note_id = restore_id.clone();
                                            let restore_store = restore_store.clone();
                                            restore_store.update(cx, |s, cx| {
                                                s.restore_note(&note_id, cx);
                                            });
                                        })),
                                )
                                .child(
                                    div()
                                        .id(SharedString::from(format!("delete-perm-btn-{}", note.id)))
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(theme.destructive))
                                        .text_size(px(12.))
                                        .text_color(theme.muted_foreground)
                                        .child("Delete Permanently")
                                        .on_click(cx.listener({
                                            let delete_store = delete_store.clone();
                                            let delete_id = delete_id.clone();
                                            move |this, _, _window, cx| {
                                                let _ = this;
                                                let note_id = delete_id.clone();
                                                let delete_store = delete_store.clone();
                                                delete_store.update(cx, |s, cx| {
                                                    s.permanently_delete_note(&note_id, cx);
                                                });
                                            }
                                        })),
                                ),
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
    fn trash_view_renders_with_sidebar() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            TrashView::new(store, cx)
        });
        cx.run_until_parked();
    }
}
