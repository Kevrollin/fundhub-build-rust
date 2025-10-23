use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::services::{stellar::StellarService, NewStellarService};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub stellar: StellarService,
    pub stellar_service: NewStellarService,
    pub notifier: broadcast::Sender<String>,
}


