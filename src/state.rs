use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::services::stellar::StellarService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub stellar: StellarService,
    pub notifier: broadcast::Sender<String>,
}


