use gpui::*;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStatus {
    LocalOnly,
    Connected {
        server_url: String,
        username: String,
        email: String,
    },
    Connecting,
    Error(String),
}

#[allow(dead_code)]
pub struct AuthStore {
    pub status: AuthStatus,
    pub user_id: String,
    pub device_id: String,
    pub server_url: String,
}

impl AuthStore {
    pub fn new(user_id: String, device_id: String) -> Self {
        Self {
            status: AuthStatus::LocalOnly,
            user_id,
            device_id,
            server_url: "http://localhost:3000".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn set_connected(
        &mut self,
        server_url: String,
        username: String,
        email: String,
        cx: &mut Context<Self>,
    ) {
        self.status = AuthStatus::Connected {
            server_url,
            username,
            email,
        };
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_local_only(&mut self, cx: &mut Context<Self>) {
        self.status = AuthStatus::LocalOnly;
        cx.notify();
    }
}
