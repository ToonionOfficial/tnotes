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
    pub fn folder_by_id(&self, id: &str) -> Option<&Folder> {
        self.folders.iter().find(|f| f.id == id)
    }

    pub fn root_folders(&self) -> Vec<Folder> {
        self.folders
            .iter()
            .filter(|f| f.parent_id.is_none())
            .cloned()
            .collect()
    }

    pub fn subfolders_of(&self, parent_id: Option<&str>) -> Vec<Folder> {
        self.folders
            .iter()
            .filter(|f| f.parent_id.as_deref() == parent_id)
            .cloned()
            .collect()
    }

    pub fn ancestors_of(&self, folder_id: &str) -> Vec<Folder> {
        let mut path = Vec::new();
        let mut curr_id = Some(folder_id.to_string());
        while let Some(id) = curr_id {
            if let Some(folder) = self.folder_by_id(&id) {
                path.push(folder.clone());
                curr_id = folder.parent_id.clone();
            } else {
                break;
            }
        }
        path.reverse();
        path
    }

    pub fn root_ancestor_id(&self, folder_id: &str) -> Option<String> {
        self.ancestors_of(folder_id).first().map(|f| f.id.clone())
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
