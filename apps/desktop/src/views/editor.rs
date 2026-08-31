use gpui::*;
use tnotes_document::{Block, Document, RichText};

pub struct EditorView {
    pub title: String,
    pub document: Document,
}

impl EditorView {
    pub fn new(title: String, html_content: &str) -> Self {
        let document = Document::from_html(html_content).unwrap_or(Document { blocks: vec![] });
        Self { title, document }
    }

    pub fn set_note(&mut self, title: String, html_content: &str) {
        self.title = title;
        self.document = Document::from_html(html_content).unwrap_or(Document { blocks: vec![] });
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("editor-view-container")
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x141318)) // Match background
            .p_8()
            .overflow_y_scroll()
            .gap_4()
            .child(
                // Note Title Header
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xe6e1e9)) // Foreground
                    .child(self.title.clone()),
            )
            .child(
                // Divider
                div().h_px().bg(rgb(0x302e36)),
            )
            .child(
                // Rendered Document AST blocks
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(self.document.blocks.iter().map(render_block)),
            )
    }
}

fn render_block(block: &Block) -> AnyElement {
    match block {
        Block::Paragraph(rt) => render_rich_text(rt, 16.0, rgb(0xe6e1e9)).into_any_element(),
        Block::Heading { level, content } => {
            let (size, weight, color) = match level {
                1 => (24.0, FontWeight::BOLD, rgb(0xe6e1e9)),
                2 => (20.0, FontWeight::SEMIBOLD, rgb(0xe6e1e9)),
                _ => (17.0, FontWeight::SEMIBOLD, rgb(0xe6e1e9)),
            };
            div()
                .font_weight(weight)
                .child(render_rich_text(content, size, color))
                .into_any_element()
        }
        Block::Quote(rt) => div()
            .pl_3()
            .border_l_2()
            .border_color(rgb(0xcabeff)) // Primary accent
            .child(render_rich_text(rt, 15.0, rgb(0xc6c2cd)))
            .into_any_element(),
        Block::CodeBlock { language, code } => div()
            .p_3()
            .rounded_xl()
            .bg(rgb(0x201f24)) // Card bg
            .border_1()
            .border_color(rgb(0x302e36))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pb_1()
                    .text_xs()
                    .text_color(rgb(0x938f99))
                    .child(language.clone().unwrap_or_else(|| "code".to_string())),
            )
            .child(
                div()
                    .font_family("monospace")
                    .text_xs()
                    .text_color(rgb(0xcabeff))
                    .child(code.clone()),
            )
            .into_any_element(),
        Block::Divider => div().h_px().bg(rgb(0x302e36)).my_2().into_any_element(),
        Block::BulletList(items) => div()
            .flex()
            .flex_col()
            .gap_1p5()
            .children(items.iter().map(|item| {
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(div().text_sm().text_color(rgb(0xcabeff)).child("•"))
                    .child(render_rich_text(&item.content, 15.0, rgb(0xe6e1e9)))
            }))
            .into_any_element(),
        Block::OrderedList(items) => div()
            .flex()
            .flex_col()
            .gap_1p5()
            .children(items.iter().enumerate().map(|(idx, item)| {
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x938f99))
                            .child(format!("{}.", idx + 1)),
                    )
                    .child(render_rich_text(&item.content, 15.0, rgb(0xe6e1e9)))
            }))
            .into_any_element(),
        Block::TaskList(items) => div()
            .flex()
            .flex_col()
            .gap_2()
            .children(items.iter().map(|item| {
                let checkbox_bg = if item.checked {
                    rgb(0xcabeff) // Primary purple
                } else {
                    rgb(0x201f24) // Card
                };
                let check_mark = if item.checked { "✓" } else { "" };
                let text_color = if item.checked {
                    rgb(0x938f99) // Muted strike
                } else {
                    rgb(0xe6e1e9)
                };

                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        div()
                            .w_4()
                            .h_4()
                            .rounded_sm()
                            .bg(checkbox_bg)
                            .border_1()
                            .border_color(rgb(0xcabeff))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x32285f))
                            .child(check_mark),
                    )
                    .child(render_rich_text(&item.content, 15.0, text_color))
            }))
            .into_any_element(),
    }
}

fn render_rich_text(rt: &RichText, _font_size: f32, default_color: Rgba) -> impl IntoElement {
    div().flex().flex_wrap().gap_0p5().children(rt.spans.iter().map(|span| {
        let color = if span.link.is_some() || span.marks.code {
            rgb(0xcabeff)
        } else {
            default_color
        };

        let weight = if span.marks.bold {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };

        let mut el = div()
            .text_sm()
            .font_weight(weight)
            .text_color(color)
            .child(span.text.clone());

        if span.marks.code {
            el = el.px_1().rounded_sm().bg(rgb(0x201f24));
        }

        el
    }))
}
