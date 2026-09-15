use gpui::*;
use crate::components::{Input, InputState};
use super::{RenameKind, RenameState, SidebarView};

impl SidebarView {
    pub fn begin_rename_note(&mut self, note_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let title = self
            .store
            .read(cx)
            .active_notes()
            .into_iter()
            .find(|n| n.id == note_id)
            .map(|n| n.title);
        if let Some(title) = title {
            self.renaming = Some(RenameState {
                kind: RenameKind::Note,
                id: note_id.to_string(),
                input: InputState::new(title),
                focus: cx.focus_handle(),
            });
            self.context_menu = None;
            self.picking_icon_for = None;
            if let Some(state) = self.renaming.as_ref() {
                window.focus(&state.focus);
            }
            cx.notify();
        }
    }

    pub fn begin_rename_folder(
        &mut self,
        folder_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = self
            .store
            .read(cx)
            .folder_tree()
            .iter()
            .find(|n| n.folder.id == folder_id)
            .map(|n| n.folder.name.clone());
        if let Some(name) = name {
            self.renaming = Some(RenameState {
                kind: RenameKind::Folder,
                id: folder_id.to_string(),
                input: InputState::new(name),
                focus: cx.focus_handle(),
            });
            self.context_menu = None;
            self.picking_icon_for = None;
            if let Some(state) = self.renaming.as_ref() {
                window.focus(&state.focus);
            }
            cx.notify();
        }
    }

    pub fn commit_rename(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = self.renaming.take() {
            let value = state.input.value().to_string();
            match state.kind {
                RenameKind::Folder => self
                    .store
                    .update(cx, |store, cx| store.rename_folder(&state.id, &value, cx)),
                RenameKind::Note => self
                    .store
                    .update(cx, |store, cx| store.rename_note(&state.id, &value, cx)),
            }
            cx.notify();
        }
    }

    pub fn cancel_rename(&mut self, cx: &mut Context<Self>) {
        if self.renaming.take().is_some() {
            cx.notify();
        }
    }

    fn handle_rename_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "enter" {
            self.commit_rename(cx);
        } else if key == "escape" {
            self.cancel_rename(cx);
        } else if let Some(state) = self.renaming.as_mut()
            && state.input.handle_key(event, window, cx)
        {
            cx.notify();
        }
    }

    /// Inline rename editor for the folder/note row with `id`, or `None` when
    /// no rename session targets it.
    pub(super) fn rename_editor_row(
        &self,
        id: &str,
        indent: Pixels,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let state = self.renaming.as_ref()?;
        if state.id != id {
            return None;
        }
        let input = state.input.clone();
        let focus = state.focus.clone();
        Some(
            div()
                .pl(indent)
                .pr_2()
                .py(px(1.))
                .child(
                    Input::new(SharedString::from(format!("rename-{id}")))
                        .state(&input)
                        .focus_handle(focus)
                        .placeholder("Name")
                        .on_key_down(cx.listener(|this, event, window, cx| {
                            this.handle_rename_key(event, window, cx);
                        })),
                )
                .into_any_element(),
        )
    }
}
