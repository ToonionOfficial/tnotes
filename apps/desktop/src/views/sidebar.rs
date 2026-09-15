use std::collections::HashSet;
use gpui::*;
use crate::components::{
    Button, ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel,
    ContextMenuSeparator, FolderTreeItem, Icon, IconName, Input, InputState, NoteTreeItem, Sidebar,
    SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarHeader, SidebarRail,
    SidebarRailItem, SidebarToggleButton,
};
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct NoteItem {
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub updated_at: String,
    pub folder_id: Option<String>,
    pub folder_name: Option<String>,
    pub is_pinned: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavigationHistory {
    pub back_stack: Vec<String>,
    pub forward_stack: Vec<String>,
}

impl NavigationHistory {
    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    pub fn push(&mut self, current: Option<&str>, next: &str) {
        if let Some(cur) = current {
            if cur != next {
                self.back_stack.push(cur.to_string());
                self.forward_stack.clear();
            }
        }
    }

    pub fn go_back(&mut self, current: Option<&str>) -> Option<String> {
        let prev = self.back_stack.pop()?;
        if let Some(cur) = current {
            self.forward_stack.push(cur.to_string());
        }
        Some(prev)
    }

    pub fn go_forward(&mut self, current: Option<&str>) -> Option<String> {
        let next = self.forward_stack.pop()?;
        if let Some(cur) = current {
            self.back_stack.push(cur.to_string());
        }
        Some(next)
    }

    pub fn remove_note(&mut self, note_id: &str) {
        self.back_stack.retain(|id| id != note_id);
        self.forward_stack.retain(|id| id != note_id);
    }
}

pub struct SidebarView {
    is_collapsed: bool,
    search_state: InputState,
    search_focus: FocusHandle,
    selected_section: String,
    selected_note_id: Option<String>,
    history: NavigationHistory,
    expanded_folders: HashSet<String>,
    starred_expanded: bool,
    notes: Vec<NoteItem>,
    context_menu: Option<SidebarContextMenu>,
    context_menu_focus: FocusHandle,
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
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut expanded_folders = HashSet::new();
        expanded_folders.insert("projects".to_string());
        expanded_folders.insert("projects:architecture".to_string());
        expanded_folders.insert("projects:architecture:core".to_string());
        expanded_folders.insert("projects:architecture:desktop".to_string());
        expanded_folders.insert("personal".to_string());
        expanded_folders.insert("personal:journal".to_string());

        let notes = vec![
            NoteItem {
                id: "note-arch-spec".to_string(),
                title: "System Architecture Spec".to_string(),
                snippet: "Local-first SQLite with FTS5, CRDT synchronization, and DankeShell token design system.".to_string(),
                updated_at: "Just now".to_string(),
                folder_id: Some("projects:architecture:core".to_string()),
                folder_name: Some("Core Engine".to_string()),
                is_pinned: true,
            },
            NoteItem {
                id: "note-desktop-gpui".to_string(),
                title: "Desktop Shell & GPUI Architecture".to_string(),
                snippet: "Unified Obsidian-style sidebar, borderless notes, twrite rope canvas rendering.".to_string(),
                updated_at: "15m ago".to_string(),
                folder_id: Some("projects:architecture:desktop".to_string()),
                folder_name: Some("Desktop Shell".to_string()),
                is_pinned: true,
            },
            NoteItem {
                id: "note-db-schema".to_string(),
                title: "Database Schema & Index Design".to_string(),
                snippet: "Tables for notes, folders, tags, sync operations log, and tokenized full-text search indexes.".to_string(),
                updated_at: "2h ago".to_string(),
                folder_id: Some("projects".to_string()),
                folder_name: Some("Projects".to_string()),
                is_pinned: false,
            },
            NoteItem {
                id: "note-ws-sync".to_string(),
                title: "WebSocket Sync Protocol".to_string(),
                snippet: "Bidirectional binary and JSON delta streaming between mobile client and desktop server.".to_string(),
                updated_at: "Yesterday".to_string(),
                folder_id: Some("projects".to_string()),
                folder_name: Some("Projects".to_string()),
                is_pinned: false,
            },
            NoteItem {
                id: "note-q3-roadmap".to_string(),
                title: "Q3 Roadmap & Planning".to_string(),
                snippet: "Offline-first conflict resolution, canvas mode, graph view, and end-to-end encryption.".to_string(),
                updated_at: "Sep 12".to_string(),
                folder_id: Some("personal".to_string()),
                folder_name: Some("Personal Notes".to_string()),
                is_pinned: false,
            },
            NoteItem {
                id: "note-sprint-goals".to_string(),
                title: "Weekly Sprint Goals".to_string(),
                snippet: "Implement single-sidebar layout, DankeShell active states, and twrite editor integration.".to_string(),
                updated_at: "Sep 10".to_string(),
                folder_id: Some("personal:journal".to_string()),
                folder_name: Some("Journal".to_string()),
                is_pinned: false,
            },
            NoteItem {
                id: "note-design-inspo".to_string(),
                title: "Design Inspiration: Obsidian x Notion".to_string(),
                snippet: "Collapsible sidebar rail, clean hierarchy, distraction-free markdown canvas, quiet chrome.".to_string(),
                updated_at: "Sep 8".to_string(),
                folder_id: Some("personal:journal".to_string()),
                folder_name: Some("Journal".to_string()),
                is_pinned: false,
            },
        ];

        let selected_note_id = Some("note-arch-spec".to_string());

        Self {
            is_collapsed: false,
            search_state: InputState::new(""),
            search_focus: cx.focus_handle(),
            selected_section: "projects".to_string(),
            selected_note_id,
            history: NavigationHistory::default(),
            expanded_folders,
            starred_expanded: false,
            notes,
            context_menu: None,
            context_menu_focus: cx.focus_handle(),
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

    pub fn selected_note_id(&self) -> Option<&str> {
        self.selected_note_id.as_deref()
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

    pub fn toggle_starred(&mut self, cx: &mut Context<Self>) {
        self.starred_expanded = !self.starred_expanded;
        cx.notify();
    }

    pub fn select_section(&mut self, section_id: &str, cx: &mut Context<Self>) {
        self.selected_section = section_id.to_string();
        cx.notify();
    }

    pub fn select_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if self.selected_note_id.as_deref() == Some(note_id) {
            return;
        }
        self.history.push(self.selected_note_id.as_deref(), note_id);
        self.selected_note_id = Some(note_id.to_string());
        cx.notify();
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(prev) = self.history.go_back(self.selected_note_id.as_deref()) {
            self.selected_note_id = Some(prev);
            cx.notify();
            true
        } else {
            false
        }
    }

    pub fn navigate_forward(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(next) = self.history.go_forward(self.selected_note_id.as_deref()) {
            self.selected_note_id = Some(next);
            cx.notify();
            true
        } else {
            false
        }
    }

    pub fn can_navigate_back(&self) -> bool {
        self.history.can_go_back()
    }

    pub fn can_navigate_forward(&self) -> bool {
        self.history.can_go_forward()
    }

    pub fn selected_note(&self) -> Option<&NoteItem> {
        let id = self.selected_note_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == id)
    }

    pub fn create_new_note(&mut self, cx: &mut Context<Self>) {
        self.create_note_in_folder(None, cx);
    }

    pub fn create_note_in_folder(&mut self, folder_id: Option<String>, cx: &mut Context<Self>) {
        let id = format!("note-{}", self.notes.len() + 1);
        let new_note = NoteItem {
            id: id.clone(),
            title: "Untitled Note".to_string(),
            snippet: "Start typing your note here...".to_string(),
            updated_at: "Just now".to_string(),
            folder_id,
            folder_name: None,
            is_pinned: false,
        };
        self.notes.insert(0, new_note);
        self.select_note(&id, cx);
        self.context_menu = None;
    }

    pub fn toggle_note_pin(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            note.is_pinned = !note.is_pinned;
        }
        self.context_menu = None;
        cx.notify();
    }

    pub fn duplicate_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(index) = self.notes.iter().position(|n| n.id == note_id) {
            let source = self.notes[index].clone();
            let id = format!("note-{}-copy", self.notes.len() + 1);
            let copy = NoteItem {
                id: id.clone(),
                title: format!("{} (copy)", source.title),
                snippet: source.snippet,
                updated_at: "Just now".to_string(),
                folder_id: source.folder_id,
                folder_name: source.folder_name,
                is_pinned: false,
            };
            self.notes.insert(index + 1, copy);
            self.select_note(&id, cx);
        }
        self.context_menu = None;
    }

    pub fn delete_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.notes.retain(|n| n.id != note_id);
        self.history.remove_note(note_id);
        if self.selected_note_id.as_deref() == Some(note_id) {
            self.selected_note_id = self.history.back_stack.pop().or_else(|| self.notes.first().map(|n| n.id.clone()));
        }
        self.context_menu = None;
        cx.notify();
    }

    pub fn rename_note(&mut self, _note_id: &str, cx: &mut Context<Self>) {
        // TODO: open an inline rename editor for the note.
        self.context_menu = None;
        cx.notify();
    }

    pub fn create_subfolder(&mut self, _folder_id: &str, cx: &mut Context<Self>) {
        // TODO: folders are hardcoded mock data; wire to a real folder model.
        self.context_menu = None;
        cx.notify();
    }

    pub fn rename_folder(&mut self, _folder_id: &str, cx: &mut Context<Self>) {
        // TODO: open an inline rename editor for the folder.
        self.context_menu = None;
        cx.notify();
    }

    pub fn delete_folder(&mut self, _folder_id: &str, cx: &mut Context<Self>) {
        // TODO: folders are hardcoded mock data; wire to a real folder model.
        self.context_menu = None;
        cx.notify();
    }

    pub fn open_context_menu(
        &mut self,
        position: Point<Pixels>,
        target: SidebarContextTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Clamp so the ~200px wide menu stays inside the viewport.
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
                        ContextMenuItem::new("ctx-folder-rename", "Rename")
                            .icon(IconName::Pencil)
                            .on_select(move |_, _, cx| {
                                rename_entity.update(cx, |this, cx| {
                                    this.rename_folder(&rename_id, cx);
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
                            .on_select(move |_, _, cx| {
                                rename_entity.update(cx, |this, cx| {
                                    this.rename_note(&rename_id, cx);
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

    pub fn matching_search_notes(&self) -> Vec<&NoteItem> {
        let query = self.search_state.value().trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        self.notes
            .iter()
            .filter(|n| {
                n.title.to_lowercase().contains(&query)
                    || n.snippet.to_lowercase().contains(&query)
                    || n.folder_name
                        .as_ref()
                        .map(|f| f.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect()
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let selected_note = self.selected_note_id.clone();
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
            let matches = self.matching_search_notes();
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
                    let right_click = Self::note_menu_handler(
                        note.id.clone(),
                        note.title.clone(),
                        note.is_pinned,
                    );
                    group = group.child(
                        NoteTreeItem::new(format!("search-note-{}", note.id), note.title.clone())
                            .depth(0)
                            .active(is_active)
                            .pinned(note.is_pinned)
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
            let starred_notes: Vec<&NoteItem> = self.notes.iter().filter(|n| n.is_pinned).collect();
            let starred_count = starred_notes.len();

            let mut nav_group = SidebarGroup::new();

            nav_group = nav_group.child(
                FolderTreeItem::new("nav-starred", "Starred")
                    .depth(0)
                    .icon(IconName::Star)
                    .expanded(self.starred_expanded)
                    .count(starred_count)
                    .on_toggle(cx.listener(|this, _, _, cx| {
                        this.toggle_starred(cx);
                    }))
                    .on_right_click(cx.listener(Self::folder_menu_handler(
                        "starred".to_string(),
                        "Starred".to_string(),
                    ))),
            );

            if self.starred_expanded {
                for note in starred_notes {
                    let note_id = note.id.clone();
                    let is_active = selected_note.as_deref() == Some(&note_id);
                    let right_click = Self::note_menu_handler(
                        note.id.clone(),
                        note.title.clone(),
                        note.is_pinned,
                    );
                    nav_group = nav_group.child(
                        NoteTreeItem::new(format!("starred-note-{}", note.id), note.title.clone())
                            .depth(1)
                            .active(is_active)
                            .pinned(true)
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

            nav_group = nav_group.child(
                Button::sidebar("trash", "Trash")
                    .leading_icon(IconName::Trash2)
                    .count(0)
                    .active(self.selected_section == "trash")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.select_section("trash", cx);
                    })),
            );

            content = content.child(nav_group);

            let projects_expanded = self.expanded_folders.contains("projects");
            let arch_expanded = self.expanded_folders.contains("projects:architecture");
            let core_expanded = self.expanded_folders.contains("projects:architecture:core");
            let desktop_expanded = self.expanded_folders.contains("projects:architecture:desktop");
            let personal_expanded = self.expanded_folders.contains("personal");
            let journal_expanded = self.expanded_folders.contains("personal:journal");

            let mut folders_group = SidebarGroup::new()
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
                        .child(Icon::new(IconName::Plus).size(px(12.))),
                );

            folders_group = folders_group.child(
                FolderTreeItem::new("folder-projects", "Projects")
                    .depth(0)
                    .expanded(projects_expanded)
                    .count(4)
                    .on_toggle(cx.listener(|this, _, _, cx| {
                        this.toggle_folder("projects", cx);
                    }))
                    .on_right_click(cx.listener(Self::folder_menu_handler(
                        "projects".to_string(),
                        "Projects".to_string(),
                    ))),
            );

            if projects_expanded {
                folders_group = folders_group.child(
                    FolderTreeItem::new("folder-architecture", "Architecture")
                        .depth(1)
                        .icon(IconName::Code)
                        .expanded(arch_expanded)
                        .count(2)
                        .on_toggle(cx.listener(|this, _, _, cx| {
                            this.toggle_folder("projects:architecture", cx);
                        }))
                        .on_right_click(cx.listener(Self::folder_menu_handler(
                            "projects:architecture".to_string(),
                            "Architecture".to_string(),
                        ))),
                );

                if arch_expanded {
                    folders_group = folders_group.child(
                        FolderTreeItem::new("folder-arch-core", "Core Engine")
                            .depth(2)
                            .icon(IconName::Zap)
                            .expanded(core_expanded)
                            .count(1)
                            .on_toggle(cx.listener(|this, _, _, cx| {
                                this.toggle_folder("projects:architecture:core", cx);
                            }))
                            .on_right_click(cx.listener(Self::folder_menu_handler(
                                "projects:architecture:core".to_string(),
                                "Core Engine".to_string(),
                            ))),
                    );

                    if core_expanded {
                        let is_active = selected_note.as_deref() == Some("note-arch-spec");
                        folders_group = folders_group.child(
                            NoteTreeItem::new("tree-note-arch-spec", "System Architecture Spec")
                                .depth(3)
                                .active(is_active)
                                .pinned(true)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.select_note("note-arch-spec", cx);
                                }))
                                .on_right_click(cx.listener(Self::note_menu_handler(
                                    "note-arch-spec".to_string(),
                                    "System Architecture Spec".to_string(),
                                    true,
                                    ))),
                        );
                    }

                    folders_group = folders_group.child(
                        FolderTreeItem::new("folder-arch-desktop", "Desktop Shell")
                            .depth(2)
                            .icon(IconName::Rocket)
                            .expanded(desktop_expanded)
                            .count(1)
                            .on_toggle(cx.listener(|this, _, _, cx| {
                                this.toggle_folder("projects:architecture:desktop", cx);
                            }))
                            .on_right_click(cx.listener(Self::folder_menu_handler(
                                "projects:architecture:desktop".to_string(),
                                "Desktop Shell".to_string(),
                            ))),
                    );

                    if desktop_expanded {
                        let is_active = selected_note.as_deref() == Some("note-desktop-gpui");
                        folders_group = folders_group.child(
                            NoteTreeItem::new("tree-note-desktop-gpui", "Desktop Shell & GPUI Architecture")
                                .depth(3)
                                .active(is_active)
                                .pinned(true)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.select_note("note-desktop-gpui", cx);
                                }))
                                .on_right_click(cx.listener(Self::note_menu_handler(
                                    "note-desktop-gpui".to_string(),
                                    "Desktop Shell & GPUI Architecture".to_string(),
                                    true,
                                ))),
                        );
                    }
                }

                let is_db_active = selected_note.as_deref() == Some("note-db-schema");
                folders_group = folders_group.child(
                    NoteTreeItem::new("tree-note-db-schema", "Database Schema & Index Design")
                        .depth(1)
                        .active(is_db_active)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.select_note("note-db-schema", cx);
                        }))
                        .on_right_click(cx.listener(Self::note_menu_handler(
                            "note-db-schema".to_string(),
                            "Database Schema & Index Design".to_string(),
                            false,
                        ))),
                );

                let is_ws_active = selected_note.as_deref() == Some("note-ws-sync");
                folders_group = folders_group.child(
                    NoteTreeItem::new("tree-note-ws-sync", "WebSocket Sync Protocol")
                        .depth(1)
                        .active(is_ws_active)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.select_note("note-ws-sync", cx);
                        }))
                        .on_right_click(cx.listener(Self::note_menu_handler(
                            "note-ws-sync".to_string(),
                            "WebSocket Sync Protocol".to_string(),
                            false,
                        ))),
                );

                folders_group = folders_group.child(
                    FolderTreeItem::new("folder-specs", "Specifications")
                        .depth(1)
                        .icon(IconName::Briefcase)
                        .count(0)
                        .on_right_click(cx.listener(Self::folder_menu_handler(
                            "specs".to_string(),
                            "Specifications".to_string(),
                        ))),
                );
            }

            folders_group = folders_group.child(
                FolderTreeItem::new("folder-personal", "Personal Notes")
                    .depth(0)
                    .expanded(personal_expanded)
                    .count(3)
                    .on_toggle(cx.listener(|this, _, _, cx| {
                        this.toggle_folder("personal", cx);
                    }))
                    .on_right_click(cx.listener(Self::folder_menu_handler(
                        "personal".to_string(),
                        "Personal Notes".to_string(),
                    ))),
            );

            if personal_expanded {
                folders_group = folders_group.child(
                    FolderTreeItem::new("folder-journal", "Journal")
                        .depth(1)
                        .icon(IconName::Bookmark)
                        .expanded(journal_expanded)
                        .count(2)
                        .on_toggle(cx.listener(|this, _, _, cx| {
                            this.toggle_folder("personal:journal", cx);
                        }))
                        .on_right_click(cx.listener(Self::folder_menu_handler(
                            "personal:journal".to_string(),
                            "Journal".to_string(),
                        ))),
                );

                if journal_expanded {
                    let is_goals_active = selected_note.as_deref() == Some("note-sprint-goals");
                    folders_group = folders_group.child(
                        NoteTreeItem::new("tree-note-sprint-goals", "Weekly Sprint Goals")
                            .depth(2)
                            .active(is_goals_active)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_note("note-sprint-goals", cx);
                            }))
                            .on_right_click(cx.listener(Self::note_menu_handler(
                                "note-sprint-goals".to_string(),
                                "Weekly Sprint Goals".to_string(),
                                false,
                            ))),
                    );

                    let is_inspo_active = selected_note.as_deref() == Some("note-design-inspo");
                    folders_group = folders_group.child(
                        NoteTreeItem::new("tree-note-design-inspo", "Design Inspiration: Obsidian x Notion")
                            .depth(2)
                            .active(is_inspo_active)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_note("note-design-inspo", cx);
                            }))
                            .on_right_click(cx.listener(Self::note_menu_handler(
                                "note-design-inspo".to_string(),
                                "Design Inspiration: Obsidian x Notion".to_string(),
                                false,
                            ))),
                    );
                }

                let is_roadmap_active = selected_note.as_deref() == Some("note-q3-roadmap");
                folders_group = folders_group.child(
                    NoteTreeItem::new("tree-note-q3-roadmap", "Q3 Roadmap & Planning")
                        .depth(1)
                        .active(is_roadmap_active)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.select_note("note-q3-roadmap", cx);
                        }))
                        .on_right_click(cx.listener(Self::note_menu_handler(
                            "note-q3-roadmap".to_string(),
                            "Q3 Roadmap & Planning".to_string(),
                            false,
                        ))),
                );
            }

            for note in self.notes.iter().filter(|n| n.folder_id.is_none()) {
                let note_id = note.id.clone();
                let is_active = selected_note.as_deref() == Some(&note_id);
                let right_click = Self::note_menu_handler(
                    note.id.clone(),
                    note.title.clone(),
                    note.is_pinned,
                );
                folders_group = folders_group.child(
                    NoteTreeItem::new(format!("root-note-{}", note.id), note.title.clone())
                        .depth(0)
                        .active(is_active)
                        .pinned(note.is_pinned)
                        .on_click(cx.listener({
                            let note_id = note_id.clone();
                            move |this, _, _, cx| {
                                this.select_note(&note_id, cx);
                            }
                        }))
                        .on_right_click(cx.listener(right_click)),
                );
            }

            content = content.child(folders_group);
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
    use super::{SidebarContextTarget, SidebarView};
    use crate::theme::{ActiveTheme, Theme};
    use gpui::{MouseButton, MouseDownEvent, TestAppContext, point, px};

    /// Regression loop for "right-click in the sidebar does nothing".
    /// Drives the real event-dispatch path: synthetic right mouse-down events
    /// sweep down the sidebar until a folder row opens the context menu.
    #[test]
    fn right_click_folder_row_opens_context_menu() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| SidebarView::new(cx));
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

        assert!(
            seen_folders.iter().any(|id| id == "starred"),
            "expected to hit the Starred row while sweeping, saw: {seen_folders:?}"
        );
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

        let (view, cx) = cx.add_window_view(|_, cx| SidebarView::new(cx));
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
            !view.read_with(cx, |v, _| v
                .notes
                .iter()
                .find(|n| n.id == "note-db-schema")
                .map(|n| n.is_pinned)
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
            view.read_with(cx, |v, _| v
                .notes
                .iter()
                .find(|n| n.id == "note-db-schema")
                .map(|n| n.is_pinned)
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

        let (view, cx) = cx.add_window_view(|_, cx| SidebarView::new(cx));
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
    fn navigation_history_stack_operations() {
        let mut history = super::NavigationHistory::default();
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(Some("note-a"), "note-b");
        history.push(Some("note-b"), "note-c");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(Some("note-c"), "note-c");
        assert_eq!(history.back_stack.len(), 2);

        let prev = history.go_back(Some("note-c"));
        assert_eq!(prev.as_deref(), Some("note-b"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        let prev = history.go_back(Some("note-b"));
        assert_eq!(prev.as_deref(), Some("note-a"));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        let next = history.go_forward(Some("note-a"));
        assert_eq!(next.as_deref(), Some("note-b"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        history.push(Some("note-b"), "note-d");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.remove_note("note-a");
        assert_eq!(history.back_stack, vec!["note-b".to_string()]);
    }

    #[test]
    fn sidebar_view_note_navigation_back_and_forward() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| SidebarView::new(cx));
        cx.run_until_parked();

        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-arch-spec".to_string())
        );
        assert!(!view.read_with(cx, |v, _| v.can_navigate_back()));
        assert!(!view.read_with(cx, |v, _| v.can_navigate_forward()));

        view.update(cx, |v, cx| {
            v.select_note("note-desktop-gpui", cx);
        });
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-desktop-gpui".to_string())
        );
        assert!(view.read_with(cx, |v, _| v.can_navigate_back()));
        assert!(!view.read_with(cx, |v, _| v.can_navigate_forward()));

        view.update(cx, |v, cx| {
            v.select_note("note-db-schema", cx);
        });
        cx.run_until_parked();

        let went_back = view.update(cx, |v, cx| v.navigate_back(cx));
        assert!(went_back);
        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-desktop-gpui".to_string())
        );
        assert!(view.read_with(cx, |v, _| v.can_navigate_back()));
        assert!(view.read_with(cx, |v, _| v.can_navigate_forward()));

        let went_back = view.update(cx, |v, cx| v.navigate_back(cx));
        assert!(went_back);
        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-arch-spec".to_string())
        );
        assert!(!view.read_with(cx, |v, _| v.can_navigate_back()));
        assert!(view.read_with(cx, |v, _| v.can_navigate_forward()));

        let went_forward = view.update(cx, |v, cx| v.navigate_forward(cx));
        assert!(went_forward);
        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-desktop-gpui".to_string())
        );
    }

    #[test]
    fn folder_toggle_does_not_select_folder_or_affect_note_selection() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| SidebarView::new(cx));
        cx.run_until_parked();

        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.toggle_folder("projects", cx);
        });
        cx.run_until_parked();

        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.toggle_starred(cx);
        });
        cx.run_until_parked();

        assert_eq!(
            view.read_with(cx, |v, _| v.selected_note_id().map(|s| s.to_string())),
            Some("note-arch-spec".to_string())
        );
        assert_ne!(
            view.read_with(cx, |v, _| v.selected_section.clone()),
            "starred"
        );
    }
}
