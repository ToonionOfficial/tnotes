use gpui::*;
use twrite::{ConcealMode, Editor, SearchHook};

/// Build the note-taking editor instance.
///
/// Centralizes all editor setup (config, theme, markdown, hooks) so
/// `NoteView` stays lean and new behavior lands here.
///
/// To add a custom hook:
/// 1. Define it in this file (or a `hooks/` submodule) by implementing
///    `twrite::EditorHook`.
/// 2. Register it with `ed.add_hook(MyHook::new())` in the section below.
pub fn build_note_editor(cx: &mut Context<Editor>) -> Editor {
    let mut ed = Editor::new("", cx);
    ed.config.line_numbers = true;
    ed.add_hook(SearchHook::new());
    ed.theme.background = rgb(0x141318).into();
    ed.config.markdown.conceal_mode = ConcealMode::Hidden;
    ed.enable_markdown();
    ed
}
