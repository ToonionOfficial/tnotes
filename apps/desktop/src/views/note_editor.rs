use gpui::*;
use twrite::{ConcealMode, ContextMenuItem, Editor, EditorHook, SearchHook};

#[derive(Default)]
struct ContextMenuHook;

impl EditorHook for ContextMenuHook {
    fn context_menu_items(&self, _ctx: &twrite::ContextMenuContext) -> Vec<ContextMenuItem> {
        vec![]
    }
}

pub fn build_note_editor(cx: &mut Context<Editor>) -> Editor {
    let mut ed = Editor::new("", cx);

    ed.config.line_numbers = true;
    ed.theme.background = rgb(0x141318).into();
    ed.config.markdown.conceal_mode = ConcealMode::Hidden;

    ed.enable_markdown();
    ed.add_hook(SearchHook::new());
    ed.add_hook(ContextMenuHook);
    ed
}
