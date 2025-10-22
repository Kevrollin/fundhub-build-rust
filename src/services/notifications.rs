use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct NotificationsService;

#[derive(Debug, Clone)]
pub enum NotificationType { Realtime, Email }

#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub user_id: Option<uuid::Uuid>,
    pub message: String,
    pub ntype: NotificationType,
    pub created_at: DateTime<Utc>,
}

impl NotificationsService {
    pub fn new() -> Self { Self }

    pub async fn send_realtime(&self, _msg: &NotificationMessage) {
        // TODO: integrate SSE/WebSocket
    }

    pub async fn send_email(&self, _msg: &NotificationMessage) {
        // TODO: integrate email provider
    }
}


