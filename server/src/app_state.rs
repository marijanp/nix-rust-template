use sqlx::SqlitePool;
use std::sync::Arc;

pub type AppState = Arc<AppConfig>;

#[derive(Debug)]
pub struct AppConfig {
    pub db_pool: SqlitePool,
}
