use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::error::ParseError;
use crate::model::*;

pub fn parse_markdown(md: &str) -> Result<Document, ParseError> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(md, options);

    let mut blocks = Vec::new();

    let mut marks = Marks::default();
    let mut current_link: Option<String> = None;
    let mut inline_spans: Vec<Span> = Vec::new();

    let mut in_heading: Option<u8> = None;
    let mut in_quote = false;
    let mut quote_spans: Vec<Span> = Vec::new();
    let mut in_code_block: Option<Option<String>> = None;
    let mut code_buffer = String::new();

    let mut list_stack: Vec<ListBuilder> = Vec::new();
    let mut current_table: Option<TableBuilder> = None;

    let mut current_image: Option<(String, String)> = None;
    let mut in_image = false;
    let mut image_alt = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                in_heading = Some(lvl);
                inline_spans.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = in_heading.take() {
                    let content = RichText::new(normalize_spans(std::mem::take(&mut inline_spans)));
                    blocks.push(Block::new(BlockKind::Heading { level, content }));
                }
            }

            Event::Start(Tag::BlockQuote(_)) => {
                in_quote = true;
                inline_spans.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                in_quote = false;
                let spans = if !inline_spans.is_empty() {
                    std::mem::take(&mut inline_spans)
                } else {
                    std::mem::take(&mut quote_spans)
                };
                let content = RichText::new(normalize_spans(spans));
                blocks.push(Block::new(BlockKind::Quote(content)));
            }

            Event::Start(Tag::Paragraph) => {
                inline_spans.clear();
                current_image = None;
            }
            Event::End(TagEnd::Paragraph) => {
                if in_quote {
                    if !quote_spans.is_empty() && !inline_spans.is_empty() {
                        quote_spans.push(Span::plain("\n\n"));
                    }
                    quote_spans.append(&mut inline_spans);
                } else if let Some((url, alt)) = current_image.take() {
                    if inline_spans.is_empty() {
                        let asset_id = url
                            .strip_prefix("asset://")
                            .unwrap_or(&url)
                            .to_string();
                        blocks.push(Block::new(BlockKind::Image(ImageData {
                            asset_id,
                            alt: if alt.is_empty() { None } else { Some(alt) },
                            width: None,
                            height: None,
                        })));
                    } else {
                        let content = RichText::new(normalize_spans(std::mem::take(&mut inline_spans)));
                        blocks.push(Block::new(BlockKind::Paragraph(content)));
                    }
                } else if !list_stack.is_empty() {
                    if let Some(current_list) = list_stack.last_mut() {
                        if let Some(current_item) = current_list.items.last_mut() {
                            current_item.append_spans(&mut inline_spans);
                        }
                    }
                } else if !inline_spans.is_empty() {
                    let content = RichText::new(normalize_spans(std::mem::take(&mut inline_spans)));
                    blocks.push(Block::new(BlockKind::Paragraph(content)));
                }
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => {
                        let s = l.trim();
                        if s.is_empty() {
                            None
                        } else {
                            Some(s.to_string())
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
                in_code_block = Some(lang);
                code_buffer.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(language) = in_code_block.take() {
                    let code = std::mem::take(&mut code_buffer);
                    blocks.push(Block::new(BlockKind::CodeBlock { language, code }));
                }
            }

            Event::Rule => {
                blocks.push(Block::new(BlockKind::Divider));
            }

            Event::Start(Tag::Table(_)) => {
                current_table = Some(TableBuilder::default());
            }
            Event::Start(Tag::TableHead) => {
                if let Some(table) = &mut current_table {
                    table.in_head = true;
                    table.current_row.clear();
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(table) = &mut current_table {
                    table.headers = std::mem::take(&mut table.current_row);
                    table.in_head = false;
                }
            }
            Event::Start(Tag::TableRow) => {
                if let Some(table) = &mut current_table {
                    table.current_row.clear();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(table) = &mut current_table {
                    if !table.in_head {
                        let row = std::mem::take(&mut table.current_row);
                        table.rows.push(row);
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                if let Some(table) = &mut current_table {
                    table.current_cell.clear();
                }
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(table) = &mut current_table {
                    let cell = table.current_cell.trim().to_string();
                    table.current_row.push(cell);
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = current_table.take() {
                    blocks.push(Block::new(BlockKind::Table(TableData {
                        headers: table.headers,
                        rows: table.rows,
                    })));
                }
            }

            Event::Start(Tag::List(first_num)) => {
                if !inline_spans.is_empty() {
                    if let Some(current_list) = list_stack.last_mut() {
                        if let Some(current_item) = current_list.items.last_mut() {
                            current_item.append_spans(&mut inline_spans);
                        }
                    }
                }
                list_stack.push(ListBuilder {
                    is_ordered: first_num.is_some(),
                    items: Vec::new(),
                });
            }
            Event::Start(Tag::Item) => {
                if !inline_spans.is_empty() {
                    if let Some(current_list) = list_stack.last_mut() {
                        if let Some(current_item) = current_list.items.last_mut() {
                            current_item.append_spans(&mut inline_spans);
                        }
                    }
                }
                if let Some(current_list) = list_stack.last_mut() {
                    current_list.items.push(ListItemBuilder::Regular {
                        content: Vec::new(),
                        sub_list: None,
                    });
                }
            }
            Event::TaskListMarker(checked) => {
                if let Some(current_list) = list_stack.last_mut() {
                    if let Some(item) = current_list.items.last_mut() {
                        *item = ListItemBuilder::Task {
                            checked,
                            content: Vec::new(),
                        };
                    }
                }
            }
            Event::End(TagEnd::Item) => {
                if !inline_spans.is_empty() {
                    if let Some(current_list) = list_stack.last_mut() {
                        if let Some(current_item) = current_list.items.last_mut() {
                            current_item.append_spans(&mut inline_spans);
                        }
                    }
                }
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(finished_list) = list_stack.pop() {
                    let is_task_list = finished_list
                        .items
                        .iter()
                        .any(|it| matches!(it, ListItemBuilder::Task { .. }));

                    if list_stack.is_empty() {
                        if is_task_list {
                            let tasks = finished_list
                                .items
                                .into_iter()
                                .map(|it| match it {
                                    ListItemBuilder::Task { checked, content } => {
                                        TaskItem::new(checked, RichText::new(normalize_spans(content)))
                                    }
                                    ListItemBuilder::Regular { content, .. } => {
                                        TaskItem::new(false, RichText::new(normalize_spans(content)))
                                    }
                                })
                                .collect();
                            blocks.push(Block::new(BlockKind::TaskList(tasks)));
                        } else if finished_list.is_ordered {
                            let list_items = finished_list
                                .items
                                .into_iter()
                                .map(|it| it.into_list_item())
                                .collect();
                            blocks.push(Block::new(BlockKind::OrderedList(list_items)));
                        } else {
                            let list_items = finished_list
                                .items
                                .into_iter()
                                .map(|it| it.into_list_item())
                                .collect();
                            blocks.push(Block::new(BlockKind::BulletList(list_items)));
                        }
                    } else {
                        let sub_list = if finished_list.is_ordered {
                            SubList::Ordered(
                                finished_list
                                    .items
                                    .into_iter()
                                    .map(|it| it.into_list_item())
                                    .collect(),
                            )
                        } else {
                            SubList::Bullet(
                                finished_list
                                    .items
                                    .into_iter()
                                    .map(|it| it.into_list_item())
                                    .collect(),
                            )
                        };

                        if let Some(parent_list) = list_stack.last_mut() {
                            if let Some(parent_item) = parent_list.items.last_mut() {
                                match parent_item {
                                    ListItemBuilder::Regular { sub_list: slot, .. } => {
                                        *slot = Some(Box::new(sub_list));
                                    }
                                    ListItemBuilder::Task { .. } => {}
                                }
                            }
                        }
                    }
                }
            }

            Event::Start(Tag::Image { dest_url, .. }) => {
                in_image = true;
                image_alt.clear();
                current_image = Some((dest_url.to_string(), String::new()));
            }
            Event::End(TagEnd::Image) => {
                in_image = false;
                if let Some((url, _)) = current_image.take() {
                    let alt = std::mem::take(&mut image_alt);
                    current_image = Some((url, alt));
                }
            }

            Event::Start(Tag::Strong) => marks.bold = true,
            Event::End(TagEnd::Strong) => marks.bold = false,
            Event::Start(Tag::Emphasis) => marks.italic = true,
            Event::End(TagEnd::Emphasis) => marks.italic = false,
            Event::Start(Tag::Strikethrough) => marks.strike = true,
            Event::End(TagEnd::Strikethrough) => marks.strike = false,
            Event::Start(Tag::Link { dest_url, .. }) => {
                current_link = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => {
                current_link = None;
            }

            Event::Text(t) => {
                if in_image {
                    image_alt.push_str(&t);
                } else if in_code_block.is_some() {
                    code_buffer.push_str(&t);
                } else if let Some(table) = &mut current_table {
                    table.current_cell.push_str(&t);
                } else {
                    inline_spans.push(Span {
                        text: t.to_string(),
                        marks,
                        link: current_link.clone(),
                    });
                }
            }
            Event::Code(c) => {
                if let Some(table) = &mut current_table {
                    table.current_cell.push('`');
                    table.current_cell.push_str(&c);
                    table.current_cell.push('`');
                } else {
                    let mut code_marks = marks;
                    code_marks.code = true;
                    inline_spans.push(Span {
                        text: c.to_string(),
                        marks: code_marks,
                        link: current_link.clone(),
                    });
                }
            }
            Event::SoftBreak => {
                if in_image {
                    image_alt.push(' ');
                } else if let Some(table) = &mut current_table {
                    table.current_cell.push(' ');
                } else {
                    inline_spans.push(Span {
                        text: " ".to_string(),
                        marks,
                        link: current_link.clone(),
                    });
                }
            }
            Event::HardBreak => {
                if in_image {
                    image_alt.push(' ');
                } else if let Some(table) = &mut current_table {
                    table.current_cell.push(' ');
                } else {
                    inline_spans.push(Span {
                        text: "\n".to_string(),
                        marks,
                        link: current_link.clone(),
                    });
                }
            }

            Event::Html(html) | Event::InlineHtml(html) => {
                let trimmed = html.trim();
                if trimmed == "<u>" {
                    marks.underline = true;
                } else if trimmed == "</u>" {
                    marks.underline = false;
                } else if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
                    parse_directive(trimmed, &mut blocks);
                }
            }

            _ => {}
        }
    }

    Ok(Document::new(blocks))
}

fn parse_directive(html: &str, blocks: &mut Vec<Block>) {
    let content = html
        .trim_start_matches("<!--")
        .trim_end_matches("-->")
        .trim();

    if let Some(rest) = content.strip_prefix("tnotes:drawing") {
        let asset_id = extract_attr(rest, "asset_id");
        blocks.push(Block::new(BlockKind::Drawing(DrawingData {
            asset_id,
            strokes: Vec::new(),
        })));
    } else if let Some(rest) = content.strip_prefix("tnotes:audio") {
        let asset_id = extract_attr(rest, "asset_id").unwrap_or_default();
        let duration_ms = extract_attr(rest, "duration_ms")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        blocks.push(Block::new(BlockKind::Audio(AudioData {
            asset_id,
            duration_ms,
            waveform: None,
        })));
    }
}

fn extract_attr(input: &str, key: &str) -> Option<String> {
    let pattern = format!("{key}=\"");
    if let Some(start) = input.find(&pattern) {
        let after = &input[start + pattern.len()..];
        if let Some(end) = after.find('"') {
            return Some(after[..end].to_string());
        }
    }
    None
}

fn normalize_spans(raw_spans: Vec<Span>) -> Vec<Span> {
    let mut normalized: Vec<Span> = Vec::with_capacity(raw_spans.len());

    for span in raw_spans {
        if span.text.is_empty() {
            continue;
        }

        if let Some(last) = normalized.last_mut() {
            if last.marks == span.marks && last.link == span.link {
                last.text.push_str(&span.text);
                continue;
            }
        }

        normalized.push(span);
    }

    normalized
}

#[derive(Default)]
struct TableBuilder {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    current_cell: String,
    in_head: bool,
}

struct ListBuilder {
    is_ordered: bool,
    items: Vec<ListItemBuilder>,
}

enum ListItemBuilder {
    Task {
        checked: bool,
        content: Vec<Span>,
    },
    Regular {
        content: Vec<Span>,
        sub_list: Option<Box<SubList>>,
    },
}

impl ListItemBuilder {
    fn append_spans(&mut self, spans: &mut Vec<Span>) {
        match self {
            Self::Task { content, .. } => content.append(spans),
            Self::Regular { content, .. } => content.append(spans),
        }
    }

    fn into_list_item(self) -> ListItem {
        match self {
            Self::Regular { content, sub_list } => ListItem {
                content: RichText::new(normalize_spans(content)),
                sub_list,
            },
            Self::Task { content, .. } => ListItem {
                content: RichText::new(normalize_spans(content)),
                sub_list: None,
            },
        }
    }
}
