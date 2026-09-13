use std::fmt::Write;

use crate::model::{Block, BlockKind, Document, ListItem, RichText, Span, SubList, TableData, TaskItem};

pub struct Serializer;

impl Serializer {
    pub fn serialize(doc: &Document) -> String {
        let mut out = String::new();
        for block in &doc.blocks {
            Self::serialize_block(block, &mut out);
        }
        out
    }

    fn serialize_block(block: &Block, out: &mut String) {
        match &block.kind {
            BlockKind::Paragraph(rt) => {
                out.push_str("<p>");
                Self::serialize_rich_text(rt, out);
                out.push_str("</p>");
            }
            BlockKind::Heading { level, content } => {
                let _ = write!(out, "<h{level}>");
                Self::serialize_rich_text(content, out);
                let _ = write!(out, "</h{level}>");
            }
            BlockKind::Quote(rt) => {
                out.push_str("<blockquote><p>");
                Self::serialize_rich_text(rt, out);
                out.push_str("</p></blockquote>");
            }
            BlockKind::CodeBlock { language, code } => {
                out.push_str("<pre>");
                if let Some(lang) = language {
                    let _ = write!(out, "<code class=\"language-{lang}\">");
                } else {
                    out.push_str("<code>");
                }
                Self::escape_html(code, out);
                out.push_str("</code></pre>");
            }
            BlockKind::Divider => {
                out.push_str("<hr>");
            }
            BlockKind::BulletList(items) => {
                out.push_str("<ul>");
                for item in items {
                    Self::serialize_list_item(item, out);
                }
                out.push_str("</ul>");
            }
            BlockKind::OrderedList(items) => {
                out.push_str("<ol>");
                for item in items {
                    Self::serialize_list_item(item, out);
                }
                out.push_str("</ol>");
            }
            BlockKind::TaskList(items) => {
                out.push_str("<ul data-type=\"taskList\">");
                for item in items {
                    Self::serialize_task_item(item, out);
                }
                out.push_str("</ul>");
            }
            BlockKind::Table(table) => {
                Self::serialize_table(table, out);
            }
            BlockKind::Image(img) => {
                out.push_str("<img src=\"");
                Self::escape_html(&img.asset_id, out);
                out.push_str("\"");
                if let Some(alt) = &img.alt {
                    out.push_str(" alt=\"");
                    Self::escape_html(alt, out);
                    out.push_str("\"");
                }
                out.push_str(" />");
            }
            BlockKind::Drawing(_) | BlockKind::Audio(_) => {}
        }
    }

    fn serialize_table(table: &TableData, out: &mut String) {
        out.push_str("<table>");
        if !table.headers.is_empty() {
            out.push_str("<thead><tr>");
            for h in &table.headers {
                out.push_str("<th>");
                Self::escape_html(h, out);
                out.push_str("</th>");
            }
            out.push_str("</tr></thead>");
        }
        if !table.rows.is_empty() {
            out.push_str("<tbody>");
            for row in &table.rows {
                out.push_str("<tr>");
                for cell in row {
                    out.push_str("<td>");
                    Self::escape_html(cell, out);
                    out.push_str("</td>");
                }
                out.push_str("</tr>");
            }
            out.push_str("</tbody>");
        }
        out.push_str("</table>");
    }

    fn serialize_list_item(item: &ListItem, out: &mut String) {
        out.push_str("<li><p>");
        Self::serialize_rich_text(&item.content, out);
        out.push_str("</p>");

        if let Some(sub_list) = &item.sub_list {
            match &**sub_list {
                SubList::Bullet(sub_items) => {
                    out.push_str("<ul>");
                    for sub in sub_items {
                        Self::serialize_list_item(sub, out);
                    }
                    out.push_str("</ul>");
                }
                SubList::Ordered(sub_items) => {
                    out.push_str("<ol>");
                    for sub in sub_items {
                        Self::serialize_list_item(sub, out);
                    }
                    out.push_str("</ol>");
                }
            }
        }

        out.push_str("</li>");
    }

    fn serialize_task_item(item: &TaskItem, out: &mut String) {
        let (checked_attr, input_checked) = if item.checked {
            ("true", " checked=\"checked\"")
        } else {
            ("false", "")
        };
        let _ = write!(
            out,
            "<li data-type=\"taskItem\" data-checked=\"{checked_attr}\"><label><input type=\"checkbox\"{input_checked}><span></span></label><div><p>"
        );
        Self::serialize_rich_text(&item.content, out);
        out.push_str("</p></div></li>");
    }

    fn serialize_rich_text(rich_text: &RichText, out: &mut String) {
        for span in &rich_text.spans {
            Self::serialize_span(span, out);
        }
    }

    fn serialize_span(span: &Span, out: &mut String) {
        let marks = span.marks;

        if let Some(link) = &span.link {
            let _ = write!(out, "<a href=\"{link}\">");
        }
        if marks.bold {
            out.push_str("<strong>");
        }
        if marks.italic {
            out.push_str("<em>");
        }
        if marks.underline {
            out.push_str("<u>");
        }
        if marks.strike {
            out.push_str("<s>");
        }
        if marks.code {
            out.push_str("<code>");
        }

        Self::escape_inline_text(&span.text, out);

        if marks.code {
            out.push_str("</code>");
        }
        if marks.strike {
            out.push_str("</s>");
        }
        if marks.underline {
            out.push_str("</u>");
        }
        if marks.italic {
            out.push_str("</em>");
        }
        if marks.bold {
            out.push_str("</strong>");
        }
        if span.link.is_some() {
            out.push_str("</a>");
        }
    }

    fn escape_inline_text(text: &str, out: &mut String) {
        let mut parts = text.split('\n');
        if let Some(first) = parts.next() {
            Self::escape_html(first, out);
        }
        for part in parts {
            out.push_str("<br>");
            Self::escape_html(part, out);
        }
    }

    fn escape_html(text: &str, out: &mut String) {
        for c in text.chars() {
            match c {
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '&' => out.push_str("&amp;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&#39;"),
                _ => out.push(c),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Marks, SubList};
    use crate::parser::Parser;

    #[test]
    fn test_serialize_empty_document() {
        let doc = Document {
            version: 1,
            blocks: vec![],
        };
        assert_eq!(Serializer::serialize(&doc), "");
    }

    #[test]
    fn test_serialize_paragraph_plain() {
        let doc = Document::new(vec![Block::new(BlockKind::Paragraph(RichText {
            spans: vec![Span {
                text: "Hello world".to_string(),
                marks: Marks::default(),
                link: None,
            }],
        }))]);
        assert_eq!(Serializer::serialize(&doc), "<p>Hello world</p>");
    }

    #[test]
    fn test_serialize_formatting_and_links() {
        let doc = Document::new(vec![Block::new(BlockKind::Paragraph(RichText {
            spans: vec![
                Span {
                    text: "Bold".to_string(),
                    marks: Marks {
                        bold: true,
                        ..Default::default()
                    },
                    link: None,
                },
                Span {
                    text: " and ".to_string(),
                    marks: Marks::default(),
                    link: None,
                },
                Span {
                    text: "link".to_string(),
                    marks: Marks {
                        italic: true,
                        ..Default::default()
                    },
                    link: Some("https://tnotes.app".to_string()),
                },
            ],
        }))]);
        assert_eq!(
            Serializer::serialize(&doc),
            "<p><strong>Bold</strong> and <a href=\"https://tnotes.app\"><em>link</em></a></p>"
        );
    }

    #[test]
    fn test_serialize_headings() {
        let doc = Document::new(vec![
            Block::new(BlockKind::Heading {
                level: 1,
                content: RichText {
                    spans: vec![Span {
                        text: "Title".to_string(),
                        marks: Marks::default(),
                        link: None,
                    }],
                },
            }),
            Block::new(BlockKind::Heading {
                level: 2,
                content: RichText {
                    spans: vec![Span {
                        text: "Heading".to_string(),
                        marks: Marks::default(),
                        link: None,
                    }],
                },
            }),
        ]);
        assert_eq!(
            Serializer::serialize(&doc),
            "<h1>Title</h1><h2>Heading</h2>"
        );
    }

    #[test]
    fn test_serialize_code_block() {
        let doc = Document::new(vec![
            Block::new(BlockKind::CodeBlock {
                language: Some("rust".to_string()),
                code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
            }),
            Block::new(BlockKind::CodeBlock {
                language: None,
                code: "plain code".to_string(),
            }),
        ]);
        assert_eq!(
            Serializer::serialize(&doc),
            "<pre><code class=\"language-rust\">fn main() {\n    println!(&quot;Hello&quot;);\n}</code></pre><pre><code>plain code</code></pre>"
        );
    }

    #[test]
    fn test_serialize_divider() {
        let doc = Document::new(vec![Block::new(BlockKind::Divider)]);
        assert_eq!(Serializer::serialize(&doc), "<hr>");
    }

    #[test]
    fn test_serialize_lists() {
        let doc = Document::new(vec![
            Block::new(BlockKind::BulletList(vec![
                ListItem {
                    content: RichText {
                        spans: vec![Span {
                            text: "Bullet 1".to_string(),
                            marks: Marks::default(),
                            link: None,
                        }],
                    },
                    sub_list: Some(Box::new(SubList::Ordered(vec![ListItem {
                        content: RichText {
                            spans: vec![Span {
                                text: "Sub ordered 1".to_string(),
                                marks: Marks::default(),
                                link: None,
                            }],
                        },
                        sub_list: None,
                    }]))),
                },
                ListItem {
                    content: RichText {
                        spans: vec![Span {
                            text: "Bullet 2".to_string(),
                            marks: Marks::default(),
                            link: None,
                        }],
                    },
                    sub_list: None,
                },
            ])),
            Block::new(BlockKind::OrderedList(vec![ListItem {
                content: RichText {
                    spans: vec![Span {
                        text: "Ordered 1".to_string(),
                        marks: Marks::default(),
                        link: None,
                    }],
                },
                sub_list: None,
            }])),
        ]);
        assert_eq!(
            Serializer::serialize(&doc),
            "<ul><li><p>Bullet 1</p><ol><li><p>Sub ordered 1</p></li></ol></li><li><p>Bullet 2</p></li></ul><ol><li><p>Ordered 1</p></li></ol>"
        );
    }

    #[test]
    fn test_serialize_task_list() {
        let doc = Document::new(vec![Block::new(BlockKind::TaskList(vec![
            TaskItem {
                checked: true,
                content: RichText {
                    spans: vec![Span {
                        text: "Done task".to_string(),
                        marks: Marks::default(),
                        link: None,
                    }],
                },
            },
            TaskItem {
                checked: false,
                content: RichText {
                    spans: vec![Span {
                        text: "Pending task".to_string(),
                        marks: Marks::default(),
                        link: None,
                    }],
                },
            },
        ]))]);
        assert_eq!(
            Serializer::serialize(&doc),
            "<ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>Done task</p></div></li><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p>Pending task</p></div></li></ul>"
        );
    }

    #[test]
    fn test_roundtrip_html_to_document_to_html() {
        let html_cases = vec![
            "<p>Simple paragraph</p>",
            "<h1>Main Title</h1><h2>Subtitle</h2>",
            "<blockquote><p>Wise words here</p></blockquote>",
            "<p><strong>Bold</strong>, <em>Italic</em>, <u>Under</u>, <s>Strike</s>, <code>Code</code></p>",
            "<p><a href=\"https://tnotes.app\">TNotes</a></p>",
            "<hr>",
            "<ul><li><p>Item 1</p></li><li><p>Item 2</p></li></ul>",
            "<ol><li><p>Step 1</p></li><li><p>Step 2</p></li></ol>",
            "<ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>Item 1</p></div></li><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p>Item 2</p></div></li></ul>",
            "<pre><code class=\"language-rust\">let x = 1;</code></pre>",
        ];

        let parser = Parser::new();
        for html in html_cases {
            let doc = parser.parse(html).unwrap();
            let serialized = Serializer::serialize(&doc);
            assert_eq!(serialized, html, "Failed roundtrip for: {html}");
        }
    }

    #[test]
    fn test_escape_special_characters() {
        let doc = Document::new(vec![Block::new(BlockKind::Paragraph(RichText {
            spans: vec![Span {
                text: "<script>alert('xss' & \"hack\")</script>".to_string(),
                marks: Marks::default(),
                link: None,
            }],
        }))]);
        assert_eq!(
            Serializer::serialize(&doc),
            "<p>&lt;script&gt;alert(&#39;xss&#39; &amp; &quot;hack&quot;)&lt;/script&gt;</p>"
        );
    }
}
