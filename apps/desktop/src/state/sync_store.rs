use gpui::*;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Synced { last_synced_at: i64 },
    Offline,
    Error(String),
}

#[allow(dead_code)]
pub struct SyncStore {
    pub status: SyncStatus,
    pub auto_sync: bool,
    pub pending_mutations_count: usize,
}

#[allow(dead_code)]
impl SyncStore {
    pub fn new() -> Self {
        Self {
            status: SyncStatus::Idle,
            auto_sync: true,
            pending_mutations_count: 0,
        }
    }

    pub fn set_status(&mut self, status: SyncStatus, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    pub fn toggle_auto_sync(&mut self, cx: &mut Context<Self>) {
        self.auto_sync = !self.auto_sync;
        cx.notify();
    }

    pub fn set_pending_count(&mut self, count: usize, cx: &mut Context<Self>) {
        self.pending_mutations_count = count;
        cx.notify();
    }
}

impl Default for SyncStore {
    fn default() -> Self {
        Self::new()
    }
}
