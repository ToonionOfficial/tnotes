mod app;
mod components;
mod editor;
mod theme;
mod views;

use gpui::*;
use views::editor::EditorView;
use views::sidebar::SidebarView;

struct TNotesWorkspace {
    sidebar: Entity<SidebarView>,
    editor: Entity<EditorView>,
}

impl TNotesWorkspace {
    fn new(cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(|_cx| SidebarView::new());
        let initial_note = sidebar.read(cx).selected_note().cloned();

        let (title, html) = if let Some(note) = initial_note {
            (note.title, note.html_content)
        } else {
            ("Untitled Note".to_string(), "<p></p>".to_string())
        };

        let editor = cx.new(|_cx| EditorView::new(title, &html));

        // Subscribe to sidebar changes to update the editor
        cx.observe(&sidebar, |this, sidebar, cx| {
            let note = sidebar.read(cx).selected_note().cloned();
            if let Some(note) = note {
                this.editor.update(cx, |editor, cx| {
                    editor.set_note(note.title, &note.html_content);
                    cx.notify();
                });
            }
        })
        .detach();

        Self { sidebar, editor }
    }
}

impl Render for TNotesWorkspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(0x141318))
            .text_color(rgb(0xe6e1e9))
            .child(
                div()
                    .w_80()
                    .h_full()
                    .flex_shrink_0()
                    .child(self.sidebar.clone()),
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(self.editor.clone()),
            )
    }
}

fn main() {
    let platform = gpui_platform::current_platform(false);
    Application::with_platform(platform).run(|cx: &mut App| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1000.0), px(650.0)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("TNotes".into()),
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(options, |_window, cx| {
            cx.new(TNotesWorkspace::new)
        })
        .unwrap();
    });
}
