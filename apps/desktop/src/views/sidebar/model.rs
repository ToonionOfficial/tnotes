use crate::components::{InputState, UniformListScrollHandle};
use crate::store::NoteStore;
use gpui::*;
use std::collections::HashSet;
use tnotes_core::models::note::Note;

pub struct SidebarView {
    pub(crate) store: Entity<NoteStore>,
    pub(crate) is_collapsed: bool,
    pub(crate) search_state: InputState,
    pub(crate) search_focus: FocusHandle,
    pub(crate) expanded_folders: HashSet<String>,
    pub(crate) context_menu: Option<SidebarContextMenu>,
    pub(crate) context_menu_focus: FocusHandle,
    pub(crate) renaming: Option<RenameState>,
    pub(crate) picking_icon_for: Option<FolderIconPickerState>,
    pub(crate) tree_scroll_handle: UniformListScrollHandle,
    pub(crate) search_scroll_handle: UniformListScrollHandle,
    pub(crate) tree_rows: Vec<TreeRow>,
    pub(crate) tree_rows_dirty: bool,
    pub(crate) search_rows: Vec<Note>,
    pub(crate) last_search_query: String,
    pub(crate) scrollbar_drag_offset: Option<Pixels>,
    pub(crate) updater: Option<Entity<crate::updater::UpdateManager>>,
    pub(crate) _store_subscription: Subscription,
    pub(crate) _updater_subscription: Option<Subscription>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FolderIconPickerState {
    pub folder_id: String,
    pub position: Point<Pixels>,
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
        note_index: usize,
        depth: usize,
    },
}

impl TreeRow {
    #[allow(dead_code)]
    pub fn id<'a>(&'a self, notes: &'a [Note]) -> Option<&'a str> {
        match self {
            Self::Folder { id, .. } => Some(id.as_str()),
            Self::Note { note_index, .. } => notes.get(*note_index).map(|n| n.id.as_str()),
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
    Folder {
        id: String,
        name: String,
    },
    Note {
        id: String,
        title: String,
        is_pinned: bool,
    },
}

#[derive(Clone, Debug)]
pub struct SidebarContextMenu {
    pub(crate) position: Point<Pixels>,
    pub(crate) target: SidebarContextTarget,
}
