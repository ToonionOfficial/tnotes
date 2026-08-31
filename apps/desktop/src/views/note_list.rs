#![allow(dead_code)]

use gpui::*;

pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub preview: String,
    pub folder: String,
    pub updated_at: String,
}

pub struct NoteListView {
    pub selected_note_id: Option<String>,
    pub notes: Vec<NoteSummary>,
}

impl NoteListView {
    pub fn new() -> Self {
        Self {
            selected_note_id: Some("1".to_string()),
            notes: vec![
                NoteSummary {
                    id: "1".to_string(),
                    title: "Project Roadmap & AST Design".to_string(),
                    preview: "The document model is editor-independent and maps cleanly to TipTap...".to_string(),
                    folder: "Work".to_string(),
                    updated_at: "2m ago".to_string(),
                },
                NoteSummary {
                    id: "2".to_string(),
                    title: "Shopping & Errands".to_string(),
                    preview: "Buy coffee beans, organic milk, and check mailbox...".to_string(),
                    folder: "Personal".to_string(),
                    updated_at: "1h ago".to_string(),
                },
                NoteSummary {
                    id: "3".to_string(),
                    title: "Rust GPUI Performance Notes".to_string(),
                    preview: "Blade graphics backend gives smooth 120 FPS rendering on Linux and macOS...".to_string(),
                    folder: "Research".to_string(),
                    updated_at: "Yesterday".to_string(),
                },
            ],
        }
    }
}

impl Default for NoteListView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for NoteListView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let selected_id = self.selected_note_id.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x18181c))
            .border_r_1()
            .border_color(rgb(0x27272a))
            .child(
                // Search & Filter header
                div()
                    .p_3()
                    .border_b_1()
                    .border_color(rgb(0x27272a))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(0x222228))
                            .border_1()
                            .border_color(rgb(0x2d2d34))
                            .child(div().text_xs().text_color(rgb(0x71717a)).child("🔍"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x71717a))
                                    .child("Search notes..."),
                            ),
                    ),
            )
            .child(
                // Note items list
                div()
                    .id("notes-list-scroll")
                    .flex()
                    .flex_col()
                    .p_2()
                    .gap_1p5()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.notes.iter().map(|note| {
                        let is_selected = selected_id.as_deref() == Some(&note.id);
                        let bg_color = if is_selected {
                            rgb(0x24242c)
                        } else {
                            rgb(0x1c1c22)
                        };
                        let border_color = if is_selected {
                            rgb(0x6366f1)
                        } else {
                            rgb(0x272730)
                        };

                        div()
                            .flex()
                            .flex_col()
                            .p_3()
                            .rounded_lg()
                            .bg(bg_color)
                            .border_1()
                            .border_color(border_color)
                            .gap_1()
                            .hover(|s| s.bg(rgb(0x22222a)))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xf4f4f5))
                                            .child(note.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x71717a))
                                            .child(note.updated_at.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xa1a1aa))
                                    .line_clamp(2)
                                    .child(note.preview.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .pt_1()
                                    .child(
                                        div()
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded_sm()
                                            .bg(rgb(0x2a2a34))
                                            .text_xs()
                                            .text_color(rgb(0xa1a1aa))
                                            .child(format!("📁 {}", note.folder)),
                                    ),
                            )
                    })),
            )
    }
}
