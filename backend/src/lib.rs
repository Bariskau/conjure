pub mod db;
pub mod default_tools;
pub mod domain;
pub mod error;
pub mod executor;
pub mod http;
pub mod mcp;
pub mod registry;

use db::Database;
use registry::ToolRegistry;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub registry: ToolRegistry,
}

impl AppState {
    pub async fn initialize(database_url: &str) -> Result<Self, error::AppError> {
        let db = Database::connect(database_url).await?;
        db.bootstrap().await?;

        let registry = ToolRegistry::new();
        registry.sync_from_db(&db).await?;

        Ok(Self { db, registry })
    }

    pub async fn initialize_from_env() -> Result<Self, error::AppError> {
        let database_url = std::env::var("CONJURE_DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://conjure.db?mode=rwc".to_string());

        Self::initialize(&database_url).await
    }
}
