use crate::state::{
    auth_store::AuthStore, folder_store::FolderStore, note_store::NoteStore, sync_store::SyncStore,
};
use gpui::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveScreen {
    Workspace,
    Auth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    CommandPalette,
    Settings,
}

#[allow(dead_code)]
pub struct AppState {
    pub active_screen: ActiveScreen,
    pub active_modal: ActiveModal,
    pub note_store: Entity<NoteStore>,
    pub folder_store: Entity<FolderStore>,
    pub auth_store: Entity<AuthStore>,
    pub sync_store: Entity<SyncStore>,
}

impl AppState {
    pub fn new(
        note_store: Entity<NoteStore>,
        folder_store: Entity<FolderStore>,
        auth_store: Entity<AuthStore>,
        sync_store: Entity<SyncStore>,
    ) -> Self {
        Self {
            active_screen: ActiveScreen::Workspace,
            active_modal: ActiveModal::None,
            note_store,
            folder_store,
            auth_store,
            sync_store,
        }
    }

    #[allow(dead_code)]
    pub fn set_screen(&mut self, screen: ActiveScreen, cx: &mut Context<Self>) {
        self.active_screen = screen;
        cx.notify();
    }

    pub fn set_modal(&mut self, modal: ActiveModal, cx: &mut Context<Self>) {
        self.active_modal = modal;
        cx.notify();
    }

    pub fn toggle_command_palette(&mut self, cx: &mut Context<Self>) {
        self.active_modal = match self.active_modal {
            ActiveModal::CommandPalette => ActiveModal::None,
            _ => ActiveModal::CommandPalette,
        };
        cx.notify();
    }

    pub fn toggle_settings(&mut self, cx: &mut Context<Self>) {
        self.active_modal = match self.active_modal {
            ActiveModal::Settings => ActiveModal::None,
            _ => ActiveModal::Settings,
        };
        cx.notify();
    }

    pub fn close_modal(&mut self, cx: &mut Context<Self>) {
        if self.active_modal != ActiveModal::None {
            self.active_modal = ActiveModal::None;
            cx.notify();
        }
    }
}
