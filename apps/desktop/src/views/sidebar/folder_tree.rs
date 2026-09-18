use super::model::TreeRow;
use super::{SidebarView, folder_icon_for};
use crate::components::{FolderTreeItem, Icon, IconName, NoteTreeItem, SidebarGroup, uniform_list};
use crate::theme::ThemeExt;
use gpui::*;
use std::collections::HashMap;
use tnotes_core::models::note::Note;

impl SidebarView {
    pub fn matching_search_notes(&self, cx: &App) -> Vec<Note> {
        let query = self.search_state.value().trim().to_string();
        self.store.read(cx).search_notes(&query)
    }

    pub(super) fn update_tree_rows(&mut self, cx: &mut Context<Self>) {
        let store = self.store.read(cx);
        let folder_tree = store.folder_tree();

        let mut direct_counts: HashMap<&str, usize> = HashMap::new();
        let mut expanded_notes: HashMap<&str, Vec<usize>> = HashMap::new();
        let mut root_notes: Vec<usize> = Vec::new();

        for (idx, note) in store.notes().iter().enumerate() {
            if note.trashed {
                continue;
            }
            if let Some(folder_id) = note.folder_id.as_deref() {
                *direct_counts.entry(folder_id).or_insert(0) += 1;
                if self.expanded_folders.contains(folder_id) {
                    expanded_notes.entry(folder_id).or_default().push(idx);
                }
            } else {
                root_notes.push(idx);
            }
        }

        let mut subtree_counts: HashMap<&str, usize> = direct_counts;
        for node in folder_tree.iter().rev() {
            if let Some(parent_id) = node.folder.parent_id.as_deref() {
                let count = subtree_counts
                    .get(node.folder.id.as_str())
                    .copied()
                    .unwrap_or(0);
                *subtree_counts.entry(parent_id).or_insert(0) += count;
            }
        }

        let mut rows = Vec::with_capacity(folder_tree.len() + root_notes.len());
        let mut skip_below: Option<usize> = None;

        for node in folder_tree {
            let depth = node.depth as usize;
            let folder_id = node.folder.id.as_str();

            if let Some(max_depth) = skip_below {
                if depth > max_depth {
                    continue;
                } else {
                    skip_below = None;
                }
            }

            let expanded = self.is_folder_expanded(folder_id);
            let count = subtree_counts.get(folder_id).copied().unwrap_or(0);

            rows.push(TreeRow::Folder {
                id: node.folder.id.clone(),
                name: node.folder.name.clone(),
                icon: node.folder.icon.clone(),
                depth,
                expanded,
                count,
            });

            if expanded {
                if let Some(notes) = expanded_notes.get(folder_id) {
                    for &note_index in notes {
                        rows.push(TreeRow::Note {
                            note_index,
                            depth: depth + 1,
                        });
                    }
                }
            } else {
                skip_below = Some(depth);
            }
        }

        for note_index in root_notes {
            rows.push(TreeRow::Note {
                note_index,
                depth: 0,
            });
        }

        self.tree_rows = rows;
    }

    pub(super) fn render_folder_tree(&mut self, cx: &mut Context<Self>) -> SidebarGroup {
        let needs_update = self.tree_rows_dirty
            || (self.tree_rows.is_empty() && {
                let store = self.store.read(cx);
                !store.folder_tree().is_empty() || !store.notes().is_empty()
            });
        if needs_update {
            self.update_tree_rows(cx);
            self.tree_rows_dirty = false;
        }
        let theme = cx.theme().clone();
        let count = self.tree_rows.len();

        let group = SidebarGroup::new().label("Folders").fill(true).action(
            div()
                .id("add-folder-btn")
                .w(px(20.))
                .h(px(20.))
                .rounded(px(4.))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                .text_color(theme.muted_foreground)
                .child(Icon::new(IconName::Plus).size(px(12.)))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.create_top_level_folder(cx);
                })),
        );

        let list = uniform_list("sidebar-tree-list", count, {
            cx.processor(
                |this: &mut SidebarView,
                 range: std::ops::Range<usize>,
                 _window: &mut Window,
                 cx: &mut Context<SidebarView>| {
                    range
                        .map(|ix| this.render_tree_row(ix, cx))
                        .collect::<Vec<_>>()
                },
            )
        })
        .track_scroll(self.tree_scroll_handle.clone())
        .size_full();

        let scrollbar = self.render_scrollbar(count, px(28.), &self.tree_scroll_handle, cx);

        group.child(
            div()
                .relative()
                .flex_1()
                .min_h_0()
                .child(list)
                .child(scrollbar),
        )
    }

    fn render_tree_row(&self, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(row) = self.tree_rows.get(ix).cloned() else {
            return div().h(px(28.)).into_any_element();
        };

        match row {
            TreeRow::Folder {
                id,
                name,
                icon,
                depth,
                expanded,
                count,
            } => {
                if let Some(editor) =
                    self.rename_editor_row(&id, px(6.0 + (depth as f32 * 12.0)), cx)
                {
                    editor
                } else {
                    let item = FolderTreeItem::new(
                        SharedString::from(format!("folder-{}", id.replace(':', "-"))),
                        name.clone(),
                    )
                    .depth(depth)
                    .expanded(expanded)
                    .count(count)
                    .icon(folder_icon_for(&icon, expanded))
                    .on_toggle(cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            this.toggle_folder(&id, cx);
                        }
                    }))
                    .on_icon_click(cx.listener({
                        let id = id.clone();
                        move |this, ev: &MouseDownEvent, _window, cx| {
                            this.toggle_icon_picker_at(
                                &id,
                                Some(point(ev.position.x, ev.position.y + px(12.))),
                                cx,
                            );
                        }
                    }))
                    .on_right_click(cx.listener(Self::folder_menu_handler(id, name)));
                    item.into_any_element()
                }
            }
            TreeRow::Note { note_index, depth } => {
                let (note_id, title, is_pinned, is_active) = {
                    let store = self.store.read(cx);
                    let Some(note) = store.notes().get(note_index) else {
                        return div().h(px(28.)).into_any_element();
                    };
                    let is_active = store.selected_note_id().as_deref() == Some(note.id.as_str());
                    (note.id.clone(), note.title.clone(), note.pinned, is_active)
                };
                let indent = px(12.0 + (depth as f32 * 16.0));
                if let Some(editor) = self.rename_editor_row(&note_id, indent, cx) {
                    editor
                } else {
                    let click_id = note_id.clone();
                    NoteTreeItem::new(format!("tree-note-{}", note_id), title.clone())
                        .depth(depth)
                        .active(is_active)
                        .pinned(is_pinned)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.select_note(&click_id, cx);
                        }))
                        .on_right_click(
                            cx.listener(Self::note_menu_handler(note_id, title, is_pinned)),
                        )
                        .into_any_element()
                }
            }
        }
    }

    pub(super) fn render_search_results(
        &mut self,
        selected_note: Option<String>,
        cx: &mut Context<Self>,
    ) -> SidebarGroup {
        let query = self.search_state.value().trim().to_string();
        if query != self.last_search_query || self.tree_rows_dirty {
            self.search_rows = self.matching_search_notes(cx);
            self.last_search_query = query;
        }
        let count = self.search_rows.len();

        let mut group = SidebarGroup::new()
            .label(format!("Search Results ({count})"))
            .fill(true);

        if self.search_rows.is_empty() {
            let theme = cx.theme().clone();
            group = group.child(
                div()
                    .px_2()
                    .py_3()
                    .text_size(px(12.))
                    .text_color(theme.muted_foreground)
                    .child("No matching notes found"),
            );
        } else {
            let list = uniform_list("search-results-list", count, {
                cx.processor(
                    move |this: &mut SidebarView,
                          range: std::ops::Range<usize>,
                          _window: &mut Window,
                          cx: &mut Context<SidebarView>| {
                        let selected = selected_note.clone();
                        range
                            .map(|ix| this.render_search_row(ix, selected.as_deref(), cx))
                            .collect::<Vec<_>>()
                    },
                )
            })
            .track_scroll(self.search_scroll_handle.clone())
            .size_full();

            let scrollbar = self.render_scrollbar(count, px(28.), &self.search_scroll_handle, cx);

            group = group.child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(list)
                    .child(scrollbar),
            );
        }

        group
    }

    fn render_search_row(
        &self,
        ix: usize,
        selected_note: Option<&str>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(note) = self.search_rows.get(ix).cloned() else {
            return div().h(px(28.)).into_any_element();
        };
        let is_active = selected_note == Some(note.id.as_str());
        let indent = px(12.0);
        if let Some(editor) = self.rename_editor_row(&note.id, indent, cx) {
            editor
        } else {
            let note_id = note.id.clone();
            NoteTreeItem::new(format!("search-note-{}", note.id), note.title.clone())
                .depth(0)
                .active(is_active)
                .pinned(note.pinned)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.select_note(&note_id, cx);
                }))
                .on_right_click(cx.listener(Self::note_menu_handler(
                    note.id,
                    note.title,
                    note.pinned,
                )))
                .into_any_element()
        }
    }

    fn render_scrollbar(
        &self,
        count: usize,
        item_height: Pixels,
        scroll_handle: &UniformListScrollHandle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (table_bounds, scroll_top, scroll_height) = {
            let state = scroll_handle.0.borrow();
            let bounds = state.base_handle.bounds();
            let offset = state.base_handle.offset().y;
            let total_height = (item_height * count as f32).max(bounds.size.height);
            (bounds, offset, total_height)
        };

        if table_bounds.size.height <= px(0.)
            || count == 0
            || scroll_height <= table_bounds.size.height
        {
            return div().into_any_element();
        }

        let table_height = table_bounds.size.height;
        let max_scroll = scroll_height - table_height;
        let ratio = (table_height / scroll_height).clamp(0.08, 1.0);
        let thumb_height = (table_height * ratio).clamp(px(24.), table_height - px(8.));
        let track_height = table_height - thumb_height;
        let percentage = (-scroll_top / max_scroll).clamp(0.0, 1.0);
        let offset_top = track_height * percentage;

        let scroll_handle = scroll_handle.clone();
        let entity = cx.entity();
        let theme = cx.theme().clone();

        div()
            .id("sidebar-scrollbar-track")
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .w(px(8.))
            .flex()
            .justify_center()
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, {
                let scroll_handle = scroll_handle.clone();
                let entity = entity.clone();
                move |ev, _window, cx| {
                    let click_y = ev.position.y - table_bounds.origin.y;
                    let target_pct =
                        ((click_y - thumb_height / 2.0) / track_height).clamp(0.0, 1.0);
                    let target_offset = max_scroll * target_pct;
                    scroll_handle
                        .0
                        .borrow_mut()
                        .base_handle
                        .set_offset(point(px(0.), -target_offset));
                    cx.notify(entity.entity_id());
                }
            })
            .child(
                div()
                    .id("sidebar-scrollbar-thumb")
                    .absolute()
                    .top(offset_top)
                    .w(px(4.))
                    .h(thumb_height)
                    .rounded(px(2.))
                    .bg(theme.muted_foreground.opacity(0.35))
                    .hover(|s| s.w(px(6.)).bg(theme.muted_foreground.opacity(0.7)))
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |thumb_bounds, _, window, _| {
                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |ev: &MouseDownEvent, _, _, cx| {
                                        if !thumb_bounds.contains(&ev.position) {
                                            return;
                                        }
                                        entity.update(cx, |this, _| {
                                            this.scrollbar_drag_offset =
                                                Some(ev.position.y - thumb_bounds.origin.y);
                                        });
                                    }
                                });
                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |_: &MouseUpEvent, _, _, cx| {
                                        entity.update(cx, |this, _| {
                                            this.scrollbar_drag_offset = None;
                                        });
                                    }
                                });
                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    let scroll_handle = scroll_handle.clone();
                                    move |ev: &MouseMoveEvent, _, _, cx| {
                                        if !ev.dragging() {
                                            return;
                                        }
                                        let Some(drag_offset) =
                                            entity.read(cx).scrollbar_drag_offset
                                        else {
                                            return;
                                        };
                                        let thumb_top =
                                            ev.position.y - table_bounds.origin.y - drag_offset;
                                        let percentage = (thumb_top / track_height).clamp(0.0, 1.0);
                                        let offset_y = max_scroll * percentage;
                                        scroll_handle
                                            .0
                                            .borrow_mut()
                                            .base_handle
                                            .set_offset(point(px(0.), -offset_y));
                                        cx.notify(entity.entity_id());
                                    }
                                });
                            },
                        )
                        .size_full(),
                    ),
            )
            .into_any_element()
    }
}
