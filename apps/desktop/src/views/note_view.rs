use super::note_editor::build_note_editor;
use crate::components::{Icon, IconName, NavButtons};
use crate::store::{NoteStore, format_relative_time};
use crate::theme::ThemeExt;
use gpui::*;
use twrite::Editor;

pub struct NoteView {
    store: Entity<NoteStore>,
    editor: Entity<Editor>,
    /// The note id currently loaded into the editor, if any.
    loaded_note_id: Option<String>,
    /// Set after the editor has been auto-focused once, so later renders
    /// don't steal focus back from the sidebar, dialogs, etc.
    did_initial_focus: bool,
    _subscription: Subscription,
}

impl NoteView {
    pub fn new(store: Entity<NoteStore>, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(build_note_editor);

        let subscription = cx.observe(&store, |this: &mut Self, store, cx| {
            this.sync_editor(store, cx);
            cx.notify();
        });

        let mut view = Self {
            store,
            editor,
            loaded_note_id: None,
            did_initial_focus: false,
            _subscription: subscription,
        };
        view.sync_editor(view.store.clone(), cx);
        view
    }

    /// Give keyboard focus to the editor.
    pub fn focus_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.editor.read(cx).focus_handle.clone();
        window.focus(&handle);
    }

    /// Load the selected note's body into the twrite editor when the
    /// selection changes (or clear it when nothing is selected).
    fn sync_editor(&mut self, store: Entity<NoteStore>, cx: &mut Context<Self>) {
        let selected_id = store.read(cx).selected_note_id();

        // Already showing this note — nothing to do.
        if selected_id == self.loaded_note_id {
            return;
        }

        self.loaded_note_id = selected_id.clone();

        if let Some(note) = store.read(cx).selected_note() {
            self.editor.update(cx, |ed, _| {
                ed.buffer.set_cursor_offset(0);
                ed.selection = None;
                ed.scroll_row = 0;
                ed.buffer = twrite::EditorBuffer::new(&note.body);
                ed.layout_cache.clear();
            });
        } else {
            self.editor.update(cx, |ed, _| {
                ed.buffer = twrite::EditorBuffer::new("");
                ed.selection = None;
                ed.scroll_row = 0;
                ed.layout_cache.clear();
            });
        }
    }
}

impl Render for NoteView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (note, folder_label, can_back, can_forward) = {
            let store = self.store.read(cx);
            let note = store.selected_note();
            let folder_label = note
                .as_ref()
                .and_then(|n| store.folder_name(n.folder_id.as_deref()))
                .unwrap_or("Notes")
                .to_string();
            (
                note,
                folder_label,
                store.can_navigate_back(),
                store.can_navigate_forward(),
            )
        };
        let theme = cx.theme().clone();

        // Auto-focus the editor on first mount so the user can type right
        // away. Guarded by a flag (and a selected note) so later renders
        // never yank focus away from the sidebar, dialogs, etc.
        if !self.did_initial_focus && note.is_some() {
            self.did_initial_focus = true;
            self.focus_editor(window, cx);
        }

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

        if let Some(note) = note {
            let updated = format_relative_time(note.updated_at);

            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .bg(theme.background)
                .child(
                    // ── Top toolbar ──────────────────────────────────
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
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(theme.muted_foreground)
                                        .child(format!("Last edited {updated}")),
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
                                        .hover(|s| {
                                            s.bg(theme.secondary).text_color(theme.foreground)
                                        })
                                        .text_color(if note.pinned {
                                            theme.primary
                                        } else {
                                            theme.muted_foreground
                                        })
                                        .child(Icon::new(IconName::Star).size(px(14.)))
                                        .on_click(cx.listener({
                                            let note_id = note.id.clone();
                                            move |this, _, _window, cx| {
                                                this.store.update(cx, |s, cx| {
                                                    s.toggle_note_pin(&note_id, cx);
                                                });
                                            }
                                        })),
                                ),
                        ),
                )
                .child(
                    // ── Editor pane ──────────────────────────────────
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .px(px(48.))
                        .py(px(24.))
                        .child(
                            div()
                                .max_w(px(760.))
                                .w_full()
                                .h_full()
                                .mx_auto()
                                .child(self.editor.clone()),
                        ),
                )
                .into_any_element()
        } else {
            // ── Empty state ──────────────────────────────────────
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
                                .child(
                                    "Select a note from the sidebar or click 'New Note' to create one",
                                ),
                        ),
                )
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ActiveTheme, Theme};
    use core::prelude::v1::test;
    use gpui::AppContext;

    #[test]
    fn note_view_renders_with_sidebar() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            NoteView::new(store, cx)
        });
        cx.run_until_parked();
    }
}
