use std::collections::HashMap;
use gpui::*;
use tnotes_core::models::note::Note;
use crate::components::{
    FolderTreeItem, Icon, IconName, NoteTreeItem, SidebarGroup, uniform_list,
};
use crate::theme::ThemeExt;
use super::model::TreeRow;
use super::{folder_icon_for, SidebarView};

impl SidebarView {
    pub fn matching_search_notes(&self, cx: &App) -> Vec<Note> {
        let query = self.search_state.value().trim().to_string();
        self.store.read(cx).search_notes(&query)
    }

    pub(super) fn update_tree_rows(&mut self, cx: &mut Context<Self>) {
        let store = self.store.read(cx);
        let folder_tree = store.folder_tree();
        let selected_note = store.selected_note_id();

        let mut direct_counts: HashMap<&str, usize> = HashMap::new();
        let mut expanded_notes: HashMap<&str, Vec<&Note>> = HashMap::new();
        let mut root_notes: Vec<&Note> = Vec::new();

        for note in store.notes() {
            if note.trashed {
                continue;
            }
            if let Some(folder_id) = note.folder_id.as_deref() {
                *direct_counts.entry(folder_id).or_insert(0) += 1;
                if self.expanded_folders.contains(folder_id) {
                    expanded_notes.entry(folder_id).or_default().push(note);
                }
            } else {
                root_notes.push(note);
            }
        }

        let mut subtree_counts: HashMap<&str, usize> = direct_counts;
        for node in folder_tree.iter().rev() {
            if let Some(parent_id) = node.folder.parent_id.as_deref() {
                let count = subtree_counts.get(node.folder.id.as_str()).copied().unwrap_or(0);
                *subtree_counts.entry(parent_id).or_insert(0) += count;
            }
        }

        let mut rows = Vec::new();
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
                    for note in notes {
                        let is_active = selected_note.as_deref() == Some(note.id.as_str());
                        rows.push(TreeRow::Note {
                            id: note.id.clone(),
                            title: note.title.clone(),
                            depth: depth + 1,
                            is_active,
                            is_pinned: note.pinned,
                        });
                    }
                }
            } else {
                skip_below = Some(depth);
            }
        }

        for note in root_notes {
            let is_active = selected_note.as_deref() == Some(note.id.as_str());
            rows.push(TreeRow::Note {
                id: note.id.clone(),
                title: note.title.clone(),
                depth: 0,
                is_active,
                is_pinned: note.pinned,
            });
        }

        self.tree_rows = rows;
    }

    pub(super) fn render_folder_tree(&mut self, cx: &mut Context<Self>) -> SidebarGroup {
        self.update_tree_rows(cx);
        let theme = cx.theme().clone();
        let count = self.tree_rows.len();

        let group = SidebarGroup::new()
            .label("Folders")
            .fill(true)
            .action(
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
            cx.processor(|this: &mut SidebarView, range: std::ops::Range<usize>, _window: &mut Window, cx: &mut Context<SidebarView>| {
                range.map(|ix| this.render_tree_row(ix, cx)).collect::<Vec<_>>()
            })
        })
        .track_scroll(self.tree_scroll_handle.clone())
        .flex_1()
        .min_h_0();

        group.child(list)
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
                    .on_right_click(cx.listener(Self::folder_menu_handler(
                        id,
                        name,
                    )));
                    item.into_any_element()
                }
            }
            TreeRow::Note {
                id,
                title,
                depth,
                is_active,
                is_pinned,
            } => {
                let indent = px(12.0 + (depth as f32 * 16.0));
                if let Some(editor) = self.rename_editor_row(&id, indent, cx) {
                    editor
                } else {
                    let note_id = id.clone();
                    NoteTreeItem::new(format!("tree-note-{}", id), title.clone())
                        .depth(depth)
                        .active(is_active)
                        .pinned(is_pinned)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.select_note(&note_id, cx);
                        }))
                        .on_right_click(cx.listener(Self::note_menu_handler(
                            id,
                            title,
                            is_pinned,
                        )))
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
        self.search_rows = self.matching_search_notes(cx);
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
                cx.processor(move |this: &mut SidebarView, range: std::ops::Range<usize>, _window: &mut Window, cx: &mut Context<SidebarView>| {
                    let selected = selected_note.clone();
                    range.map(|ix| this.render_search_row(ix, selected.as_deref(), cx)).collect::<Vec<_>>()
                })
            })
            .track_scroll(self.search_scroll_handle.clone())
            .flex_1()
            .min_h_0();

            group = group.child(list);
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
}
