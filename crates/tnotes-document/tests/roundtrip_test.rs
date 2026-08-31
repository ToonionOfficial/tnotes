use tnotes_document::{
    Block, Document, ListItem, Marks, RichText, Span, SubList, TaskItem,
};

#[test]
fn test_document_to_html_to_document_roundtrip() {
    let original_doc = Document {
        blocks: vec![
            Block::Heading {
                level: 1,
                content: RichText {
                    spans: vec![Span {
                        text: "Project Roadmap".to_string(),
                        marks: Marks::default(),
                        link: None,
                    }],
                },
            },
            Block::Paragraph(RichText {
                spans: vec![
                    Span {
                        text: "This note contains ".to_string(),
                        marks: Marks::default(),
                        link: None,
                    },
                    Span {
                        text: "bold text".to_string(),
                        marks: Marks {
                            bold: true,
                            ..Default::default()
                        },
                        link: None,
                    },
                    Span {
                        text: ", ".to_string(),
                        marks: Marks::default(),
                        link: None,
                    },
                    Span {
                        text: "italic text".to_string(),
                        marks: Marks {
                            italic: true,
                            ..Default::default()
                        },
                        link: None,
                    },
                    Span {
                        text: ", and a ".to_string(),
                        marks: Marks::default(),
                        link: None,
                    },
                    Span {
                        text: "link".to_string(),
                        marks: Marks::default(),
                        link: Some("https://tnotes.app".to_string()),
                    },
                    Span {
                        text: ".".to_string(),
                        marks: Marks::default(),
                        link: None,
                    },
                ],
            }),
            Block::Quote(RichText {
                spans: vec![Span {
                    text: "Simplicity is prerequisite for reliability.".to_string(),
                    marks: Marks::default(),
                    link: None,
                }],
            }),
            Block::Divider,
            Block::TaskList(vec![
                TaskItem {
                    checked: true,
                    content: RichText {
                        spans: vec![Span {
                            text: "Setup repo".to_string(),
                            marks: Marks::default(),
                            link: None,
                        }],
                    },
                },
                TaskItem {
                    checked: false,
                    content: RichText {
                        spans: vec![Span {
                            text: "Implement parser & serializer".to_string(),
                            marks: Marks::default(),
                            link: None,
                        }],
                    },
                },
            ]),
            Block::BulletList(vec![
                ListItem {
                    content: RichText {
                        spans: vec![Span {
                            text: "Category A".to_string(),
                            marks: Marks::default(),
                            link: None,
                        }],
                    },
                    sub_list: Some(Box::new(SubList::Ordered(vec![ListItem {
                        content: RichText {
                            spans: vec![Span {
                                text: "Sub-item 1".to_string(),
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
                            text: "Category B".to_string(),
                            marks: Marks::default(),
                            link: None,
                        }],
                    },
                    sub_list: None,
                },
            ]),
            Block::CodeBlock {
                language: Some("rust".to_string()),
                code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
            },
        ],
    };

    let html = original_doc.to_html();
    let parsed_doc = Document::from_html(&html).expect("Failed to parse serialized HTML");

    assert_eq!(
        original_doc, parsed_doc,
        "Document -> HTML -> Document failed to produce identical AST"
    );
}

#[test]
fn test_code_block_with_html_tags_inside() {
    let html = r#"<pre><code class="language-html">&lt;div class=&quot;box&quot;&gt;Hello &amp; world&lt;/div&gt;</code></pre>"#;
    let doc = Document::from_html(html).unwrap();

    if let Block::CodeBlock { language, code } = &doc.blocks[0] {
        assert_eq!(language.as_deref(), Some("html"));
        assert_eq!(code, "<div class=\"box\">Hello & world</div>");
    } else {
        panic!("Expected CodeBlock");
    }

    let serialized = doc.to_html();
    let doc2 = Document::from_html(&serialized).unwrap();
    assert_eq!(doc, doc2);
}

#[test]
fn test_checklist_with_rich_text_and_links() {
    let html = r#"<ul data-type="taskList"><li data-type="taskItem" data-checked="true"><label><input type="checkbox" checked="checked"><span></span></label><div><p><strong>Urgent:</strong> Buy <a href="https://store.com">groceries</a></p></div></li><li data-type="taskItem" data-checked="false"><label><input type="checkbox"><span></span></label><div><p>Review <code>PR #10</code></p></div></li></ul>"#;
    let doc = Document::from_html(html).unwrap();
    let serialized = doc.to_html();

    assert_eq!(serialized, html);
    let doc2 = Document::from_html(&serialized).unwrap();
    assert_eq!(doc, doc2);
}

#[test]
fn test_multiline_line_breaks_roundtrip() {
    let html = "<p>Line 1<br>Line 2<br>Line 3</p>";
    let doc = Document::from_html(html).unwrap();
    let serialized = doc.to_html();

    assert_eq!(serialized, html);
    let doc2 = Document::from_html(&serialized).unwrap();
    assert_eq!(doc, doc2);
}

#[test]
fn test_deeply_nested_lists_roundtrip() {
    let html = "<ul><li><p>Level 1</p><ol><li><p>Level 2</p><ul><li><p>Level 3</p></li></ul></li></ol></li></ul>";
    let doc = Document::from_html(html).unwrap();
    let serialized = doc.to_html();

    assert_eq!(serialized, html);
    let doc2 = Document::from_html(&serialized).unwrap();
    assert_eq!(doc, doc2);
}
