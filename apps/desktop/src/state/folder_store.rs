use crate::db::AppDb;
use gpui::*;
use tnotes_core::models::folder::Folder;

#[allow(dead_code)]
pub struct FolderStore {
    pub db: AppDb,
    pub folders: Vec<Folder>,
    pub selected_folder_id: Option<String>,
}

impl FolderStore {
    pub fn new(db: AppDb) -> Self {
        let folders = db.list_folders().unwrap_or_default();
        Self {
            db,
            folders,
            selected_folder_id: None,
        }
    }

    #[allow(dead_code)]
    pub fn select_folder(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.selected_folder_id = id;
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn create_folder(
        &mut self,
        name: &str,
        parent_id: Option<String>,
        cx: &mut Context<Self>,
    ) -> String {
        let sort_order = self.folders.len() as i32;
        let folder = Folder::new(
            name,
            None,
            parent_id,
            sort_order,
            &self.db.device_id,
            &self.db.default_user_id,
        );

        let folder_id = folder.id.clone();
        let _ = self.db.insert_folder(&folder);
        self.folders.push(folder);
        cx.notify();
        folder_id
    }
}
