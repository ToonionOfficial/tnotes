use crate::model::*;

pub fn serialize_markdown(blocks: &[Block]) -> String {
    let mut out = String::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        serialize_block(block, &mut out);
    }
    out
}

fn serialize_block(block: &Block, out: &mut String) {
    match &block.kind {
        BlockKind::Paragraph(rt) => {
            serialize_rich_text(rt, out);
            out.push('\n');
        }
        BlockKind::Heading { level, content } => {
            let clamped = (*level).clamp(1, 6);
            for _ in 0..clamped {
                out.push('#');
            }
            out.push(' ');
            serialize_rich_text(content, out);
            out.push('\n');
        }
        BlockKind::Quote(rt) => {
            let mut quote_text = String::new();
            serialize_rich_text(rt, &mut quote_text);
            for line in quote_text.lines() {
                out.push_str("> ");
                out.push_str(line);
                out.push('\n');
            }
            if quote_text.is_empty() {
                out.push_str(">\n");
            }
        }
        BlockKind::CodeBlock { language, code } => {
            out.push_str("```");
            if let Some(lang) = language {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(code);
            if !code.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
        }
        BlockKind::Divider => {
            out.push_str("---\n");
        }
        BlockKind::BulletList(items) => {
            for item in items {
                serialize_list_item(item, "- ", 0, out);
            }
        }
        BlockKind::OrderedList(items) => {
            for (idx, item) in items.iter().enumerate() {
                let prefix = format!("{}. ", idx + 1);
                serialize_list_item(item, &prefix, 0, out);
            }
        }
        BlockKind::TaskList(items) => {
            for item in items {
                let check = if item.checked { "[x]" } else { "[ ]" };
                let prefix = format!("- {check} ");
                out.push_str(&prefix);
                serialize_rich_text(&item.content, out);
                out.push('\n');
            }
        }
        BlockKind::Table(table) => {
            serialize_table(table, out);
        }
        BlockKind::Image(img) => {
            let alt = img.alt.as_deref().unwrap_or("");
            let url = if img.asset_id.contains("://") {
                img.asset_id.clone()
            } else {
                format!("asset://{}", img.asset_id)
            };
            out.push_str(&format!("![{alt}]({url})\n"));
        }
        BlockKind::Drawing(drawing) => {
            if let Some(id) = &drawing.asset_id {
                out.push_str(&format!("<!-- tnotes:drawing asset_id=\"{id}\" -->\n"));
            } else {
                out.push_str("<!-- tnotes:drawing -->\n");
            }
        }
        BlockKind::Audio(audio) => {
            out.push_str(&format!(
                "<!-- tnotes:audio asset_id=\"{}\" duration_ms=\"{}\" -->\n",
                audio.asset_id, audio.duration_ms
            ));
        }
    }
}

fn serialize_list_item(item: &ListItem, prefix: &str, indent_level: usize, out: &mut String) {
    let indent = "  ".repeat(indent_level);
    out.push_str(&indent);
    out.push_str(prefix);
    serialize_rich_text(&item.content, out);
    out.push('\n');

    if let Some(sub) = &item.sub_list {
        match &**sub {
            SubList::Bullet(sub_items) => {
                for sub_item in sub_items {
                    serialize_list_item(sub_item, "- ", indent_level + 1, out);
                }
            }
            SubList::Ordered(sub_items) => {
                for (idx, sub_item) in sub_items.iter().enumerate() {
                    let sub_prefix = format!("{}. ", idx + 1);
                    serialize_list_item(sub_item, &sub_prefix, indent_level + 1, out);
                }
            }
        }
    }
}

fn serialize_table(table: &TableData, out: &mut String) {
    if table.headers.is_empty() && table.rows.is_empty() {
        return;
    }

    let col_count = table
        .headers
        .len()
        .max(table.rows.first().map(|r| r.len()).unwrap_or(0));

    if col_count == 0 {
        return;
    }

    out.push('|');
    for i in 0..col_count {
        let cell = table.headers.get(i).map(|s| s.as_str()).unwrap_or("");
        out.push(' ');
        out.push_str(cell);
        out.push_str(" |");
    }
    out.push('\n');

    out.push('|');
    for _ in 0..col_count {
        out.push_str(" --- |");
    }
    out.push('\n');

    for row in &table.rows {
        out.push('|');
        for i in 0..col_count {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            out.push(' ');
            out.push_str(cell);
            out.push_str(" |");
        }
        out.push('\n');
    }
}

fn serialize_rich_text(rt: &RichText, out: &mut String) {
    for span in &rt.spans {
        serialize_span(span, out);
    }
}

fn serialize_span(span: &Span, out: &mut String) {
    if span.text.is_empty() {
        return;
    }

    let raw = &span.text;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        out.push_str(raw);
        return;
    }

    let leading_ws = &raw[..raw.len() - raw.trim_start().len()];
    let trailing_ws = &raw[raw.trim_end().len()..];

    out.push_str(leading_ws);

    let mut inner = String::new();
    let marks = span.marks;

    if marks.code {
        inner.push('`');
        inner.push_str(trimmed);
        inner.push('`');
    } else {
        if marks.underline {
            inner.push_str("<u>");
        }
        if marks.strike {
            inner.push_str("~~");
        }
        if marks.bold && marks.italic {
            inner.push_str("***");
        } else if marks.bold {
            inner.push_str("**");
        } else if marks.italic {
            inner.push('*');
        }

        inner.push_str(trimmed);

        if marks.bold && marks.italic {
            inner.push_str("***");
        } else if marks.bold {
            inner.push_str("**");
        } else if marks.italic {
            inner.push('*');
        }
        if marks.strike {
            inner.push_str("~~");
        }
        if marks.underline {
            inner.push_str("</u>");
        }
    }

    if let Some(link) = &span.link {
        out.push('[');
        out.push_str(&inner);
        out.push_str("](");
        out.push_str(link);
        out.push(')');
    } else {
        out.push_str(&inner);
    }

    out.push_str(trailing_ws);
}
