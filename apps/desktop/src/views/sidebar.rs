use std::collections::HashSet;
use gpui::*;
use tnotes_core::models::note::Note;
use crate::components::{
    Button, ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel,
    ContextMenuSeparator, FolderTreeItem, Icon, IconName, Input, InputState, NoteTreeItem, Sidebar,
    SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarHeader, SidebarRail,
    SidebarRailItem, SidebarToggleButton,
};
use crate::store::{NoteStore, NavigationLocation};
use crate::theme::ThemeExt;

pub struct SidebarView {
    store: Entity<NoteStore>,
    is_collapsed: bool,
    search_state: InputState,
    search_focus: FocusHandle,
    expanded_folders: HashSet<String>,
    context_menu: Option<SidebarContextMenu>,
    context_menu_focus: FocusHandle,
    renaming: Option<RenameState>,
    picking_icon_for: Option<String>,
    _store_subscription: Subscription,
}

/// Inline rename session for one folder or note row.
#[derive(Clone, Debug)]
pub struct RenameState {
    kind: RenameKind,
    id: String,
    input: InputState,
    focus: FocusHandle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameKind {
    Folder,
    Note,
}

/// Folder icon options (mobile parity) with their stored lowercase names.
pub const FOLDER_ICON_OPTIONS: &[(&str, IconName)] = &[
    ("folder", IconName::Folder),
    ("briefcase", IconName::Briefcase),
    ("lightbulb", IconName::Lightbulb),
    ("file-text", IconName::FileText),
    ("rocket", IconName::Rocket),
    ("target", IconName::Target),
    ("graduation-cap", IconName::GraduationCap),
    ("palette", IconName::Palette),
    ("home", IconName::Home),
    ("wallet", IconName::Wallet),
    ("star", IconName::Star),
    ("heart", IconName::Heart),
    ("bookmark", IconName::Bookmark),
    ("code", IconName::Code),
    ("music", IconName::Music),
    ("zap", IconName::Zap),
    ("shopping-cart", IconName::ShoppingCart),
];

pub fn folder_icon_name(icon: IconName) -> &'static str {
    FOLDER_ICON_OPTIONS
        .iter()
        .find(|(_, candidate)| *candidate == icon)
        .map(|(name, _)| *name)
        .unwrap_or("folder")
}

pub fn folder_icon_from_name(name: &str) -> Option<IconName> {
    FOLDER_ICON_OPTIONS
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name.trim()))
        .map(|(_, icon)| *icon)
}

/// Resolve a stored `Folder.icon` value to a renderable icon, falling back to
/// the open/closed folder glyph for legacy (emoji) or unknown values.
pub fn folder_icon_for(stored: &str, expanded: bool) -> IconName {
    folder_icon_from_name(stored).unwrap_or(if expanded {
        IconName::FolderOpen
    } else {
        IconName::Folder
    })
}

/// Right-click target for the sidebar context menu.
#[derive(Clone, Debug)]
pub enum SidebarContextTarget {
    Folder { id: String, name: String },
    Note { id: String, title: String, is_pinned: bool },
}

#[derive(Clone, Debug)]
pub struct SidebarContextMenu {
    position: Point<Pixels>,
    target: SidebarContextTarget,
}

impl SidebarView {
    pub fn new(store: Entity<NoteStore>, cx: &mut Context<Self>) -> Self {
        let store_sub = cx.observe(&store, |_, _, cx| cx.notify());
        // Folders start collapsed; expansion is purely local UI state.
        let expanded_folders = HashSet::new();

        Self {
            store,
            is_collapsed: false,
            search_state: InputState::new(""),
            search_focus: cx.focus_handle(),
            expanded_folders,
            context_menu: None,
            context_menu_focus: cx.focus_handle(),
            renaming: None,
            picking_icon_for: None,
            _store_subscription: store_sub,
        }
    }

    #[allow(dead_code)]
    pub fn search_query(&self) -> &str {
        self.search_state.value()
    }

    fn handle_search_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.search_state.handle_key(event, window, cx) {
            cx.notify();
        }
    }

    pub fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_collapsed {
            self.is_collapsed = false;
        }
        window.focus(&self.search_focus);
        cx.notify();
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.is_collapsed = !self.is_collapsed;
        self.context_menu = None;
        cx.notify();
    }

    pub fn toggle_folder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        if self.expanded_folders.contains(folder_id) {
            self.expanded_folders.remove(folder_id);
        } else {
            self.expanded_folders.insert(folder_id.to_string());
        }
        cx.notify();
    }

    pub fn is_folder_expanded(&self, folder_id: &str) -> bool {
        self.expanded_folders.contains(folder_id)
    }

    #[cfg(test)]
    pub fn test_store(&self) -> Entity<NoteStore> {
        self.store.clone()
    }

    // ---- thin delegates over NoteStore ----

    pub fn selected_note_id(&self, cx: &App) -> Option<String> {
        self.store.read(cx).selected_note_id()
    }

    pub fn open_starred(&mut self, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.open_starred(cx));
    }

    #[allow(dead_code)]
    pub fn toggle_starred(&mut self, cx: &mut Context<Self>) {
        self.open_starred(cx);
    }

    pub fn open_trash(&mut self, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.open_trash(cx));
    }

    #[allow(dead_code)]
    pub fn select_section(&mut self, section_id: &str, cx: &mut Context<Self>) {
        match section_id {
            "starred" => self.open_starred(cx),
            "trash" => self.open_trash(cx),
            _ => cx.notify(),
        }
    }

    pub fn select_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.select_note(note_id, cx));
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) -> bool {
        self.store.update(cx, |store, cx| store.navigate_back(cx))
    }

    pub fn navigate_forward(&mut self, cx: &mut Context<Self>) -> bool {
        self.store.update(cx, |store, cx| store.navigate_forward(cx))
    }

    pub fn create_new_note(&mut self, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.create_new_note(cx));
        self.context_menu = None;
    }

    pub fn create_note_in_folder(&mut self, folder_id: Option<String>, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.create_note_in_folder(folder_id, cx));
        self.context_menu = None;
    }

    pub fn toggle_note_pin(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.toggle_note_pin(note_id, cx));
        self.context_menu = None;
    }

    pub fn duplicate_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.duplicate_note(note_id, cx));
        self.context_menu = None;
    }

    pub fn delete_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.delete_note(note_id, cx));
        self.context_menu = None;
    }

    pub fn restore_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.restore_note(note_id, cx));
    }

    pub fn empty_trash(&mut self, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.empty_trash(cx));
    }

    pub fn begin_rename_note(&mut self, note_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let title = self
            .store
            .read(cx)
            .active_notes()
            .into_iter()
            .find(|n| n.id == note_id)
            .map(|n| n.title);
        if let Some(title) = title {
            self.renaming = Some(RenameState {
                kind: RenameKind::Note,
                id: note_id.to_string(),
                input: InputState::new(title),
                focus: cx.focus_handle(),
            });
            self.context_menu = None;
            self.picking_icon_for = None;
            if let Some(state) = self.renaming.as_ref() {
                window.focus(&state.focus);
            }
            cx.notify();
        }
    }

    pub fn begin_rename_folder(
        &mut self,
        folder_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = self
            .store
            .read(cx)
            .folder_tree()
            .iter()
            .find(|n| n.folder.id == folder_id)
            .map(|n| n.folder.name.clone());
        if let Some(name) = name {
            self.renaming = Some(RenameState {
                kind: RenameKind::Folder,
                id: folder_id.to_string(),
                input: InputState::new(name),
                focus: cx.focus_handle(),
            });
            self.context_menu = None;
            self.picking_icon_for = None;
            if let Some(state) = self.renaming.as_ref() {
                window.focus(&state.focus);
            }
            cx.notify();
        }
    }

    pub fn commit_rename(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = self.renaming.take() {
            let value = state.input.value().to_string();
            match state.kind {
                RenameKind::Folder => self
                    .store
                    .update(cx, |store, cx| store.rename_folder(&state.id, &value, cx)),
                RenameKind::Note => self
                    .store
                    .update(cx, |store, cx| store.rename_note(&state.id, &value, cx)),
            }
            cx.notify();
        }
    }

    pub fn cancel_rename(&mut self, cx: &mut Context<Self>) {
        if self.renaming.take().is_some() {
            cx.notify();
        }
    }

    fn handle_rename_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "enter" {
            self.commit_rename(cx);
        } else if key == "escape" {
            self.cancel_rename(cx);
        } else if let Some(state) = self.renaming.as_mut()
            && state.input.handle_key(event, window, cx)
        {
            cx.notify();
        }
    }

    pub fn toggle_icon_picker(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        if self.picking_icon_for.as_deref() == Some(folder_id) {
            self.picking_icon_for = None;
        } else {
            self.picking_icon_for = Some(folder_id.to_string());
            self.renaming = None;
        }
        self.context_menu = None;
        cx.notify();
    }

    pub fn pick_folder_icon(&mut self, folder_id: &str, icon: IconName, cx: &mut Context<Self>) {
        let folder_id = folder_id.to_string();
        let name = folder_icon_name(icon);
        self.store.update(cx, |store, cx| {
            store.set_folder_icon(&folder_id, name, cx)
        });
        self.picking_icon_for = None;
        cx.notify();
    }

    pub fn create_top_level_folder(&mut self, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.create_top_level_folder(cx));
        self.context_menu = None;
    }

    pub fn create_subfolder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        let folder_id = folder_id.to_string();
        self.store.update(cx, |store, cx| {
            store.create_folder("Untitled Folder", Some(folder_id), cx);
        });
        self.context_menu = None;
        cx.notify();
    }

    pub fn delete_folder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        let subtree = self
            .store
            .read(cx)
            .folder_subtree_ids(folder_id);
        self.store
            .update(cx, |store, cx| store.delete_folder(folder_id, cx));
        for fid in subtree {
            self.expanded_folders.remove(&fid);
        }
        self.context_menu = None;
    }

    pub fn open_context_menu(
        &mut self,
        position: Point<Pixels>,
        target: SidebarContextTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // A new menu takes focus; any inline rename or icon picking ends.
        self.renaming = None;
        self.picking_icon_for = None;
        let viewport = window.viewport_size();
        let max_x = (viewport.width - px(300.)).max(px(0.));
        let max_y = (viewport.height - px(260.)).max(px(0.));
        let clamped = point(position.x.min(max_x), position.y.min(max_y));

        self.context_menu = Some(SidebarContextMenu {
            position: clamped,
            target,
        });
        window.focus(&self.context_menu_focus);
        cx.notify();
    }

    pub fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        self.context_menu = None;
        cx.notify();
    }

    fn folder_menu_handler(
        folder_id: String,
        folder_name: String,
    ) -> impl Fn(&mut Self, &MouseDownEvent, &mut Window, &mut Context<Self>) + 'static {
        move |this, event, window, cx| {
            this.open_context_menu(
                event.position,
                SidebarContextTarget::Folder {
                    id: folder_id.clone(),
                    name: folder_name.clone(),
                },
                window,
                cx,
            );
        }
    }

    fn note_menu_handler(
        note_id: String,
        title: String,
        is_pinned: bool,
    ) -> impl Fn(&mut Self, &MouseDownEvent, &mut Window, &mut Context<Self>) + 'static {
        move |this, event, window, cx| {
            this.open_context_menu(
                event.position,
                SidebarContextTarget::Note {
                    id: note_id.clone(),
                    title: title.clone(),
                    is_pinned,
                },
                window,
                cx,
            );
        }
    }

    fn context_menu_element(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let state = self.context_menu.clone()?;

        let content = match &state.target {
            SidebarContextTarget::Folder { id, name } => {
                let entity = cx.entity();
                let new_note_id = id.clone();
                let new_note_entity = entity.clone();
                let subfolder_id = id.clone();
                let subfolder_entity = entity.clone();
                let icon_id = id.clone();
                let icon_entity = entity.clone();
                let rename_id = id.clone();
                let rename_entity = entity.clone();
                let delete_id = id.clone();

                ContextMenuContent::new()
                    .child(ContextMenuLabel::new(name.clone()))
                    .child(
                        ContextMenuItem::new("ctx-folder-new-note", "New Note")
                            .icon(IconName::FilePlus)
                            .on_select(move |_, _, cx| {
                                new_note_entity.update(cx, |this, cx| {
                                    this.create_note_in_folder(Some(new_note_id.clone()), cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-folder-new-subfolder", "New Subfolder")
                            .icon(IconName::FolderPlus)
                            .on_select(move |_, _, cx| {
                                subfolder_entity.update(cx, |this, cx| {
                                    this.create_subfolder(&subfolder_id, cx);
                                });
                            }),
                    )
                    .child(ContextMenuSeparator)
                    .child(
                        ContextMenuItem::new("ctx-folder-icon", "Change Icon")
                            .icon(IconName::Palette)
                            .on_select(move |_, _, cx| {
                                icon_entity.update(cx, |this, cx| {
                                    this.toggle_icon_picker(&icon_id, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-folder-rename", "Rename")
                            .icon(IconName::Pencil)
                            .on_select(move |_, window, cx| {
                                rename_entity.update(cx, |this, cx| {
                                    this.begin_rename_folder(&rename_id, window, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-folder-delete", "Delete")
                            .icon(IconName::Trash2)
                            .destructive(true)
                            .on_select(move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.delete_folder(&delete_id, cx);
                                });
                            }),
                    )
            }
            SidebarContextTarget::Note {
                id,
                title,
                is_pinned,
            } => {
                let entity = cx.entity();
                let pin_id = id.clone();
                let pin_entity = entity.clone();
                let rename_id = id.clone();
                let rename_entity = entity.clone();
                let duplicate_id = id.clone();
                let duplicate_entity = entity.clone();
                let delete_id = id.clone();
                let pin_label: SharedString = if *is_pinned {
                    "Unpin from Starred"
                } else {
                    "Pin to Starred"
                }
                .into();

                ContextMenuContent::new()
                    .child(ContextMenuLabel::new(title.clone()))
                    .child(
                        ContextMenuItem::new("ctx-note-pin", pin_label)
                            .icon(IconName::Star)
                            .on_select(move |_, _, cx| {
                                pin_entity.update(cx, |this, cx| {
                                    this.toggle_note_pin(&pin_id, cx);
                                });
                            }),
                    )
                    .child(ContextMenuSeparator)
                    .child(
                        ContextMenuItem::new("ctx-note-rename", "Rename")
                            .icon(IconName::Pencil)
                            .on_select(move |_, window, cx| {
                                rename_entity.update(cx, |this, cx| {
                                    this.begin_rename_note(&rename_id, window, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-note-duplicate", "Duplicate")
                            .icon(IconName::Copy)
                            .on_select(move |_, _, cx| {
                                duplicate_entity.update(cx, |this, cx| {
                                    this.duplicate_note(&duplicate_id, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-note-delete", "Delete")
                            .icon(IconName::Trash2)
                            .destructive(true)
                            .on_select(move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.delete_note(&delete_id, cx);
                                });
                            }),
                    )
            }
        };

        let dismiss = cx.entity();
        Some(
            ContextMenu::new("sidebar-context-menu", state.position)
                .focus_handle(self.context_menu_focus.clone())
                .on_dismiss(move |_, cx| {
                    dismiss.update(cx, |this, cx| this.close_context_menu(cx));
                })
                .content(content)
                .into_any_element(),
        )
    }

    pub fn matching_search_notes(&self, cx: &App) -> Vec<Note> {
        let query = self.search_state.value().trim().to_string();
        self.store.read(cx).search_notes(&query)
    }

    /// Inline rename editor for the folder/note row with `id`, or `None` when
    /// no rename session targets it.
    fn rename_editor_row(
        &self,
        id: &str,
        indent: Pixels,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let state = self.renaming.as_ref()?;
        if state.id != id {
            return None;
        }
        let input = state.input.clone();
        let focus = state.focus.clone();
        Some(
            div()
                .pl(indent)
                .pr_2()
                .py(px(1.))
                .child(
                    Input::new(SharedString::from(format!("rename-{id}")))
                        .state(&input)
                        .focus_handle(focus)
                        .placeholder("Name")
                        .on_key_down(cx.listener(|this, event, window, cx| {
                            this.handle_rename_key(event, window, cx);
                        })),
                )
                .into_any_element(),
        )
    }

    fn render_icon_strip(
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

    fn render_folder_tree(
        &self,
        cx: &mut Context<Self>,
    ) -> SidebarGroup {
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
            let counts: std::collections::HashMap<String, usize> = folders
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
        let notes_by_folder: std::collections::HashMap<String, Vec<Note>> =
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
                group = group.child(
                    self.render_icon_strip(folder_id, folder_icon, depth + 1, cx),
                );
            }

            if expanded {
                if let Some(notes) = notes_by_folder.get(folder_id) {
                    for note in notes {
                        let note_id = note.id.clone();
                        let is_active = selected_note.as_deref() == Some(&note_id);
                        if let Some(editor) = self.rename_editor_row(
                            &note_id,
                            px(12.0 + ((*depth + 1) as f32 * 16.0)),
                            cx,
                        ) {
                            group = group.child(editor);
                            continue;
                        }
                        group = group.child(
                            NoteTreeItem::new(
                                format!("tree-note-{}", note_id),
                                note.title.clone(),
                            )
                            .depth(depth + 1)
                            .active(is_active)
                            .pinned(note.pinned)
                            .on_click(cx.listener({
                                let note_id = note_id.clone();
                                move |this, _, _, cx| {
                                    this.select_note(&note_id, cx);
                                }
                            }))
                            .on_right_click(cx.listener(Self::note_menu_handler(
                                note_id.clone(),
                                note.title.clone(),
                                note.pinned,
                            ))),
                        );
                    }
                }
            } else {
                skip_below = Some(*depth);
            }
        }

        for note in &root_notes {
            let note_id = note.id.clone();
            let is_active = selected_note.as_deref() == Some(&note_id);
            if let Some(editor) = self.rename_editor_row(&note_id, px(12.0), cx) {
                group = group.child(editor);
                continue;
            }
            group = group.child(
                NoteTreeItem::new(format!("root-note-{}", note.id), note.title.clone())
                    .depth(0)
                    .active(is_active)
                    .pinned(note.pinned)
                    .on_click(cx.listener({
                        let note_id = note_id.clone();
                        move |this, _, _, cx| {
                            this.select_note(&note_id, cx);
                        }
                    }))
                    .on_right_click(cx.listener(Self::note_menu_handler(
                        note.id.clone(),
                        note.title.clone(),
                        note.pinned,
                    ))),
            );
        }

        group
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        // Clone store data up front; guards must not be held across listener setup.
        let (selected_note, active_location, starred_count, trash_count) = {
            let store = self.store.read(cx);
            (
                store.selected_note_id(),
                store.active_location().clone(),
                store.starred_notes().len(),
                store.trashed_notes().len(),
            )
        };
        let query = self.search_state.value().trim().to_string();

        let rail = SidebarRail::new("main-sidebar-collapsed")
            .top_item(
                SidebarRailItem::new("sidebar-expand-rail-btn", IconName::PanelLeft)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_collapsed(cx);
                    })),
            )
            .separator()
            .top_item(
                SidebarRailItem::new("sidebar-new-note-rail-btn", IconName::Plus)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.create_new_note(cx);
                    })),
            )
            .bottom_item(
                SidebarRailItem::new("sidebar-settings-rail-btn", IconName::Settings)
                    .on_click(cx.listener(|_this, _, window, cx| {
                        window.dispatch_action(Box::new(crate::keymap::OpenSettings), cx);
                    })),
            );

        let header = SidebarHeader::new()
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(20.))
                                    .h(px(20.))
                                    .rounded(px(5.))
                                    .bg(theme.primary)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        Icon::new(IconName::Zap)
                                            .size(px(12.))
                                            .color(theme.primary_foreground),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.foreground)
                                    .child("TNotes"),
                            ),
                    )
                    .child(
                        SidebarToggleButton::new("sidebar-collapse-btn")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_collapsed(cx);
                            })),
                    ),
            )
            .child(
                Button::primary("new-note-btn", "New Note")
                    .leading_icon(IconName::Plus)
                    .full_width(true)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.create_new_note(cx);
                    })),
            )
            .child(
                Input::sidebar_search("sidebar-search-input")
                    .state(&self.search_state)
                    .focus_handle(self.search_focus.clone())
                    .on_key_down(cx.listener(|this, event, window, cx| {
                        this.handle_search_key(event, window, cx);
                    }))
                    .on_clear(cx.listener(|this, _, _, cx| {
                        this.search_state.clear();
                        cx.notify();
                    })),
            );

        let mut content = SidebarContent::new();

        if !query.is_empty() {
            let matches = self.matching_search_notes(cx);
            let count = matches.len();

            let mut group = SidebarGroup::new()
                .label(format!("Search Results ({count})"));

            if matches.is_empty() {
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
                    let note_id = note.id.clone();
                    let is_active = selected_note.as_deref() == Some(&note_id);
                    if let Some(editor) = self.rename_editor_row(&note_id, px(12.0), cx) {
                        group = group.child(editor);
                        continue;
                    }
                    let right_click = Self::note_menu_handler(
                        note.id.clone(),
                        note.title.clone(),
                        note.pinned,
                    );
                    group = group.child(
                        NoteTreeItem::new(format!("search-note-{}", note.id), note.title.clone())
                            .depth(0)
                            .active(is_active)
                            .pinned(note.pinned)
                            .on_click(cx.listener({
                                let note_id = note_id.clone();
                                move |this, _, _, cx| {
                                    this.select_note(&note_id, cx);
                                }
                            }))
                            .on_right_click(cx.listener(right_click)),
                    );
                }
            }

            content = content.child(group);
        } else {
            let is_starred_active = active_location == NavigationLocation::Starred;
            let is_trash_active = active_location == NavigationLocation::Trash;

            let mut nav_group = SidebarGroup::new();

            // Starred is a virtual view over `WHERE pinned = 1`, not a folder:
            // no folder context menu here.
            nav_group = nav_group.child(
                Button::sidebar("nav-starred", "Starred")
                    .leading_icon(IconName::Star)
                    .count(starred_count)
                    .active(is_starred_active)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_starred(cx);
                    })),
            );

            nav_group = nav_group.child(
                Button::sidebar("nav-trash", "Trash")
                    .leading_icon(IconName::Trash2)
                    .count(trash_count)
                    .active(is_trash_active)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_trash(cx);
                    })),
            );

            content = content.child(nav_group);
            content = content.child(self.render_folder_tree(cx));
        }

        let footer = SidebarFooter::new().child(
            div()
                .id("sidebar-footer-settings-btn")
                .w_full()
                .h(px(34.))
                .px_2()
                .rounded(px(6.))
                .flex()
                .items_center()
                .justify_between()
                .cursor_pointer()
                .hover(|s| s.bg(theme.secondary))
                .on_click(cx.listener(|_this, _, window, cx| {
                    window.dispatch_action(Box::new(crate::keymap::OpenSettings), cx);
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .w(px(22.))
                                .h(px(22.))
                                .rounded_full()
                                .bg(theme.muted)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(11.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground)
                                .child("A"),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground)
                                .child("Personal Vault"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(IconName::Settings).size(px(14.))),
                ),
        );

        let sidebar = Sidebar::new("main-sidebar")
            .width(px(260.))
            .collapsed(self.is_collapsed)
            .collapsible(SidebarCollapsible::Icon)
            .header(header)
            .content(content)
            .footer(footer)
            .rail(rail)
            .into_any_element();

        // The wrapper sits at the window origin (first flex child), so the
        // menu's window-coordinate offsets resolve 1:1 onto it. The deferred
        // menu paints above the sidebar without being clipped by the scroll
        // area. (The menu must stay a direct child of this wrapper: any
        // in-flow element between them would shift the absolute offsets by
        // its own origin and push the popup off-screen.)
        if let Some(menu) = self.context_menu_element(cx) {
            div()
                .id("sidebar-with-context-menu")
                .h_full()
                .child(sidebar)
                .child(menu)
                .into_any_element()
        } else {
            sidebar
        }
    }
}

#[cfg(test)]
mod tests {
    // NOTE: no glob imports here on purpose. `use gpui::*` would pull the
    // `gpui::test` attribute macro into scope, shadowing the builtin `#[test]`
    // and breaking compilation of this module.
    use super::{
        SidebarContextTarget, SidebarView, folder_icon_for, folder_icon_from_name,
        folder_icon_name, FOLDER_ICON_OPTIONS,
    };
    use crate::components::IconName;
    use crate::store::{NavigationLocation, NoteStore};
    use crate::theme::{ActiveTheme, Theme};
    use gpui::{AppContext, MouseButton, MouseDownEvent, TestAppContext, point, px};

    #[test]
    fn folder_icon_table_roundtrips_and_falls_back() {
        assert_eq!(FOLDER_ICON_OPTIONS.len(), 17);
        assert_eq!(folder_icon_name(IconName::Briefcase), "briefcase");
        assert_eq!(folder_icon_from_name("rocket"), Some(IconName::Rocket));
        assert_eq!(folder_icon_from_name("  STAR  "), Some(IconName::Star));
        assert_eq!(folder_icon_from_name("nope"), None);
        // Legacy emoji + unknown values fall back to open/closed glyphs.
        assert_eq!(folder_icon_for("📁", true), IconName::FolderOpen);
        assert_eq!(folder_icon_for("???", false), IconName::Folder);
        assert_eq!(folder_icon_for("code", false), IconName::Code);
    }

    #[test]
    fn icon_picker_toggle_and_pick_flow() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert!(view.read_with(cx, |v, _| v.picking_icon_for.is_none()));

        view.update(cx, |v, cx| v.toggle_icon_picker("projects", cx));
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |v, _| v.picking_icon_for.clone()),
            Some("projects".to_string())
        );

        view.update(cx, |v, cx| v.toggle_icon_picker("projects", cx));
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.picking_icon_for.is_none()));

        view.update(cx, |v, cx| {
            v.toggle_icon_picker("projects", cx);
            v.pick_folder_icon("projects", IconName::Rocket, cx);
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.picking_icon_for.is_none()));
        assert_eq!(
            store.read_with(cx, |s, _| s
                .folder_tree()
                .iter()
                .find(|n| n.folder.id == "projects")
                .map(|n| n.folder.icon.clone())),
            Some("rocket".to_string())
        );
    }

    #[test]
    fn rename_commit_and_cancel_flow() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        // Folder rename: begin → type → commit.
        cx.update(|window, cx| {
            view.update(cx, |v, cx| v.begin_rename_folder("projects", window, cx));
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_some()));
        view.update(cx, |v, cx| {
            if let Some(state) = v.renaming.as_mut() {
                state.input.set_value("Renamed");
            }
            v.commit_rename(cx);
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_none()));
        assert_eq!(
            store.read_with(cx, |s, _| s
                .folder_tree()
                .iter()
                .find(|n| n.folder.id == "projects")
                .map(|n| n.folder.name.clone())),
            Some("Renamed".to_string())
        );

        // Note rename: begin → cancel leaves the title untouched.
        cx.update(|window, cx| {
            view.update(cx, |v, cx| v.begin_rename_note("note-db-schema", window, cx));
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_some()));
        view.update(cx, |v, cx| v.cancel_rename(cx));
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_none()));
        assert_eq!(
            store.read_with(cx, |s, _| s
                .active_notes()
                .into_iter()
                .find(|n| n.id == "note-db-schema")
                .map(|n| n.title)),
            Some("Database Schema & Index Design".to_string())
        );
    }

    /// Regression loop for "right-click in the sidebar does nothing".
    /// Drives the real event-dispatch path: synthetic right mouse-down events
    /// sweep down the sidebar until a folder row opens the context menu.
    #[test]
    fn right_click_folder_row_opens_context_menu() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        let mut opened = None;
        let mut seen_folders = Vec::new();
        let mut y = 150.;
        while y < 700. {
            cx.simulate_event(MouseDownEvent {
                button: MouseButton::Right,
                position: point(px(130.), px(y)),
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            });
            cx.run_until_parked();
            if let Some(state) = view.read_with(cx, |v, _| v.context_menu.clone()) {
                if let SidebarContextTarget::Folder { id, .. } = &state.target {
                    seen_folders.push(id.clone());
                    if id == "projects" {
                        opened = Some((y, state));
                        break;
                    }
                }
            }
            y += 4.;
        }

        let (y, state) = opened.expect("right-click should open a context menu on Projects");
        match state.target {
            SidebarContextTarget::Folder { id, .. } => {
                assert_eq!(id, "projects", "folder row hit while sweeping at y={y}")
            }
            other => panic!("expected folder target at y={y}, got {other:?}"),
        }
    }

    /// Full user flow: right-click a note row, left-click the first menu item
    /// ("Pin to Starred"), and assert the note flips to pinned. This covers
    /// dispatch → state → render → hit-test → action end to end.
    #[test]
    fn right_click_note_then_pin_item_pins_note() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        view.update(cx, |v, cx| v.toggle_folder("projects", cx));
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // Sweep until the "Database Schema" note row opens its menu.
        let mut menu_position = None;
        let mut y = 150.;
        while y < 700. {
            cx.simulate_event(MouseDownEvent {
                button: MouseButton::Right,
                position: point(px(130.), px(y)),
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            });
            cx.run_until_parked();
            if let Some(state) = view.read_with(cx, |v, _| v.context_menu.clone()) {
                if let SidebarContextTarget::Note { id, .. } = &state.target {
                    if id == "note-db-schema" {
                        menu_position = Some(state.position);
                        break;
                    }
                }
            }
            y += 4.;
        }
        let menu_position =
            menu_position.expect("right-click should open a context menu on note-db-schema");

        assert!(
            !store
                .read_with(cx, |s, _| s
                    .active_notes()
                    .iter()
                    .find(|n| n.id == "note-db-schema")
                    .map(|n| n.pinned)
                    .unwrap_or(true)),
            "note-db-schema should start unpinned"
        );

        // Redraw with the menu open so hitboxes exist, then press the first
        // item ("Pin to Starred"): label (~22px) + half the 28px row, past
        // the p_1 (4px) padding. Items activate on mouse-down.
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });
        cx.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position: point(menu_position.x + px(100.), menu_position.y + px(40.)),
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();

        assert!(
            store
                .read_with(cx, |s, _| s
                    .active_notes()
                    .iter()
                    .find(|n| n.id == "note-db-schema")
                    .map(|n| n.pinned)
                    .unwrap_or(false)),
            "clicking Pin to Starred should pin note-db-schema"
        );
        assert!(
            view.read_with(cx, |v, _| v.context_menu.is_none()),
            "menu should close after selecting an item"
        );
    }

    #[test]
    fn toggle_collapsed_switches_between_expanded_and_rail() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        cx.run_until_parked();

        assert!(!view.read_with(cx, |v, _| v.is_collapsed));

        view.update(cx, |v, cx| {
            v.toggle_collapsed(cx);
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.is_collapsed));

        view.update(cx, |v, cx| {
            v.toggle_collapsed(cx);
        });
        cx.run_until_parked();
        assert!(!view.read_with(cx, |v, _| v.is_collapsed));
    }

    #[test]
    fn sidebar_view_note_navigation_back_and_forward() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );
        assert!(!store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(!store.read_with(cx, |s, _| s.can_navigate_forward()));

        view.update(cx, |v, cx| {
            v.select_note("note-desktop-gpui", cx);
        });
        cx.run_until_parked();
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-desktop-gpui".to_string())
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(!store.read_with(cx, |s, _| s.can_navigate_forward()));

        view.update(cx, |v, cx| {
            v.select_note("note-db-schema", cx);
        });
        cx.run_until_parked();

        let went_back = view.update(cx, |v, cx| v.navigate_back(cx));
        assert!(went_back);
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-desktop-gpui".to_string())
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(store.read_with(cx, |s, _| s.can_navigate_forward()));

        let went_back = view.update(cx, |v, cx| v.navigate_back(cx));
        assert!(went_back);
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );
        assert!(!store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(store.read_with(cx, |s, _| s.can_navigate_forward()));

        let went_forward = view.update(cx, |v, cx| v.navigate_forward(cx));
        assert!(went_forward);
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-desktop-gpui".to_string())
        );
    }

    #[test]
    fn folder_toggle_does_not_select_folder_or_affect_note_selection() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.toggle_folder("projects", cx);
        });
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.open_starred(cx);
        });
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Starred
        );

        view.update(cx, |v, cx| {
            v.open_trash(cx);
        });
        cx.run_until_parked();
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Trash
        );

        view.update(cx, |v, cx| {
            v.select_note("note-desktop-gpui", cx);
        });
        cx.run_until_parked();
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Note("note-desktop-gpui".to_string())
        );
    }

    #[test]
    fn test_starred_and_trash_navigation_and_item_actions() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Note("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| v.open_starred(cx));
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Starred
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(!store.read_with(cx, |s, _| s.starred_notes().is_empty()));

        view.update(cx, |v, cx| v.open_trash(cx));
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Trash
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));

        view.update(cx, |v, cx| {
            assert!(v.navigate_back(cx));
        });
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Starred
        );

        view.update(cx, |v, cx| {
            assert!(v.navigate_back(cx));
        });
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Note("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.delete_note("note-arch-spec", cx);
        });
        assert_eq!(
            store.read_with(cx, |s, _| s
                .trash_notes()
                .iter()
                .any(|n| n.id == "note-arch-spec")),
            true
        );

        view.update(cx, |v, cx| {
            v.open_trash(cx);
            v.restore_note("note-arch-spec", cx);
        });
        assert_eq!(
            store.read_with(cx, |s, _| s
                .trash_notes()
                .iter()
                .any(|n| n.id == "note-arch-spec")),
            false
        );
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Trash
        );
        assert!(
            store.read_with(cx, |s, _| s
                .active_notes()
                .iter()
                .any(|n| n.id == "note-arch-spec"))
        );

        view.update(cx, |v, cx| {
            v.delete_note("note-arch-spec", cx);
            v.empty_trash(cx);
        });
        assert!(store.read_with(cx, |s, _| s.trash_notes().is_empty()),);
    }
}
