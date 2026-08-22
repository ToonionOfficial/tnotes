pub struct Folder {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub icon: String,
    pub sort_order: i32,
    pub version: u64,
    pub updated_at: i64,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
    pub device_id: String,
}
