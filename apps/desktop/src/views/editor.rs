use crate::state::NoteStore;
use gpui::*;
use tnotes_document::{Block, Document, ListItem, RichText, SubList, TaskItem};

pub struct EditorView {
    pub note_store: Entity<NoteStore>,
}

impl EditorView {
    pub fn new(note_store: Entity<NoteStore>, cx: &mut Context<Self>) -> Self {
        cx.observe(&note_store, |_this, _note_store, cx| {
            cx.notify();
        })
        .detach();

        Self { note_store }
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let note_store = self.note_store.read(cx);
        let selected_note = note_store.selected_note();

        let (title, html_body) = if let Some(note) = selected_note {
            (note.title.clone(), note.body.clone())
        } else {
            (
                "No Note Selected".to_string(),
                "<p>Select or create a note from the sidebar.</p>".to_string(),
            )
        };

        let document = Document::from_html(&html_body).unwrap_or(Document { blocks: vec![] });

        div()
            .id("editor-view-container")
            .flex()
            .flex_col()
            .items_center()
            .size_full()
            .bg(rgb(0x141318))
            .p_8()
            .overflow_y_scroll()
            .child(
                div()
                    .w_full()
                    .max_w(px(720.0))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xe6e1e9))
                            .child(title),
                    )
                    .child(
                        div().h_px().bg(rgb(0x302e36)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .children(document.blocks.iter().map(render_block)),
                    ),
            )
    }
}

fn render_block(block: &Block) -> AnyElement {
    match block {
        Block::Paragraph(rt) => render_rich_text(rt, 16.0, rgb(0xe6e1e9)).into_any_element(),
        Block::Heading { level, content } => {
            let (size, weight, color) = match level {
                1 => (24.0, FontWeight::BOLD, rgb(0xe6e1e9)),
                2 => (20.0, FontWeight::BOLD, rgb(0xe6e1e9)),
                3 => (18.0, FontWeight::SEMIBOLD, rgb(0xe6e1e9)),
                _ => (16.0, FontWeight::SEMIBOLD, rgb(0xe6e1e9)),
            };
            div()
                .text_size(px(size))
                .font_weight(weight)
                .text_color(color)
                .child(render_rich_text(content, size, color))
                .into_any_element()
        }
        Block::Quote(rt) => div()
            .border_l_4()
            .border_color(rgb(0xcabeff))
            .bg(rgb(0x201f24))
            .rounded_r_md()
            .px_4()
            .py_2()
            .text_color(rgb(0x938f99))
            .child(render_rich_text(rt, 15.0, rgb(0xcabeff)))
            .into_any_element(),
        Block::CodeBlock { language, code } => div()
            .bg(rgb(0x201f24))
            .border_1()
            .border_color(rgb(0x302e36))
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0x79747e))
                    .child(language.clone().unwrap_or_else(|| "code".to_string())),
            )
            .child(
                div()
                    .font_family(".SystemUIFont")
                    .text_sm()
                    .text_color(rgb(0xa6e3a1))
                    .child(code.clone()),
            )
            .into_any_element(),
        Block::BulletList(items) => div()
            .flex()
            .flex_col()
            .gap_1p5()
            .pl_4()
            .children(items.iter().map(render_bullet_item))
            .into_any_element(),
        Block::OrderedList(items) => div()
            .flex()
            .flex_col()
            .gap_1p5()
            .pl_4()
            .children(items.iter().enumerate().map(|(idx, item)| {
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(
                                div()
                                    .text_color(rgb(0xcabeff))
                                    .child(format!("{}.", idx + 1)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(render_rich_text(&item.content, 16.0, rgb(0xe6e1e9))),
                            ),
                    )
                    .children(item.sub_list.as_ref().map(|sub| render_sub_list(sub)))
            }))
            .into_any_element(),
        Block::TaskList(items) => div()
            .flex()
            .flex_col()
            .gap_2()
            .children(items.iter().map(render_task_item))
            .into_any_element(),
        Block::Divider => div().my_2().h_px().bg(rgb(0x302e36)).into_any_element(),
    }
}

fn render_bullet_item(item: &ListItem) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(div().text_color(rgb(0xcabeff)).child("•"))
                .child(
                    div()
                        .flex_1()
                        .child(render_rich_text(&item.content, 16.0, rgb(0xe6e1e9))),
                ),
        )
        .children(item.sub_list.as_ref().map(|sub| render_sub_list(sub)))
}

fn render_sub_list(sub: &SubList) -> impl IntoElement {
    match sub {
        SubList::Bullet(items) => div()
            .pl_4()
            .flex()
            .flex_col()
            .gap_1p5()
            .children(items.iter().map(render_bullet_item))
            .into_any_element(),
        SubList::Ordered(items) => div()
            .pl_4()
            .flex()
            .flex_col()
            .gap_1p5()
            .children(items.iter().enumerate().map(|(idx, item)| {
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(
                        div()
                            .text_color(rgb(0xcabeff))
                            .child(format!("{}.", idx + 1)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(render_rich_text(&item.content, 16.0, rgb(0xe6e1e9))),
                    )
            }))
            .into_any_element(),
    }
}

fn render_task_item(item: &TaskItem) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap_2p5()
        .child(
            div()
                .w_4()
                .h_4()
                .mt_0p5()
                .rounded_sm()
                .border_1()
                .border_color(if item.checked {
                    rgb(0xa6e3a1)
                } else {
                    rgb(0x79747e)
                })
                .bg(if item.checked {
                    rgb(0x2d4f3e)
                } else {
                    rgb(0x201f24)
                })
                .flex()
                .items_center()
                .justify_center()
                .child(if item.checked {
                    div().text_xs().text_color(rgb(0xa6e3a1)).child("✓")
                } else {
                    div().child("")
                }),
        )
        .child(
            div()
                .flex_1()
                .child(render_rich_text(&item.content, 16.0, rgb(0xe6e1e9))),
        )
}

fn render_rich_text(rt: &RichText, base_size: f32, default_color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_baseline()
        .text_size(px(base_size))
        .children(rt.spans.iter().map(|span| {
            let mut el = div().child(span.text.clone());

            if span.marks.bold {
                el = el.font_weight(FontWeight::BOLD);
            }
            if span.marks.italic {
                el = el.italic();
            }
            if span.marks.code {
                el = el
                    .bg(rgb(0x201f24))
                    .px_1()
                    .rounded_sm()
                    .text_color(rgb(0xa6e3a1));
            } else if span.link.is_some() {
                el = el
                    .text_color(rgb(0xcabeff))
                    .cursor_pointer()
                    .hover(|s| s.underline());
            } else {
                el = el.text_color(default_color);
            }

            el
        }))
}
