pub mod db;
pub mod default_tools;
pub mod domain;
pub mod error;
pub mod executor;
pub mod http;
pub mod mcp;
pub mod registry;

use std::{env, path::PathBuf};

use db::Database;
use error::AppError;
use registry::ToolRegistry;

const DATABASE_FILE_NAME: &str = "conjure.db";

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub registry: ToolRegistry,
}

impl AppState {
    pub async fn initialize(database_url: &str) -> Result<Self, AppError> {
        let db = Database::connect(database_url).await?;
        Self::initialize_database(db).await
    }

    async fn initialize_database(db: Database) -> Result<Self, AppError> {
        db.bootstrap().await?;

        let registry = ToolRegistry::new();
        registry.sync_from_db(&db).await?;

        Ok(Self { db, registry })
    }

    pub async fn initialize_from_env() -> Result<Self, AppError> {
        if let Ok(database_url) = env::var("CONJURE_DATABASE_URL") {
            return Self::initialize(&database_url).await;
        }

        let db = Database::connect_path(default_database_path()?).await?;
        Self::initialize_database(db).await
    }
}

fn default_database_path() -> Result<PathBuf, AppError> {
    platform_data_dir()
        .map(|data_dir| data_dir.join(DATABASE_FILE_NAME))
        .ok_or_else(|| AppError::Internal("could not determine Conjure data directory".to_string()))
}

#[cfg(target_os = "macos")]
fn platform_data_dir() -> Option<PathBuf> {
    env_path("HOME").map(|home| {
        home.join("Library")
            .join("Application Support")
            .join("Conjure")
    })
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> Option<PathBuf> {
    env_path("APPDATA").map(|data_dir| data_dir.join("Conjure"))
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn platform_data_dir() -> Option<PathBuf> {
    env_path("XDG_DATA_HOME")
        .or_else(|| env_path("HOME").map(|home| home.join(".local").join("share")))
        .map(|data_dir| data_dir.join("conjure"))
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
