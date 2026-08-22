pub struct Note {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub trashed: bool,
    pub version: u64,
    pub updated_at: i64,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
    pub device_id: String,
    pub checksum: String,
}
