use gpui::*;
use crate::components::{Badge, Icon, IconName, NoteCard};
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub updated_at: String,
    pub folder_id: Option<String>,
    pub folder_name: Option<String>,
    pub is_pinned: bool,
}

#[allow(dead_code)]
pub struct NoteListView {
    notes: Vec<NoteSummary>,
    selected_note_id: Option<String>,
    search_query: String,
    active_section: String,
    active_title: String,
}

#[allow(dead_code)]
impl NoteListView {
    pub fn new() -> Self {
        let notes = vec![
            NoteSummary {
                id: "note-arch-spec".to_string(),
                title: "System Architecture Spec".to_string(),
                snippet: "Local-first SQLite with FTS5, CRDT synchronization, and DankeShell token design system.".to_string(),
                updated_at: "Just now".to_string(),
                folder_id: Some("projects:architecture:core".to_string()),
                folder_name: Some("Core Engine".to_string()),
                is_pinned: true,
            },
            NoteSummary {
                id: "note-desktop-gpui".to_string(),
                title: "Desktop Shell & GPUI Architecture".to_string(),
                snippet: "Three-pane Obsidian-style layout, borderless note cards, twrite rope canvas rendering.".to_string(),
                updated_at: "15m ago".to_string(),
                folder_id: Some("projects:architecture:desktop".to_string()),
                folder_name: Some("Desktop Shell".to_string()),
                is_pinned: true,
            },
            NoteSummary {
                id: "note-db-schema".to_string(),
                title: "Database Schema & Index Design".to_string(),
                snippet: "Tables for notes, folders, tags, sync operations log, and tokenized full-text search indexes.".to_string(),
                updated_at: "2h ago".to_string(),
                folder_id: Some("projects".to_string()),
                folder_name: Some("Projects".to_string()),
                is_pinned: false,
            },
            NoteSummary {
                id: "note-ws-sync".to_string(),
                title: "WebSocket Sync Protocol".to_string(),
                snippet: "Bidirectional binary and JSON delta streaming between mobile client and desktop server.".to_string(),
                updated_at: "Yesterday".to_string(),
                folder_id: Some("projects".to_string()),
                folder_name: Some("Projects".to_string()),
                is_pinned: false,
            },
            NoteSummary {
                id: "note-q3-roadmap".to_string(),
                title: "Q3 Roadmap & Planning".to_string(),
                snippet: "Offline-first conflict resolution, canvas mode, graph view, and end-to-end encryption.".to_string(),
                updated_at: "Sep 12".to_string(),
                folder_id: Some("personal".to_string()),
                folder_name: Some("Personal".to_string()),
                is_pinned: false,
            },
            NoteSummary {
                id: "note-sprint-goals".to_string(),
                title: "Weekly Sprint Goals".to_string(),
                snippet: "Implement NoteList explorer pane, DankeShell card states, and twrite editor integration.".to_string(),
                updated_at: "Sep 10".to_string(),
                folder_id: Some("personal".to_string()),
                folder_name: Some("Personal".to_string()),
                is_pinned: false,
            },
            NoteSummary {
                id: "note-design-inspo".to_string(),
                title: "Design Inspiration: Obsidian x Notion".to_string(),
                snippet: "Collapsible sidebar rail, clean hierarchy, distraction-free markdown canvas, quiet chrome.".to_string(),
                updated_at: "Sep 8".to_string(),
                folder_id: Some("personal".to_string()),
                folder_name: Some("Personal".to_string()),
                is_pinned: false,
            },
        ];

        let selected_note_id = Some("note-arch-spec".to_string());

        Self {
            notes,
            selected_note_id,
            search_query: String::new(),
            active_section: "all_notes".to_string(),
            active_title: "All Notes".to_string(),
        }
    }

    pub fn set_filter(&mut self, section_id: String, section_title: String, cx: &mut Context<Self>) {
        self.active_section = section_id;
        self.active_title = section_title;
        cx.notify();
    }

    pub fn set_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query;
        cx.notify();
    }

    pub fn select_note(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.selected_note_id = id;
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn selected_note_id(&self) -> Option<&str> {
        self.selected_note_id.as_deref()
    }

    pub fn selected_note(&self) -> Option<&NoteSummary> {
        let id = self.selected_note_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == id)
    }

    pub fn filtered_notes(&self) -> Vec<&NoteSummary> {
        let query = self.search_query.trim().to_lowercase();

        self.notes
            .iter()
            .filter(|note| {
                let matches_section = match self.active_section.as_str() {
                    "all_notes" => true,
                    "favorites" => note.is_pinned,
                    "trash" => false,
                    folder => {
                        if let Some(ref note_folder) = note.folder_id {
                            note_folder == folder || note_folder.starts_with(&format!("{folder}:"))
                        } else {
                            false
                        }
                    }
                };

                if !matches_section {
                    return false;
                }

                if query.is_empty() {
                    return true;
                }

                note.title.to_lowercase().contains(&query)
                    || note.snippet.to_lowercase().contains(&query)
                    || note
                        .folder_name
                        .as_ref()
                        .map(|f| f.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect()
    }
}

impl Render for NoteListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let filtered = self.filtered_notes();
        let total_count = filtered.len();

        let (pinned, regular): (Vec<&NoteSummary>, Vec<&NoteSummary>) =
            filtered.into_iter().partition(|n| n.is_pinned);

        let header = div()
            .w_full()
            .h(px(48.))
            .px(px(14.))
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground)
                            .child(self.active_title.clone()),
                    )
                    .child(Badge::secondary(format!("{total_count}"))),
            );

        let mut list = div()
            .id("note-list-scroll")
            .flex_1()
            .overflow_y_scroll()
            .px(px(8.))
            .py(px(8.))
            .flex()
            .flex_col()
            .gap(px(4.));

        if total_count == 0 {
            list = list.child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .py(px(40.))
                    .child(
                        Icon::new(IconName::FileText)
                            .size(px(28.))
                            .color(theme.muted_foreground),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground)
                            .child("No notes found"),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.muted_foreground)
                            .child(if !self.search_query.is_empty() {
                                "Try a different search query"
                            } else {
                                "No notes in this section"
                            }),
                    ),
            );
        } else {
            if !pinned.is_empty() {
                list = list.child(
                    div()
                        .px(px(6.))
                        .pt(px(4.))
                        .pb(px(2.))
                        .text_size(px(10.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.muted_foreground)
                        .child("PINNED"),
                );

                for note in &pinned {
                    let note_id = note.id.clone();
                    let is_active = self.selected_note_id.as_deref() == Some(&note_id);

                    let mut card = NoteCard::new(format!("card-{}", note.id), note.title.clone())
                        .snippet(note.snippet.clone())
                        .updated_at(note.updated_at.clone())
                        .is_pinned(true)
                        .is_active(is_active)
                        .on_select(cx.listener({
                            let note_id = note_id.clone();
                            move |this, _, _, cx| {
                                this.select_note(Some(note_id.clone()), cx);
                            }
                        }));

                    if let Some(ref folder) = note.folder_name {
                        card = card.folder_name(folder.clone());
                    }

                    list = list.child(card);
                }
            }

            if !regular.is_empty() {
                if !pinned.is_empty() {
                    list = list.child(
                        div()
                            .px(px(6.))
                            .pt(px(8.))
                            .pb(px(2.))
                            .text_size(px(10.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.muted_foreground)
                            .child("NOTES"),
                    );
                }

                for note in &regular {
                    let note_id = note.id.clone();
                    let is_active = self.selected_note_id.as_deref() == Some(&note_id);

                    let mut card = NoteCard::new(format!("card-{}", note.id), note.title.clone())
                        .snippet(note.snippet.clone())
                        .updated_at(note.updated_at.clone())
                        .is_pinned(false)
                        .is_active(is_active)
                        .on_select(cx.listener({
                            let note_id = note_id.clone();
                            move |this, _, _, cx| {
                                this.select_note(Some(note_id.clone()), cx);
                            }
                        }));

                    if let Some(ref folder) = note.folder_name {
                        card = card.folder_name(folder.clone());
                    }

                    list = list.child(card);
                }
            }
        }

        div()
            .w(px(280.))
            .h_full()
            .flex()
            .flex_col()
            .bg(theme.card)
            .border_r_1()
            .border_color(theme.border)
            .child(header)
            .child(list)
    }
}
