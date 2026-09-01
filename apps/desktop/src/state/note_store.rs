use crate::db::AppDb;
use gpui::*;
use tnotes_core::models::note::Note;

#[allow(dead_code)]
pub struct NoteStore {
    pub db: AppDb,
    pub notes: Vec<Note>,
    pub selected_note_id: Option<String>,
    pub search_query: String,
}

impl NoteStore {
    pub fn new(db: AppDb) -> Self {
        let notes = db.list_notes().unwrap_or_default();
        let selected_note_id = notes.first().map(|n| n.id.clone());

        Self {
            db,
            notes,
            selected_note_id,
            search_query: String::new(),
        }
    }

    pub fn selected_note(&self) -> Option<&Note> {
        let sel_id = self.selected_note_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == sel_id)
    }

    pub fn notes_in_folder(&self, folder_id: Option<&str>) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|n| match folder_id {
                Some(fid) => n.folder_id.as_deref() == Some(fid),
                None => true,
            })
            .cloned()
            .collect()
    }

    pub fn count_in_folder(&self, folder_id: Option<&str>) -> usize {
        self.notes
            .iter()
            .filter(|n| match folder_id {
                Some(fid) => n.folder_id.as_deref() == Some(fid),
                None => true,
            })
            .count()
    }

    #[allow(dead_code)]
    pub fn filtered_notes(&self) -> Vec<&Note> {
        if self.search_query.trim().is_empty() {
            self.notes.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            self.notes
                .iter()
                .filter(|n| {
                    n.title.to_lowercase().contains(&q) || n.body.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn select_note(&mut self, id: String, cx: &mut Context<Self>) {
        if self.selected_note_id.as_deref() != Some(&id) {
            self.selected_note_id = Some(id);
            cx.notify();
        }
    }

    pub fn create_note(
        &mut self,
        title: &str,
        body: &str,
        folder_id: Option<String>,
        cx: &mut Context<Self>,
    ) -> String {
        let note = Note::new(
            title,
            body,
            folder_id,
            &self.db.device_id,
            &self.db.default_user_id,
        );

        let note_id = note.id.clone();
        let _ = self.db.insert_note(&note);
        self.notes.insert(0, note);
        self.selected_note_id = Some(note_id.clone());
        cx.notify();
        note_id
    }

    #[allow(dead_code)]
    pub fn update_selected_note(&mut self, title: String, body: String, cx: &mut Context<Self>) {
        let Some(sel_id) = self.selected_note_id.clone() else {
            return;
        };
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == sel_id)
            && (note.title != title || note.body != body)
        {
            note.title = title;
            note.body = body;
            note.updated_at = tnotes_core::models::current_time_ms();
            note.version += 1;
            note.checksum = tnotes_core::models::compute_checksum(&note.body);
            let _ = self.db.update_note(note);
            cx.notify();
        }
    }

    #[allow(dead_code)]
    pub fn delete_selected_note(&mut self, cx: &mut Context<Self>) {
        let Some(sel_id) = self.selected_note_id.clone() else {
            return;
        };
        let _ = self.db.delete_note(&sel_id);
        self.notes.retain(|n| n.id != sel_id);
        self.selected_note_id = self.notes.first().map(|n| n.id.clone());
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query;
        cx.notify();
    }
}
