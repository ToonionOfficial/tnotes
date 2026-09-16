use tnotes_document::{Block, BlockKind, Document, RichText};

/// Stored `notes.body` holds canonical document JSON (`Document::to_json`).
/// Older rows may hold plain markdown and empty means an empty note; both
/// are handled on read so no data migration is required.
/// Markdown text for the editor from a stored body.
pub fn body_to_markdown(body: &str) -> String {
    if body.trim().is_empty() {
        return String::new();
    }
    match Document::from_json(body) {
        Ok(doc) => doc.to_markdown(),
        // Legacy row holding plain markdown: show it as-is.
        Err(_) => body.to_string(),
    }
}

/// Canonical JSON to store for editor markdown. Never drops content:
/// input the markdown parser rejects is kept as a plain paragraph.
pub fn markdown_to_body_json(markdown: &str) -> String {
    let doc = Document::from_markdown(markdown).unwrap_or_else(|_| {
        Document::new(vec![Block::new(BlockKind::Paragraph(RichText::plain(
            markdown,
        )))])
    });
    doc.to_json().unwrap_or_default()
}

/// Plain-text search content for editor markdown.
pub fn markdown_to_searchable(markdown: &str) -> String {
    Document::from_markdown(markdown)
        .map(|doc| doc.extract_text())
        .unwrap_or_else(|_| markdown.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_round_trips_through_json() {
        let markdown = "# Shopping\n\nBuy **milk** and eggs";
        let stored = markdown_to_body_json(markdown);
        // Stored form is JSON, not the raw markdown.
        assert!(stored.contains("\"blocks\""));
        let back = body_to_markdown(&stored);
        assert!(back.contains("Shopping"));
        assert!(back.contains("milk"));
    }

    #[test]
    fn empty_body_loads_empty() {
        assert_eq!(body_to_markdown(""), "");
        assert_eq!(body_to_markdown("   "), "");
    }

    #[test]
    fn legacy_markdown_body_loads_as_is() {
        let legacy = "# Old note\n\nplain **markdown**, never converted";
        assert_eq!(body_to_markdown(legacy), legacy);
    }

    #[test]
    fn empty_markdown_stores_loadable_json() {
        let stored = markdown_to_body_json("");
        assert_eq!(body_to_markdown(&stored), "");
    }

    #[test]
    fn searchable_text_has_no_markup() {
        let searchable = markdown_to_searchable("# Title\n\nSome **bold** text");
        assert!(searchable.contains("Title"));
        assert!(searchable.contains("bold"));
        assert!(!searchable.contains("**"));
        assert!(!searchable.contains('#'));
    }
}
