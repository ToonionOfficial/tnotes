use std::collections::HashMap;
use gpui::*;
use tnotes_core::models::note::Note;
use crate::components::{
    FolderTreeItem, Icon, IconName, NoteTreeItem, SidebarGroup,
};
use crate::theme::ThemeExt;
use super::{SidebarView, folder_icon_for, FOLDER_ICON_OPTIONS};

impl SidebarView {
    pub fn matching_search_notes(&self, cx: &App) -> Vec<Note> {
        let query = self.search_state.value().trim().to_string();
        self.store.read(cx).search_notes(&query)
    }

    pub(super) fn render_icon_strip(
        &self,
        folder_id: &str,
        current_icon: &str,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let indent = px(6.0 + (depth as f32 * 12.0));
        let theme = cx.theme().clone();
        div()
            .id(SharedString::from(format!(
                "folder-icons-{}",
                folder_id.replace(':', "-")
            )))
            .pl(indent)
            .pr_2()
            .py(px(2.))
            .flex()
            .flex_wrap()
            .gap_1()
            .children(FOLDER_ICON_OPTIONS.iter().map(|(name, icon)| {
                let selected = current_icon.eq_ignore_ascii_case(name);
                let icon = *icon;
                let folder_id = folder_id.to_string();
                div()
                    .id(SharedString::from(format!(
                        "folder-icon-{}-{name}",
                        folder_id.replace(':', "-")
                    )))
                    .w(px(28.))
                    .h(px(28.))
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .bg(if selected {
                        theme.secondary
                    } else {
                        gpui::transparent_black()
                    })
                    .border_1()
                    .border_color(if selected {
                        theme.ring
                    } else {
                        theme.border
                    })
                    .hover(|s| s.bg(theme.secondary))
                    .text_color(if selected {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    })
                    .child(Icon::new(icon).size(px(14.)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.pick_folder_icon(&folder_id, icon, cx);
                    }))
            }))
            .into_any_element()
    }

    fn render_note_row(
        &self,
        element_id: String,
        note: &Note,
        depth: usize,
        indent: Pixels,
        is_active: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(editor) = self.rename_editor_row(&note.id, indent, cx) {
            return editor;
        }
        let note_id = note.id.clone();
        NoteTreeItem::new(element_id, note.title.clone())
            .depth(depth)
            .active(is_active)
            .pinned(note.pinned)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_note(&note_id, cx);
            }))
            .on_right_click(cx.listener(Self::note_menu_handler(
                note.id.clone(),
                note.title.clone(),
                note.pinned,
            )))
            .into_any_element()
    }

    pub(super) fn render_folder_tree(&self, cx: &mut Context<Self>) -> SidebarGroup {
        // Clone out of the store first so no read guard is held while
        // building listeners below.
        let (folders, selected_note, counts) = {
            let store = self.store.read(cx);
            let folders: Vec<(String, String, String, usize)> = store
                .folder_tree()
                .iter()
                .map(|n| {
                    (
                        n.folder.id.clone(),
                        n.folder.name.clone(),
                        n.folder.icon.clone(),
                        n.depth as usize,
                    )
                })
                .collect();
            let selected = store.selected_note_id();
            let counts: HashMap<String, usize> = folders
                .iter()
                .map(|(id, _, _, _)| (id.clone(), store.note_count_in_folder(id)))
                .collect();
            let folder_notes: Vec<(String, Vec<Note>)> = folders
                .iter()
                .map(|(id, _, _, _)| (id.clone(), store.notes_in_folder(id)))
                .collect();
            let root_notes = store.notes_without_folder();
            (folders, selected, (counts, folder_notes, root_notes))
        };
        let (counts, folder_notes, root_notes) = counts;
        let notes_by_folder: HashMap<String, Vec<Note>> =
            folder_notes.into_iter().collect();
        let theme = cx.theme().clone();

        let mut group = SidebarGroup::new()
            .label("Folders")
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

        let mut skip_below: Option<usize> = None;
        for (folder_id, folder_name, folder_icon, depth) in &folders {
            if let Some(max_depth) = skip_below {
                if *depth > max_depth {
                    continue;
                } else {
                    skip_below = None;
                }
            }

            let expanded = self.is_folder_expanded(folder_id);
            let count = counts.get(folder_id).copied().unwrap_or(0);
            if let Some(editor) =
                self.rename_editor_row(folder_id, px(6.0 + (*depth as f32 * 12.0)), cx)
            {
                group = group.child(editor);
            } else {
                let item = FolderTreeItem::new(
                    SharedString::from(format!("folder-{}", folder_id.replace(':', "-"))),
                    folder_name.clone(),
                )
                .depth(*depth)
                .expanded(expanded)
                .count(count)
                .icon(folder_icon_for(folder_icon, expanded))
                .on_toggle(cx.listener({
                    let folder_id = folder_id.clone();
                    move |this, _, _, cx| {
                        this.toggle_folder(&folder_id, cx);
                    }
                }))
                .on_right_click(cx.listener(Self::folder_menu_handler(
                    folder_id.clone(),
                    folder_name.clone(),
                )));
                group = group.child(item);
            }

            if self.picking_icon_for.as_deref() == Some(folder_id.as_str()) {
                group =
                    group.child(self.render_icon_strip(folder_id, folder_icon, depth + 1, cx));
            }

            if expanded {
                if let Some(notes) = notes_by_folder.get(folder_id) {
                    for note in notes {
                        let is_active =
                            selected_note.as_deref() == Some(note.id.as_str());
                        group = group.child(self.render_note_row(
                            format!("tree-note-{}", note.id),
                            note,
                            depth + 1,
                            px(12.0 + ((depth + 1) as f32 * 16.0)),
                            is_active,
                            cx,
                        ));
                    }
                }
            } else {
                skip_below = Some(*depth);
            }
        }

        for note in &root_notes {
            let is_active = selected_note.as_deref() == Some(note.id.as_str());
            group = group.child(self.render_note_row(
                format!("root-note-{}", note.id),
                note,
                0,
                px(12.0),
                is_active,
                cx,
            ));
        }

        group
    }

    pub(super) fn render_search_results(
        &self,
        selected_note: Option<String>,
        cx: &mut Context<Self>,
    ) -> SidebarGroup {
        let matches = self.matching_search_notes(cx);
        let count = matches.len();

        let mut group = SidebarGroup::new().label(format!("Search Results ({count})"));

        if matches.is_empty() {
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
            for note in matches {
                let is_active = selected_note.as_deref() == Some(note.id.as_str());
                group = group.child(self.render_note_row(
                    format!("search-note-{}", note.id),
                    &note,
                    0,
                    px(12.0),
                    is_active,
                    cx,
                ));
            }
        }

        group
    }
}
