use gpui::Context;
use tnotes_core::models::note::Note;

use super::note_store::NoteStore;

const MAX_HISTORY_SIZE: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationLocation {
    Note(String),
    Starred,
    Trash,
}

impl NavigationLocation {
    pub fn as_note_id(&self) -> Option<&str> {
        match self {
            Self::Note(id) => Some(id.as_str()),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Note(id) => id.as_str(),
            Self::Starred => "starred",
            Self::Trash => "trash",
        }
    }
}

impl std::ops::Deref for NavigationLocation {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for NavigationLocation {
    fn from(s: &str) -> Self {
        match s {
            "starred" => Self::Starred,
            "trash" => Self::Trash,
            other => Self::Note(other.to_string()),
        }
    }
}

impl From<String> for NavigationLocation {
    fn from(s: String) -> Self {
        match s.as_str() {
            "starred" => Self::Starred,
            "trash" => Self::Trash,
            _ => Self::Note(s),
        }
    }
}

impl From<&NavigationLocation> for NavigationLocation {
    fn from(loc: &NavigationLocation) -> Self {
        loc.clone()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavigationHistory {
    pub back_stack: Vec<NavigationLocation>,
    pub forward_stack: Vec<NavigationLocation>,
}

impl NavigationHistory {
    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    pub fn push(
        &mut self,
        current: Option<impl Into<NavigationLocation>>,
        next: impl Into<NavigationLocation>,
    ) {
        let next = next.into();
        if let Some(cur) = current {
            let cur = cur.into();
            if cur != next {
                self.back_stack.push(cur);
                if self.back_stack.len() > MAX_HISTORY_SIZE {
                    let excess = self.back_stack.len() - MAX_HISTORY_SIZE;
                    self.back_stack.drain(..excess);
                }
                self.forward_stack.clear();
            }
        }
    }

    pub fn go_back(
        &mut self,
        current: Option<impl Into<NavigationLocation>>,
    ) -> Option<NavigationLocation> {
        let prev = self.back_stack.pop()?;
        if let Some(cur) = current {
            self.forward_stack.push(cur.into());
            if self.forward_stack.len() > MAX_HISTORY_SIZE {
                let excess = self.forward_stack.len() - MAX_HISTORY_SIZE;
                self.forward_stack.drain(..excess);
            }
        }
        Some(prev)
    }

    pub fn go_forward(
        &mut self,
        current: Option<impl Into<NavigationLocation>>,
    ) -> Option<NavigationLocation> {
        let next = self.forward_stack.pop()?;
        if let Some(cur) = current {
            self.back_stack.push(cur.into());
            if self.back_stack.len() > MAX_HISTORY_SIZE {
                let excess = self.back_stack.len() - MAX_HISTORY_SIZE;
                self.back_stack.drain(..excess);
            }
        }
        Some(next)
    }

    pub fn remove_note(&mut self, note_id: &str) {
        let target = NavigationLocation::Note(note_id.to_string());
        self.back_stack.retain(|id| id != &target);
        self.forward_stack.retain(|id| id != &target);
    }
}

impl NoteStore {
    pub fn selected_note_id(&self) -> Option<String> {
        self.active_location.as_note_id().map(|s| s.to_string())
    }

    pub fn selected_note(&self) -> Option<Note> {
        let id = self.selected_note_id()?;
        self.notes.iter().find(|n| n.id == id).cloned()
    }

    pub fn open_starred(&mut self, cx: &mut Context<Self>) {
        let next = NavigationLocation::Starred;
        if self.active_location == next {
            return;
        }
        self.history.push(Some(&self.active_location), next.clone());
        self.active_location = next;
        cx.notify();
    }

    pub fn open_trash(&mut self, cx: &mut Context<Self>) {
        let next = NavigationLocation::Trash;
        if self.active_location == next {
            return;
        }
        self.history.push(Some(&self.active_location), next.clone());
        self.active_location = next;
        cx.notify();
    }

    pub fn select_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        let next = NavigationLocation::Note(note_id.to_string());
        if self.active_location == next {
            return;
        }
        self.history.push(Some(&self.active_location), next.clone());
        self.active_location = next;
        cx.notify();
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(prev) = self.history.go_back(Some(&self.active_location)) {
            self.active_location = prev;
            cx.notify();
            true
        } else {
            false
        }
    }

    pub fn navigate_forward(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(next) = self.history.go_forward(Some(&self.active_location)) {
            self.active_location = next;
            cx.notify();
            true
        } else {
            false
        }
    }

    pub fn can_navigate_back(&self) -> bool {
        self.history.can_go_back()
    }

    pub fn can_navigate_forward(&self) -> bool {
        self.history.can_go_forward()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn history_caps_at_50_entries() {
        let mut history = NavigationHistory::default();
        for i in 0..60 {
            history.push(
                Some(format!("note-{i}")),
                NavigationLocation::Note(format!("note-{}", i + 1)),
            );
        }
        assert_eq!(history.back_stack.len(), 50);
    }

    #[test]
    fn navigation_history_stack_operations() {
        let mut history = NavigationHistory::default();
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(Some("note-a"), "note-b");
        history.push(Some("note-b"), "note-c");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(Some("note-c"), "note-c");
        assert_eq!(history.back_stack.len(), 2);

        let prev = history.go_back(Some("note-c"));
        assert_eq!(prev.as_deref(), Some("note-b"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        let prev = history.go_back(Some("note-b"));
        assert_eq!(prev.as_deref(), Some("note-a"));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        let next = history.go_forward(Some("note-a"));
        assert_eq!(next.as_deref(), Some("note-b"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        history.push(Some("note-b"), "note-d");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.remove_note("note-a");
        assert_eq!(
            history.back_stack,
            vec![NavigationLocation::Note("note-b".to_string())]
        );
    }
}
