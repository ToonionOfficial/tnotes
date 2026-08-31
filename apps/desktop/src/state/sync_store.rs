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
    pub pending_mutations_count: usize,
}

impl SyncStore {
    pub fn new() -> Self {
        Self {
            status: SyncStatus::Idle,
            pending_mutations_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn set_status(&mut self, status: SyncStatus, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    #[allow(dead_code)]
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
