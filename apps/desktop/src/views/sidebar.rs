use std::collections::HashSet;
use gpui::*;
use crate::components::{
    Button, FolderTreeItem, Icon, IconName, Input, NoteTreeItem, Sidebar, SidebarContent,
    SidebarFooter, SidebarGroup, SidebarHeader,
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

pub struct SidebarView {
    is_collapsed: bool,
    search_query: String,
    search_focus: FocusHandle,
    selected_section: String,
    selected_note_id: Option<String>,
    expanded_folders: HashSet<String>,
    starred_expanded: bool,
    notes: Vec<NoteItem>,
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
            search_query: String::new(),
            search_focus: cx.focus_handle(),
            selected_section: "projects".to_string(),
            selected_note_id,
            expanded_folders,
            starred_expanded: false,
            notes,
        }
    }

    fn handle_search_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = &event.keystroke.key;
        if key == "backspace" {
            self.search_query.pop();
            cx.notify();
        } else if key == "escape" {
            self.search_query.clear();
            window.blur();
            cx.notify();
        } else if key == "enter" {
            window.blur();
            cx.notify();
        } else if !event.keystroke.modifiers.platform
            && !event.keystroke.modifiers.control
            && !event.keystroke.modifiers.alt
        {
            if let Some(ref text) = event.keystroke.key_char {
                if !text.is_empty() && !text.chars().all(|c| c.is_control()) {
                    self.search_query.push_str(text);
                    cx.notify();
                }
            } else if key == "space" {
                self.search_query.push(' ');
                cx.notify();
            } else if key.chars().count() == 1 {
                self.search_query.push_str(key);
                cx.notify();
            }
        }
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.is_collapsed = !self.is_collapsed;
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
        self.selected_section = "starred".to_string();
        cx.notify();
    }

    pub fn select_section(&mut self, section_id: &str, cx: &mut Context<Self>) {
        self.selected_section = section_id.to_string();
        cx.notify();
    }

    pub fn select_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.selected_note_id = Some(note_id.to_string());
        cx.notify();
    }

    pub fn selected_note(&self) -> Option<&NoteItem> {
        let id = self.selected_note_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == id)
    }

    pub fn create_new_note(&mut self, cx: &mut Context<Self>) {
        let id = format!("note-{}", self.notes.len() + 1);
        let new_note = NoteItem {
            id: id.clone(),
            title: "Untitled Note".to_string(),
            snippet: "Start typing your note here...".to_string(),
            updated_at: "Just now".to_string(),
            folder_id: None,
            folder_name: None,
            is_pinned: false,
        };
        self.notes.insert(0, new_note);
        self.selected_note_id = Some(id);
        cx.notify();
    }

    pub fn matching_search_notes(&self) -> Vec<&NoteItem> {
        let query = self.search_query.trim().to_lowercase();
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
        let query = self.search_query.trim().to_string();

        if self.is_collapsed {
            return div()
                .id("main-sidebar-collapsed")
                .w(px(48.))
                .h_full()
                .bg(theme.background)
                .border_r_1()
                .border_color(theme.border)
                .flex()
                .flex_col()
                .items_center()
                .justify_between()
                .py_2p5()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .id("sidebar-expand-rail-btn")
                                .w(px(28.))
                                .h(px(28.))
                                .rounded(px(5.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                                .text_color(theme.muted_foreground)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toggle_collapsed(cx);
                                }))
                                .child(Icon::new(IconName::PanelLeft).size(px(16.))),
                        )
                        .child(
                            div()
                                .w(px(24.))
                                .h(px(1.))
                                .bg(theme.border),
                        )
                        .child(
                            div()
                                .id("sidebar-new-note-rail-btn")
                                .w(px(28.))
                                .h(px(28.))
                                .rounded(px(5.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                                .text_color(theme.muted_foreground)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.create_new_note(cx);
                                }))
                                .child(Icon::new(IconName::Plus).size(px(16.))),
                        ),
                )
                .child(
                    div()
                        .id("sidebar-settings-rail-btn")
                        .w(px(28.))
                        .h(px(28.))
                        .rounded(px(5.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(IconName::Settings).size(px(16.))),
                )
                .into_any_element();
        }

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
                        div()
                            .id("sidebar-collapse-btn")
                            .w(px(24.))
                            .h(px(24.))
                            .rounded(px(4.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                            .text_color(theme.muted_foreground)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_collapsed(cx);
                            }))
                            .child(Icon::new(IconName::PanelLeft).size(px(14.))),
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
                    .value(self.search_query.clone())
                    .focus_handle(self.search_focus.clone())
                    .on_key_down(cx.listener(|this, event, window, cx| {
                        this.handle_search_key(event, window, cx);
                    }))
                    .on_clear(cx.listener(|this, _, _, cx| {
                        this.search_query.clear();
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
                            })),
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
                    .selected(self.selected_section == "starred")
                    .count(starred_count)
                    .on_toggle(cx.listener(|this, _, _, cx| {
                        this.toggle_starred(cx);
                    }))
                    .on_select(cx.listener(|this, _, _, cx| {
                        this.toggle_starred(cx);
                    })),
            );

            if self.starred_expanded {
                for note in starred_notes {
                    let note_id = note.id.clone();
                    let is_active = selected_note.as_deref() == Some(&note_id);
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
                            })),
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
                    .selected(self.selected_section == "folder:projects")
                    .count(4)
                    .on_toggle(cx.listener(|this, _, _, cx| {
                        this.toggle_folder("projects", cx);
                    }))
                    .on_select(cx.listener(|this, _, _, cx| {
                        this.select_section("folder:projects", cx);
                    })),
            );

            if projects_expanded {
                folders_group = folders_group.child(
                    FolderTreeItem::new("folder-architecture", "Architecture")
                        .depth(1)
                        .icon(IconName::Code)
                        .expanded(arch_expanded)
                        .selected(self.selected_section == "folder:architecture")
                        .count(2)
                        .on_toggle(cx.listener(|this, _, _, cx| {
                            this.toggle_folder("projects:architecture", cx);
                        }))
                        .on_select(cx.listener(|this, _, _, cx| {
                            this.select_section("folder:architecture", cx);
                        })),
                );

                if arch_expanded {
                    folders_group = folders_group.child(
                        FolderTreeItem::new("folder-arch-core", "Core Engine")
                            .depth(2)
                            .icon(IconName::Zap)
                            .expanded(core_expanded)
                            .selected(self.selected_section == "folder:arch-core")
                            .count(1)
                            .on_toggle(cx.listener(|this, _, _, cx| {
                                this.toggle_folder("projects:architecture:core", cx);
                            }))
                            .on_select(cx.listener(|this, _, _, cx| {
                                this.select_section("folder:arch-core", cx);
                            })),
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
                                })),
                        );
                    }

                    folders_group = folders_group.child(
                        FolderTreeItem::new("folder-arch-desktop", "Desktop Shell")
                            .depth(2)
                            .icon(IconName::Rocket)
                            .expanded(desktop_expanded)
                            .selected(self.selected_section == "folder:arch-desktop")
                            .count(1)
                            .on_toggle(cx.listener(|this, _, _, cx| {
                                this.toggle_folder("projects:architecture:desktop", cx);
                            }))
                            .on_select(cx.listener(|this, _, _, cx| {
                                this.select_section("folder:arch-desktop", cx);
                            })),
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
                                })),
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
                        })),
                );

                let is_ws_active = selected_note.as_deref() == Some("note-ws-sync");
                folders_group = folders_group.child(
                    NoteTreeItem::new("tree-note-ws-sync", "WebSocket Sync Protocol")
                        .depth(1)
                        .active(is_ws_active)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.select_note("note-ws-sync", cx);
                        })),
                );

                folders_group = folders_group.child(
                    FolderTreeItem::new("folder-specs", "Specifications")
                        .depth(1)
                        .icon(IconName::Briefcase)
                        .selected(self.selected_section == "folder:specs")
                        .count(0)
                        .on_select(cx.listener(|this, _, _, cx| {
                            this.select_section("folder:specs", cx);
                        })),
                );
            }

            folders_group = folders_group.child(
                FolderTreeItem::new("folder-personal", "Personal Notes")
                    .depth(0)
                    .expanded(personal_expanded)
                    .selected(self.selected_section == "folder:personal")
                    .count(3)
                    .on_toggle(cx.listener(|this, _, _, cx| {
                        this.toggle_folder("personal", cx);
                    }))
                    .on_select(cx.listener(|this, _, _, cx| {
                        this.select_section("folder:personal", cx);
                    })),
            );

            if personal_expanded {
                folders_group = folders_group.child(
                    FolderTreeItem::new("folder-journal", "Journal")
                        .depth(1)
                        .icon(IconName::Bookmark)
                        .expanded(journal_expanded)
                        .selected(self.selected_section == "folder:journal")
                        .count(2)
                        .on_toggle(cx.listener(|this, _, _, cx| {
                            this.toggle_folder("personal:journal", cx);
                        }))
                        .on_select(cx.listener(|this, _, _, cx| {
                            this.select_section("folder:journal", cx);
                        })),
                );

                if journal_expanded {
                    let is_goals_active = selected_note.as_deref() == Some("note-sprint-goals");
                    folders_group = folders_group.child(
                        NoteTreeItem::new("tree-note-sprint-goals", "Weekly Sprint Goals")
                            .depth(2)
                            .active(is_goals_active)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_note("note-sprint-goals", cx);
                            })),
                    );

                    let is_inspo_active = selected_note.as_deref() == Some("note-design-inspo");
                    folders_group = folders_group.child(
                        NoteTreeItem::new("tree-note-design-inspo", "Design Inspiration: Obsidian x Notion")
                            .depth(2)
                            .active(is_inspo_active)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_note("note-design-inspo", cx);
                            })),
                    );
                }

                let is_roadmap_active = selected_note.as_deref() == Some("note-q3-roadmap");
                folders_group = folders_group.child(
                    NoteTreeItem::new("tree-note-q3-roadmap", "Q3 Roadmap & Planning")
                        .depth(1)
                        .active(is_roadmap_active)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.select_note("note-q3-roadmap", cx);
                        })),
                );
            }

            for note in self.notes.iter().filter(|n| n.folder_id.is_none()) {
                let note_id = note.id.clone();
                let is_active = selected_note.as_deref() == Some(&note_id);
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
                        })),
                );
            }

            content = content.child(folders_group);
        }

        let footer = SidebarFooter::new().child(
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
                                .w(px(24.))
                                .h(px(24.))
                                .rounded_full()
                                .bg(theme.secondary)
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
                        .id("sidebar-footer-settings-btn")
                        .w(px(24.))
                        .h(px(24.))
                        .rounded(px(4.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(IconName::Settings).size(px(14.))),
                ),
        );

        Sidebar::new("main-sidebar")
            .width(px(260.))
            .header(header)
            .content(content)
            .footer(footer)
            .into_any_element()
    }
}
