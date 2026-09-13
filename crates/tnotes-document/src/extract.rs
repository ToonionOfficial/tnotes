use crate::model::{BlockKind, Document, ListItem, RichText, SubList};

impl Document {
    pub fn extract_text(&self) -> String {
        let mut buffer = Vec::new();
        for block in &self.blocks {
            match &block.kind {
                BlockKind::Paragraph(rt) | BlockKind::Quote(rt) => {
                    extract_rich_text(rt, &mut buffer);
                }
                BlockKind::Heading { content, .. } => {
                    extract_rich_text(content, &mut buffer);
                }
                BlockKind::BulletList(items) | BlockKind::OrderedList(items) => {
                    for item in items {
                        extract_list_item(item, &mut buffer);
                    }
                }
                BlockKind::TaskList(items) => {
                    for item in items {
                        extract_rich_text(&item.content, &mut buffer);
                    }
                }
                BlockKind::CodeBlock { code, .. } => {
                    if !code.trim().is_empty() {
                        buffer.push(code.trim().to_string());
                    }
                }
                BlockKind::Table(table) => {
                    for h in &table.headers {
                        if !h.trim().is_empty() {
                            buffer.push(h.trim().to_string());
                        }
                    }
                    for row in &table.rows {
                        for cell in row {
                            if !cell.trim().is_empty() {
                                buffer.push(cell.trim().to_string());
                            }
                        }
                    }
                }
                BlockKind::Image(img) => {
                    if let Some(alt) = &img.alt {
                        if !alt.trim().is_empty() {
                            buffer.push(alt.trim().to_string());
                        }
                    }
                }
                BlockKind::Divider | BlockKind::Drawing(_) | BlockKind::Audio(_) => {}
            }
        }
        buffer.join(" ")
    }
}

fn extract_rich_text(rt: &RichText, buffer: &mut Vec<String>) {
    let text: String = rt.spans.iter().map(|s| s.text.as_str()).collect();
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        buffer.push(trimmed.to_string());
    }
}

fn extract_list_item(item: &ListItem, buffer: &mut Vec<String>) {
    extract_rich_text(&item.content, buffer);
    if let Some(sub) = &item.sub_list {
        match &**sub {
            SubList::Bullet(sub_items) | SubList::Ordered(sub_items) => {
                for sub_item in sub_items {
                    extract_list_item(sub_item, buffer);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::*;

    #[test]
    fn test_extract_text_empty() {
        let doc = Document::empty();
        assert_eq!(doc.extract_text(), "");
    }

    #[test]
    fn test_extract_text_comprehensive() {
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
            Block::new(BlockKind::Image(ImageData {
                asset_id: "img1".to_string(),
                alt: Some("Architecture diagram".to_string()),
                width: None,
                height: None,
            })),
            Block::new(BlockKind::Divider),
        ]);

        let extracted = doc.extract_text();
        assert_eq!(
            extracted,
            "Meeting Notes Discussed roadmap. Feature Status FTS Done Write tests Ship release Architecture diagram"
        );
        assert!(!extracted.contains("blocks"));
        assert!(!extracted.contains("type"));
        assert!(!extracted.contains("Paragraph"));
    }
}
