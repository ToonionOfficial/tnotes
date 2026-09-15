use std::collections::HashSet;
use gpui::*;
use crate::components::InputState;
use crate::store::NoteStore;

pub struct SidebarView {
    pub(crate) store: Entity<NoteStore>,
    pub(crate) is_collapsed: bool,
    pub(crate) search_state: InputState,
    pub(crate) search_focus: FocusHandle,
    pub(crate) expanded_folders: HashSet<String>,
    pub(crate) context_menu: Option<SidebarContextMenu>,
    pub(crate) context_menu_focus: FocusHandle,
    pub(crate) renaming: Option<RenameState>,
    pub(crate) picking_icon_for: Option<String>,
    pub(crate) _store_subscription: Subscription,
}

/// Inline rename session for one folder or note row.
#[derive(Clone, Debug)]
pub struct RenameState {
    pub(crate) kind: RenameKind,
    pub(crate) id: String,
    pub(crate) input: InputState,
    pub(crate) focus: FocusHandle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameKind {
    Folder,
    Note,
}

/// Right-click target for the sidebar context menu.
#[derive(Clone, Debug)]
pub enum SidebarContextTarget {
    Folder { id: String, name: String },
    Note { id: String, title: String, is_pinned: bool },
}

#[derive(Clone, Debug)]
pub struct SidebarContextMenu {
    pub(crate) position: Point<Pixels>,
    pub(crate) target: SidebarContextTarget,
}
