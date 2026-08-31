use crate::components::*;
use gpui::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FolderItem {
    pub id: String,
    pub name: String,
    pub count: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoteItem {
    pub id: String,
    pub title: String,
    pub folder_id: Option<String>,
    pub html_content: String,
}

#[allow(dead_code)]
pub struct SidebarView {
    pub folders: Vec<FolderItem>,
    pub notes: Vec<NoteItem>,
    pub selected_item_id: Option<String>,
    pub search_query: String,
}

impl SidebarView {
    pub fn new() -> Self {
        let folders = vec![
            FolderItem {
                id: "f-1".to_string(),
                name: "Work".to_string(),
                count: 6,
            },
            FolderItem {
                id: "f-2".to_string(),
                name: "Personal".to_string(),
                count: 4,
            },
            FolderItem {
                id: "f-3".to_string(),
                name: "Research".to_string(),
                count: 4,
            },
            FolderItem {
                id: "f-4".to_string(),
                name: "Studio".to_string(),
                count: 2,
            },
        ];

        let notes = vec![
            NoteItem {
                id: "n-1".to_string(),
                title: "Welcome to TNotes".to_string(),
                folder_id: None,
                html_content: "<h1>Welcome to TNotes</h1><p>TNotes is a fast, minimal, native note-taking app with instant sync across desktop, mobile, and web.</p><blockquote><p>Simplicity is prerequisite for reliability.</p></blockquote><ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>HTML parser & serializer</p></div></li><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p>GPUI Desktop UI</p></div></li></ul>".to_string(),
            },
            NoteItem {
                id: "n-2".to_string(),
                title: "Architecture & Document AST".to_string(),
                folder_id: None,
                html_content: "<h2>Architecture & Document AST</h2><p>The document model in <code>tnotes-document</code> provides editor-independent AST representation for rich text.</p><pre><code class=\"language-rust\">let doc = Document::from_html(html)?;\nlet out = doc.to_html();</code></pre>".to_string(),
            },
            NoteItem {
                id: "n-3".to_string(),
                title: "Bevy Engine Exploration".to_string(),
                folder_id: None,
                html_content: "<h2>Bevy Engine Exploration</h2><p>ECS architecture notes and rendering pipeline ideas.</p>".to_string(),
            },
            NoteItem {
                id: "n-4".to_string(),
                title: "Neovim Shortcuts".to_string(),
                folder_id: None,
                html_content: "<h2>Neovim Shortcuts</h2><p>Keybindings reference for custom workflow.</p>".to_string(),
            },
            NoteItem {
                id: "n-5".to_string(),
                title: "Release Checklist".to_string(),
                folder_id: None,
                html_content: "<h2>Release Checklist</h2><ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>Verify SQLite migrations</p></div></li><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p>Publish desktop build</p></div></li></ul>".to_string(),
            },
            NoteItem {
                id: "n-6".to_string(),
                title: "Untitled".to_string(),
                folder_id: None,
                html_content: "<p></p>".to_string(),
            },
        ];

        let selected_item_id = notes.first().map(|n| n.id.clone());

        Self {
            folders,
            notes,
            selected_item_id,
            search_query: String::new(),
        }
    }

    pub fn selected_note(&self) -> Option<&NoteItem> {
        let sel_id = self.selected_item_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == sel_id)
    }

    pub fn select_item(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_item_id = Some(id);
        cx.notify();
    }

    pub fn add_note(&mut self, cx: &mut Context<Self>) {
        let id = format!("n-{}", self.notes.len() + 1);
        let new_note = NoteItem {
            id: id.clone(),
            title: "Untitled Note".to_string(),
            folder_id: None,
            html_content: "<h1>Untitled Note</h1><p></p>".to_string(),
        };
        self.notes.insert(0, new_note);
        self.selected_item_id = Some(id);
        cx.notify();
    }
}

impl Default for SidebarView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_id = self.selected_item_id.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x141318)) // Theme background
            .border_r_1()
            .border_color(rgb(0x302e36)) // Theme border
            .p_2p5()
            .gap_3()
            // 1. Header
            .child(sidebar_header(
                "TNotes",
                cx.listener(|this, _, _window, cx| {
                    this.add_note(cx);
                }),
            ))
            // 2. Search Bar
            .child(search_bar())
            // 3. Scrollable Tree: Folders & Notes
            .child(
                div()
                    .id("sidebar-tree-scroll")
                    .flex()
                    .flex_col()
                    .gap_3()
                    .flex_1()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(section_header("NOTES"))
                            // Folders list
                            .children(self.folders.iter().enumerate().map(|(idx, folder)| {
                                folder_row(idx as u64, folder.name.clone(), folder.count)
                            }))
                            // Root Notes list
                            .children(self.notes.iter().enumerate().map(|(idx, note)| {
                                let is_selected = selected_id.as_deref() == Some(&note.id);
                                let note_id = note.id.clone();

                                note_row(
                                    idx as u64,
                                    note.title.clone(),
                                    is_selected,
                                    cx.listener({
                                        let note_id = note_id.clone();
                                        move |this, _, _window, cx| {
                                            this.select_item(note_id.clone(), cx);
                                        }
                                    }),
                                )
                            })),
                    ),
            )
            // 4. Pinned SYSTEM Section at Bottom
            .child(system_section())
            // 5. Pinned Profile & Settings Footer
            .child(sidebar_footer("Local Account", "Synced with server"))
    }
}

fn section_header(label: &'static str) -> impl IntoElement {
    div()
        .px_2()
        .pt_2()
        .pb_1()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0x79747e)) // Subtle uppercase category label
        .child(label)
}
