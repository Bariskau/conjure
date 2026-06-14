use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::{
    db::Database,
    domain::{Tool, input_schema},
    error::AppError,
};

#[derive(Debug, Clone, Serialize)]
pub struct RegisteredTool {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, RegisteredTool>>>,
    notifications: broadcast::Sender<RegistryNotification>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryNotification {
    ToolsListChanged,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let (notifications, _) = broadcast::channel(64);

        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            notifications,
        }
    }

    pub async fn sync_from_db(&self, db: &Database) -> Result<(), AppError> {
        let tools = db.list_enabled_tools().await?;
        let next_registry = tools
            .iter()
            .map(registered_tool_from_tool)
            .map(|tool| (tool.name.clone(), tool))
            .collect();

        let mut registry = self.tools.write().await;
        *registry = next_registry;

        Ok(())
    }

    pub async fn list_tools(&self) -> Vec<RegisteredTool> {
        let registry = self.tools.read().await;
        let mut tools = registry.values().cloned().collect::<Vec<_>>();
        tools.sort_by(|left, right| left.name.cmp(&right.name));
        tools
    }

    pub async fn get_by_name(&self, name: &str) -> Option<RegisteredTool> {
        let registry = self.tools.read().await;
        registry.get(name).cloned()
    }

    pub fn notify_tools_changed(&self) {
        let _ = self
            .notifications
            .send(RegistryNotification::ToolsListChanged);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RegistryNotification> {
        self.notifications.subscribe()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn registered_tool_from_tool(tool: &Tool) -> RegisteredTool {
    RegisteredTool {
        id: tool.id,
        name: tool.name.clone(),
        description: tool.description.clone(),
        input_schema: input_schema(tool),
    }
}
