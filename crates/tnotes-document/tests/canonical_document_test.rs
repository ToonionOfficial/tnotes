use tnotes_document::{
    AudioData, Block, BlockKind, Document, DrawingData, ImageData, ListItem, Marks, Point,
    RichText, Span, Stroke, TableData, TaskItem,
};
use ulid::Ulid;

#[test]
fn test_document_json_roundtrip() {
    let doc = Document::new(vec![
        Block::with_id(
            "01J8ABC123XYZ0000000000001",
            BlockKind::Heading {
                level: 1,
                content: RichText::plain("Title"),
            },
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000002",
            BlockKind::Paragraph(RichText {
                spans: vec![
                    Span::plain("Hello "),
                    Span {
                        text: "world".to_string(),
                        marks: Marks {
                            bold: true,
                            ..Default::default()
                        },
                        link: None,
                    },
                ],
            }),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000003",
            BlockKind::BulletList(vec![ListItem::new(RichText::plain("Item 1"))]),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000004",
            BlockKind::OrderedList(vec![ListItem::new(RichText::plain("Step 1"))]),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000005",
            BlockKind::TaskList(vec![TaskItem::new(true, RichText::plain("Done"))]),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000006",
            BlockKind::Quote(RichText::plain("A quote")),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000007",
            BlockKind::CodeBlock {
                language: Some("rust".to_string()),
                code: "let a = 1;".to_string(),
            },
        ),
        Block::with_id("01J8ABC123XYZ0000000000008", BlockKind::Divider),
        Block::with_id(
            "01J8ABC123XYZ0000000000009",
            BlockKind::Table(TableData {
                headers: vec!["Col 1".to_string(), "Col 2".to_string()],
                rows: vec![vec!["A".to_string(), "B".to_string()]],
            }),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000010",
            BlockKind::Drawing(DrawingData {
                asset_id: Some("draw_1".to_string()),
                strokes: vec![Stroke {
                    color: "#000000".to_string(),
                    width: 2.0,
                    points: vec![Point { x: 10, y: 20 }, Point { x: 30, y: 40 }],
                }],
            }),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000011",
            BlockKind::Image(ImageData {
                asset_id: "img_1".to_string(),
                alt: Some("Alt image".to_string()),
                width: Some(800),
                height: Some(600),
            }),
        ),
        Block::with_id(
            "01J8ABC123XYZ0000000000012",
            BlockKind::Audio(AudioData {
                asset_id: "audio_1".to_string(),
                duration_ms: 5000,
                waveform: Some(vec![1, 2, 3, 4]),
            }),
        ),
    ]);

    let json = doc.to_json().expect("Serialization to JSON failed");
    let deserialized = Document::from_json(&json).expect("Deserialization from JSON failed");

    assert_eq!(doc, deserialized);
}

#[test]
fn test_extract_text() {
    let doc = Document::new(vec![
        Block::new(BlockKind::Heading {
            level: 1,
            content: RichText::plain("Meeting Notes"),
        }),
        Block::new(BlockKind::Paragraph(RichText::plain("Discussed roadmap."))),
        Block::new(BlockKind::Table(TableData {
            headers: vec!["Feature".to_string(), "Status".to_string()],
            rows: vec![vec!["FTS".to_string(), "Done".to_string()]],
        })),
        Block::new(BlockKind::TaskList(vec![
            TaskItem::new(true, RichText::plain("Write tests")),
            TaskItem::new(false, RichText::plain("Ship release")),
        ])),
    ]);

    let extracted = doc.extract_text();
    assert_eq!(
        extracted,
        "Meeting Notes Discussed roadmap. Feature Status FTS Done Write tests Ship release"
    );
    assert!(!extracted.contains("blocks"));
    assert!(!extracted.contains("type"));
    assert!(!extracted.contains("Paragraph"));
}

#[test]
fn test_markdown_parser_headings_lists() {
    let md = "# Main Heading\n\n- [x] Task completed\n- [ ] Task pending\n\n* Bullet 1\n* Bullet 2\n\n> Quote block\n";
    let doc = Document::from_markdown(md).expect("Parsing markdown failed");

    assert_eq!(doc.blocks.len(), 4);

    match &doc.blocks[0].kind {
        BlockKind::Heading { level, content } => {
            assert_eq!(*level, 1);
            assert_eq!(content.spans[0].text, "Main Heading");
        }
        _ => panic!("Expected Heading"),
    }

    match &doc.blocks[1].kind {
        BlockKind::TaskList(items) => {
            assert_eq!(items.len(), 2);
            assert!(items[0].checked);
            assert_eq!(items[0].content.spans[0].text, "Task completed");
            assert!(!items[1].checked);
            assert_eq!(items[1].content.spans[0].text, "Task pending");
        }
        _ => panic!("Expected TaskList"),
    }

    match &doc.blocks[2].kind {
        BlockKind::BulletList(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].content.spans[0].text, "Bullet 1");
            assert_eq!(items[1].content.spans[0].text, "Bullet 2");
        }
        _ => panic!("Expected BulletList"),
    }

    match &doc.blocks[3].kind {
        BlockKind::Quote(content) => {
            assert_eq!(content.spans[0].text, "Quote block");
        }
        _ => panic!("Expected Quote"),
    }
}

#[test]
fn test_markdown_serializer_roundtrip() {
    let original = Document::new(vec![
        Block::new(BlockKind::Heading {
            level: 1,
            content: RichText::plain("Header"),
        }),
        Block::new(BlockKind::Paragraph(RichText::plain("Simple paragraph."))),
        Block::new(BlockKind::TaskList(vec![
            TaskItem::new(true, RichText::plain("Done")),
            TaskItem::new(false, RichText::plain("Pending")),
        ])),
        Block::new(BlockKind::Divider),
        Block::new(BlockKind::Table(TableData {
            headers: vec!["A".to_string(), "B".to_string()],
            rows: vec![vec!["1".to_string(), "2".to_string()]],
        })),
    ]);

    let md = original.to_markdown();
    let reparsed = Document::from_markdown(&md).expect("Roundtrip parse failed");

    assert_eq!(original.blocks.len(), reparsed.blocks.len());
    for (i, (b1, b2)) in original.blocks.iter().zip(&reparsed.blocks).enumerate() {
        assert_eq!(b1.kind, b2.kind, "Block kind mismatch at {i}");
    }
}

#[test]
fn test_stable_block_ids() {
    let b1 = Block::new(BlockKind::Divider);
    std::thread::sleep(std::time::Duration::from_millis(2));
    let b2 = Block::new(BlockKind::Divider);

    assert_eq!(b1.id.len(), 26);
    assert_eq!(b2.id.len(), 26);
    assert_ne!(b1.id, b2.id);

    let u1 = Ulid::from_string(&b1.id).expect("Invalid ULID in block 1");
    let u2 = Ulid::from_string(&b2.id).expect("Invalid ULID in block 2");

    assert!(u1.datetime() <= u2.datetime());
    assert!(u1 <= u2);
}

#[test]
fn test_markdown_inline_formatting_combinations() {
    let md = "Normal **bold** *italic* ~~strike~~ `code` [link](https://example.com)\n";
    let doc = Document::from_markdown(md).expect("Markdown parsing failed");
    assert_eq!(doc.blocks.len(), 1);

    if let BlockKind::Paragraph(rt) = &doc.blocks[0].kind {
        assert!(rt.spans.iter().any(|s| s.marks.bold && s.text == "bold"));
        assert!(
            rt.spans
                .iter()
                .any(|s| s.marks.italic && s.text == "italic")
        );
        assert!(
            rt.spans
                .iter()
                .any(|s| s.marks.strike && s.text == "strike")
        );
        assert!(rt.spans.iter().any(|s| s.marks.code && s.text == "code"));
        assert!(
            rt.spans
                .iter()
                .any(|s| s.link == Some("https://example.com".to_string()))
        );
    } else {
        panic!("Expected paragraph");
    }
}

#[test]
fn test_markdown_directives_roundtrip() {
    let original = Document::new(vec![
        Block::new(BlockKind::Drawing(DrawingData {
            asset_id: Some("draw_123".to_string()),
            strokes: vec![],
        })),
        Block::new(BlockKind::Audio(AudioData {
            asset_id: "audio_456".to_string(),
            duration_ms: 12000,
            waveform: None,
        })),
    ]);

    let md = original.to_markdown();
    assert!(md.contains("<!-- tnotes:drawing asset_id=\"draw_123\" -->"));
    assert!(md.contains("<!-- tnotes:audio asset_id=\"audio_456\" duration_ms=\"12000\" -->"));

    let reparsed = Document::from_markdown(&md).expect("Directive parsing failed");
    assert_eq!(reparsed.blocks.len(), 2);

    match &reparsed.blocks[0].kind {
        BlockKind::Drawing(d) => assert_eq!(d.asset_id.as_deref(), Some("draw_123")),
        _ => panic!("Expected Drawing"),
    }
    match &reparsed.blocks[1].kind {
        BlockKind::Audio(a) => {
            assert_eq!(a.asset_id, "audio_456");
            assert_eq!(a.duration_ms, 12000);
        }
        _ => panic!("Expected Audio"),
    }
}

#[test]
fn test_markdown_nested_lists() {
    let md = "1. First\n   - Sub A\n   - Sub B\n2. Second\n";
    let doc = Document::from_markdown(md).expect("Nested list parsing failed");
    assert_eq!(doc.blocks.len(), 1);

    if let BlockKind::OrderedList(items) = &doc.blocks[0].kind {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content.spans[0].text, "First");
        assert!(items[0].sub_list.is_some());
        assert_eq!(items[1].content.spans[0].text, "Second");
    } else {
        panic!("Expected OrderedList");
    }
}

#[test]
fn test_extract_text_deeply_nested() {
    let doc = Document::new(vec![
        Block::new(BlockKind::BulletList(vec![ListItem {
            content: RichText::plain("Top level"),
            sub_list: Some(Box::new(tnotes_document::SubList::Ordered(vec![
                ListItem {
                    content: RichText::plain("Nested level"),
                    sub_list: None,
                },
            ]))),
        }])),
        Block::new(BlockKind::CodeBlock {
            language: Some("rust".to_string()),
            code: "fn test() {}".to_string(),
        }),
    ]);

    let text = doc.extract_text();
    assert_eq!(text, "Top level Nested level fn test() {}");
}
