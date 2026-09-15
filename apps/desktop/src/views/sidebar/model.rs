use std::collections::HashSet;
use gpui::*;
use tnotes_core::models::note::Note;
use crate::components::{InputState, UniformListScrollHandle};
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
    pub(crate) tree_scroll_handle: UniformListScrollHandle,
    pub(crate) search_scroll_handle: UniformListScrollHandle,
    pub(crate) tree_rows: Vec<TreeRow>,
    pub(crate) search_rows: Vec<Note>,
    pub(crate) _store_subscription: Subscription,
}

#[derive(Clone, Debug)]
pub enum TreeRow {
    Folder {
        id: String,
        name: String,
        icon: String,
        depth: usize,
        expanded: bool,
        count: usize,
    },
    Note {
        id: String,
        title: String,
        depth: usize,
        is_active: bool,
        is_pinned: bool,
    },
}

impl TreeRow {
    #[allow(dead_code)]
    pub fn id(&self) -> &str {
        match self {
            Self::Folder { id, .. } => id,
            Self::Note { id, .. } => id,
        }
    }
}

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
