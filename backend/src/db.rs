use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{AssertSqlSafe, Row, SqlitePool, sqlite::SqlitePoolOptions};
use uuid::Uuid;

use crate::{
    default_tools::{DefaultToolSpec, default_tools},
    domain::{
        AppSettings, CallLog, ExecutionStatus, ExportedParameter, ExportedTool, LogFilter,
        NewCallLog, NewParameter, NewTool, Tool, ToolImportResult, ToolParameter, ToolSummary,
        ToolsExport, UpdateParameter, UpdateSettings, UpdateTool, ValidationRules,
        default_mcp_endpoint, default_settings, normalize_optional_text, sanitize_category,
        sanitize_settings, validate_new_tool, validate_parameter, validate_parameter_update,
        validate_update_tool,
    },
    error::AppError,
};

const SCHEMA: &[&str] = &[
    "PRAGMA foreign_keys = ON",
    "CREATE TABLE IF NOT EXISTS tools (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        description TEXT NOT NULL,
        category TEXT,
        script_body TEXT,
        script_path TEXT,
        working_dir TEXT,
        working_dir_expose INTEGER NOT NULL DEFAULT 0,
        working_dir_required INTEGER NOT NULL DEFAULT 0,
        timeout_ms INTEGER NOT NULL,
        enabled INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS tool_parameters (
        id TEXT PRIMARY KEY,
        tool_id TEXT NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        type TEXT NOT NULL,
        description TEXT,
        required INTEGER NOT NULL,
        default_value TEXT,
        validation_json TEXT NOT NULL,
        position INTEGER NOT NULL,
        UNIQUE(tool_id, name)
    )",
    "CREATE TABLE IF NOT EXISTS call_logs (
        id TEXT PRIMARY KEY,
        tool_id TEXT REFERENCES tools(id) ON DELETE SET NULL,
        tool_name TEXT NOT NULL,
        source TEXT NOT NULL,
        params_json TEXT NOT NULL,
        stdout TEXT NOT NULL,
        stderr TEXT NOT NULL,
        exit_code INTEGER,
        started_at TEXT NOT NULL,
        finished_at TEXT NOT NULL,
        duration_ms INTEGER NOT NULL,
        status TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS app_settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        default_working_dir TEXT,
        allowed_base_paths_json TEXT NOT NULL,
        default_timeout_ms INTEGER NOT NULL DEFAULT 30000,
        mcp_endpoint TEXT NOT NULL DEFAULT ''
    )",
    "CREATE TABLE IF NOT EXISTS tool_categories (
        name TEXT PRIMARY KEY COLLATE NOCASE,
        created_at TEXT NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS idx_call_logs_tool_id ON call_logs(tool_id)",
    "CREATE INDEX IF NOT EXISTS idx_call_logs_status ON call_logs(status)",
    "CREATE INDEX IF NOT EXISTS idx_call_logs_finished_at ON call_logs(finished_at)",
];

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, AppError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn bootstrap(&self) -> Result<(), AppError> {
        for statement in SCHEMA {
            sqlx::query(*statement).execute(&self.pool).await?;
        }

        self.migrate_tools_schema().await?;
        self.migrate_settings_schema().await?;
        self.seed_settings().await?;
        self.seed_default_tools().await?;
        self.sync_categories_from_tools().await
    }

    pub async fn create_tool(&self, payload: NewTool) -> Result<Tool, AppError> {
        validate_new_tool(&payload)?;

        let id = Uuid::new_v4();
        let now = Utc::now();
        let category = sanitize_category(payload.category);
        let working_dir = normalize_optional_text(payload.working_dir.as_deref());
        self.ensure_category(category.as_deref()).await?;

        sqlx::query(
            "INSERT INTO tools (
                id, name, description, category, script_body, script_path,
                working_dir, working_dir_expose, working_dir_required,
                timeout_ms, enabled, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .bind(id.to_string())
        .bind(payload.name)
        .bind(payload.description)
        .bind(category)
        .bind(payload.script_body)
        .bind(payload.script_path)
        .bind(working_dir)
        .bind(i64::from(payload.working_dir_expose))
        .bind(i64::from(payload.working_dir_required))
        .bind(payload.timeout_ms)
        .bind(i64::from(payload.enabled))
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.get_tool(id)
            .await?
            .ok_or_else(|| AppError::Internal("created tool could not be loaded".to_string()))
    }

    pub async fn list_tools(&self) -> Result<Vec<ToolSummary>, AppError> {
        let rows = sqlx::query(
            "SELECT
                t.id, t.name, t.description, t.category, t.working_dir,
                t.working_dir_expose, t.working_dir_required, t.timeout_ms, t.enabled,
                MAX(l.finished_at) AS last_run_at
            FROM tools t
            LEFT JOIN call_logs l ON l.tool_id = t.id
            GROUP BY t.id
            ORDER BY t.updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(tool_summary_from_row).collect()
    }

    pub async fn list_enabled_tools(&self) -> Result<Vec<Tool>, AppError> {
        let rows = sqlx::query(
            "SELECT
                t.*, MAX(l.finished_at) AS last_run_at
            FROM tools t
            LEFT JOIN call_logs l ON l.tool_id = t.id
            WHERE t.enabled = 1
            GROUP BY t.id
            ORDER BY t.name",
        )
        .fetch_all(&self.pool)
        .await?;

        self.tools_from_rows(rows).await
    }

    pub async fn export_tools(&self) -> Result<ToolsExport, AppError> {
        let rows = sqlx::query(
            "SELECT
                t.*, MAX(l.finished_at) AS last_run_at
            FROM tools t
            LEFT JOIN call_logs l ON l.tool_id = t.id
            GROUP BY t.id
            ORDER BY LOWER(t.name)",
        )
        .fetch_all(&self.pool)
        .await?;
        let tools = self.tools_from_rows(rows).await?;

        Ok(ToolsExport {
            version: 1,
            exported_at: Utc::now(),
            categories: self.list_categories().await?,
            tools: tools.iter().map(exported_tool_from_tool).collect(),
        })
    }

    pub async fn import_tools(&self, payload: ToolsExport) -> Result<ToolImportResult, AppError> {
        if payload.version != 1 {
            return Err(AppError::Validation(format!(
                "unsupported tools export version `{}`",
                payload.version
            )));
        }

        let imported_categories = self.import_categories(payload.categories).await?;
        let mut imported_tools = 0;
        for tool in payload.tools {
            self.import_tool(tool).await?;
            imported_tools += 1;
        }

        Ok(ToolImportResult {
            imported_tools,
            imported_categories,
        })
    }

    pub async fn get_tool(&self, id: Uuid) -> Result<Option<Tool>, AppError> {
        let row = sqlx::query(
            "SELECT
                t.*, MAX(l.finished_at) AS last_run_at
            FROM tools t
            LEFT JOIN call_logs l ON l.tool_id = t.id
            WHERE t.id = ?1
            GROUP BY t.id",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let mut tool = tool_from_row(&row)?;
        tool.parameters = self.list_parameters(id).await?;

        Ok(Some(tool))
    }

    pub async fn get_tool_by_name(&self, name: &str) -> Result<Option<Tool>, AppError> {
        let row = sqlx::query(
            "SELECT
                t.*, MAX(l.finished_at) AS last_run_at
            FROM tools t
            LEFT JOIN call_logs l ON l.tool_id = t.id
            WHERE t.name = ?1
            GROUP BY t.id",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id = parse_uuid(row.try_get::<String, _>("id")?.as_str())?;
        let mut tool = tool_from_row(&row)?;
        tool.parameters = self.list_parameters(id).await?;

        Ok(Some(tool))
    }

    pub async fn update_tool(&self, id: Uuid, payload: UpdateTool) -> Result<Tool, AppError> {
        validate_update_tool(&payload)?;

        let current = self
            .get_tool(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("tool `{id}`")))?;

        let name = payload.name.unwrap_or(current.name);
        let description = payload.description.unwrap_or(current.description);
        let category = match payload.category {
            Some(category) => sanitize_category(Some(category)),
            None => current.category,
        };
        let script_body = payload.script_body.or(current.script_body);
        let script_path = payload.script_path.or(current.script_path);
        let working_dir = match payload.working_dir {
            Some(working_dir) => normalize_optional_text(Some(&working_dir)),
            None => current.working_dir,
        };
        let working_dir_expose = payload
            .working_dir_expose
            .unwrap_or(current.working_dir_expose);
        let working_dir_required = payload
            .working_dir_required
            .unwrap_or(current.working_dir_required);
        let timeout_ms = payload.timeout_ms.unwrap_or(current.timeout_ms);
        let enabled = payload.enabled.unwrap_or(current.enabled);
        let updated_at = Utc::now();
        self.ensure_category(category.as_deref()).await?;

        sqlx::query(
            "UPDATE tools
            SET name = ?2, description = ?3, category = ?4, script_body = ?5,
                script_path = ?6, working_dir = ?7, working_dir_expose = ?8,
                working_dir_required = ?9, timeout_ms = ?10, enabled = ?11, updated_at = ?12
            WHERE id = ?1",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(description)
        .bind(category)
        .bind(script_body)
        .bind(script_path)
        .bind(working_dir)
        .bind(i64::from(working_dir_expose))
        .bind(i64::from(working_dir_required))
        .bind(timeout_ms)
        .bind(i64::from(enabled))
        .bind(updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.get_tool(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("tool `{id}`")))
    }

    pub async fn delete_tool(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM tools WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("tool `{id}`")));
        }

        Ok(())
    }

    pub async fn set_tool_enabled(&self, id: Uuid, enabled: bool) -> Result<Tool, AppError> {
        let updated_at = Utc::now();
        let result = sqlx::query("UPDATE tools SET enabled = ?2, updated_at = ?3 WHERE id = ?1")
            .bind(id.to_string())
            .bind(i64::from(enabled))
            .bind(updated_at.to_rfc3339())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("tool `{id}`")));
        }

        self.get_tool(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("tool `{id}`")))
    }

    pub async fn create_parameter(
        &self,
        tool_id: Uuid,
        payload: NewParameter,
    ) -> Result<ToolParameter, AppError> {
        validate_parameter(&payload)?;
        self.require_tool(tool_id).await?;

        let id = Uuid::new_v4();
        let validation_json = serde_json::to_string(&payload.validation)?;
        let default_value = payload
            .default_value
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        sqlx::query(
            "INSERT INTO tool_parameters (
                id, tool_id, name, type, description, required,
                default_value, validation_json, position
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(id.to_string())
        .bind(tool_id.to_string())
        .bind(payload.name)
        .bind(payload.parameter_type.as_str())
        .bind(payload.description)
        .bind(i64::from(payload.required))
        .bind(default_value)
        .bind(validation_json)
        .bind(payload.position)
        .execute(&self.pool)
        .await?;

        self.touch_tool(tool_id).await?;
        self.get_parameter(id)
            .await?
            .ok_or_else(|| AppError::Internal("created parameter could not be loaded".to_string()))
    }

    pub async fn list_parameters(&self, tool_id: Uuid) -> Result<Vec<ToolParameter>, AppError> {
        let rows =
            sqlx::query("SELECT * FROM tool_parameters WHERE tool_id = ?1 ORDER BY position, name")
                .bind(tool_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        rows.iter().map(parameter_from_row).collect()
    }

    pub async fn get_parameter(&self, id: Uuid) -> Result<Option<ToolParameter>, AppError> {
        let row = sqlx::query("SELECT * FROM tool_parameters WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        row.as_ref().map(parameter_from_row).transpose()
    }

    pub async fn update_parameter(
        &self,
        tool_id: Uuid,
        parameter_id: Uuid,
        payload: UpdateParameter,
    ) -> Result<ToolParameter, AppError> {
        validate_parameter_update(&payload)?;

        let current = self
            .get_parameter(parameter_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("parameter `{parameter_id}`")))?;

        if current.tool_id != tool_id {
            return Err(AppError::NotFound(format!(
                "parameter `{parameter_id}` for tool `{tool_id}`"
            )));
        }

        let parameter_type = payload.parameter_type.unwrap_or(current.parameter_type);
        let validation = payload.validation.unwrap_or(current.validation);
        let validation_json = serde_json::to_string(&validation)?;
        let default_value = payload
            .default_value
            .or(current.default_value)
            .map(|value| serde_json::to_string(&value))
            .transpose()?;

        sqlx::query(
            "UPDATE tool_parameters
            SET name = ?3, type = ?4, description = ?5, required = ?6,
                default_value = ?7, validation_json = ?8, position = ?9
            WHERE id = ?1 AND tool_id = ?2",
        )
        .bind(parameter_id.to_string())
        .bind(tool_id.to_string())
        .bind(payload.name.unwrap_or(current.name))
        .bind(parameter_type.as_str())
        .bind(payload.description.or(current.description))
        .bind(i64::from(payload.required.unwrap_or(current.required)))
        .bind(default_value)
        .bind(validation_json)
        .bind(payload.position.unwrap_or(current.position))
        .execute(&self.pool)
        .await?;

        self.touch_tool(tool_id).await?;
        self.get_parameter(parameter_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("parameter `{parameter_id}`")))
    }

    pub async fn delete_parameter(
        &self,
        tool_id: Uuid,
        parameter_id: Uuid,
    ) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM tool_parameters WHERE id = ?1 AND tool_id = ?2")
            .bind(parameter_id.to_string())
            .bind(tool_id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("parameter `{parameter_id}`")));
        }

        self.touch_tool(tool_id).await
    }

    pub async fn insert_call_log(&self, payload: NewCallLog) -> Result<CallLog, AppError> {
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO call_logs (
                id, tool_id, tool_name, source, params_json, stdout, stderr,
                exit_code, started_at, finished_at, duration_ms, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(id.to_string())
        .bind(payload.tool_id.map(|tool_id| tool_id.to_string()))
        .bind(payload.tool_name)
        .bind(payload.source.as_str())
        .bind(serde_json::to_string(&payload.params_json)?)
        .bind(payload.stdout)
        .bind(payload.stderr)
        .bind(payload.exit_code)
        .bind(payload.started_at.to_rfc3339())
        .bind(payload.finished_at.to_rfc3339())
        .bind(payload.duration_ms)
        .bind(payload.status.as_str())
        .execute(&self.pool)
        .await?;

        self.get_call_log(id)
            .await?
            .ok_or_else(|| AppError::Internal("created call log could not be loaded".to_string()))
    }

    pub async fn list_call_logs(&self, filter: LogFilter) -> Result<Vec<CallLog>, AppError> {
        let rows = sqlx::query("SELECT * FROM call_logs ORDER BY started_at DESC LIMIT 250")
            .fetch_all(&self.pool)
            .await?;

        let mut logs = Vec::with_capacity(rows.len());
        for row in rows {
            let log = call_log_from_row(&row)?;
            if log_matches_filter(&log, &filter)? {
                logs.push(log);
            }
        }

        Ok(logs)
    }

    pub async fn list_categories(&self) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query(
            "SELECT name FROM (
                SELECT name FROM tool_categories
                UNION
                SELECT DISTINCT category AS name FROM tools
                WHERE category IS NOT NULL AND TRIM(category) != ''
            )
            ORDER BY LOWER(name)",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter()
            .map(|row| row.try_get("name"))
            .collect::<Result<Vec<String>, _>>()
            .map_err(AppError::from)
    }

    pub async fn create_category(&self, name: String) -> Result<String, AppError> {
        let category = category_name_from_input(name)?;
        self.ensure_category(Some(&category)).await?;

        Ok(category)
    }

    pub async fn rename_category(
        &self,
        previous_name: String,
        next_name: String,
    ) -> Result<String, AppError> {
        let previous_category = category_name_from_input(previous_name)?;
        let next_category = category_name_from_input(next_name)?;

        if same_text(&previous_category, &next_category) {
            self.ensure_category(Some(&next_category)).await?;
            return Ok(next_category);
        }

        self.ensure_category(Some(&next_category)).await?;
        sqlx::query(
            "UPDATE tools
            SET category = ?1, updated_at = ?2
            WHERE category = ?3 COLLATE NOCASE",
        )
        .bind(&next_category)
        .bind(Utc::now().to_rfc3339())
        .bind(&previous_category)
        .execute(&self.pool)
        .await?;

        sqlx::query("DELETE FROM tool_categories WHERE name = ?1 COLLATE NOCASE")
            .bind(previous_category)
            .execute(&self.pool)
            .await?;

        Ok(next_category)
    }

    pub async fn delete_category(&self, name: String) -> Result<(), AppError> {
        let category = category_name_from_input(name)?;

        sqlx::query("DELETE FROM tool_categories WHERE name = ?1 COLLATE NOCASE")
            .bind(&category)
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "UPDATE tools
            SET category = NULL, updated_at = ?1
            WHERE category = ?2 COLLATE NOCASE",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(category)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_settings(&self) -> Result<AppSettings, AppError> {
        let row = sqlx::query(
            "SELECT
                default_working_dir, allowed_base_paths_json, default_timeout_ms, mcp_endpoint
            FROM app_settings
            WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => settings_from_row(&row),
            None => Ok(default_settings()),
        }
    }

    pub async fn update_settings(&self, payload: UpdateSettings) -> Result<AppSettings, AppError> {
        let settings = sanitize_settings(payload)?;
        let allowed_base_paths_json = serde_json::to_string(&settings.allowed_base_paths)?;

        sqlx::query(
            "INSERT INTO app_settings (
                id, default_working_dir, allowed_base_paths_json, default_timeout_ms, mcp_endpoint
            )
            VALUES (1, ?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                default_working_dir = excluded.default_working_dir,
                allowed_base_paths_json = excluded.allowed_base_paths_json,
                default_timeout_ms = excluded.default_timeout_ms,
                mcp_endpoint = excluded.mcp_endpoint",
        )
        .bind(settings.default_working_dir)
        .bind(allowed_base_paths_json)
        .bind(settings.default_timeout_ms)
        .bind(settings.mcp_endpoint)
        .execute(&self.pool)
        .await?;

        self.get_settings().await
    }

    async fn import_categories(&self, categories: Vec<String>) -> Result<usize, AppError> {
        let mut imported_categories = 0;
        for category in categories {
            if normalize_optional_text(Some(&category)).is_some() {
                self.create_category(category).await?;
                imported_categories += 1;
            }
        }

        Ok(imported_categories)
    }

    async fn import_tool(&self, tool: ExportedTool) -> Result<(), AppError> {
        let payload = new_tool_from_exported_tool(&tool);
        validate_new_tool(&payload)?;
        validate_exported_parameters(&tool.parameters)?;

        let imported_tool = match self.get_tool_by_name(&tool.name).await? {
            Some(existing_tool) => self.update_imported_tool(existing_tool.id, payload).await?,
            None => self.create_tool(payload).await?,
        };

        self.replace_parameters_from_export(imported_tool.id, &tool.parameters)
            .await
    }

    async fn update_imported_tool(&self, id: Uuid, payload: NewTool) -> Result<Tool, AppError> {
        validate_new_tool(&payload)?;

        let category = sanitize_category(payload.category);
        let working_dir = normalize_optional_text(payload.working_dir.as_deref());
        let updated_at = Utc::now();
        self.ensure_category(category.as_deref()).await?;

        let result = sqlx::query(
            "UPDATE tools
            SET name = ?2, description = ?3, category = ?4, script_body = ?5,
                script_path = ?6, working_dir = ?7, working_dir_expose = ?8,
                working_dir_required = ?9, timeout_ms = ?10, enabled = ?11, updated_at = ?12
            WHERE id = ?1",
        )
        .bind(id.to_string())
        .bind(payload.name)
        .bind(payload.description)
        .bind(category)
        .bind(payload.script_body)
        .bind(payload.script_path)
        .bind(working_dir)
        .bind(i64::from(payload.working_dir_expose))
        .bind(i64::from(payload.working_dir_required))
        .bind(payload.timeout_ms)
        .bind(i64::from(payload.enabled))
        .bind(updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("tool `{id}`")));
        }

        self.get_tool(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("tool `{id}`")))
    }

    async fn replace_parameters_from_export(
        &self,
        tool_id: Uuid,
        parameters: &[ExportedParameter],
    ) -> Result<(), AppError> {
        validate_exported_parameters(parameters)?;

        sqlx::query("DELETE FROM tool_parameters WHERE tool_id = ?1")
            .bind(tool_id.to_string())
            .execute(&self.pool)
            .await?;

        for parameter in parameters {
            self.create_parameter(tool_id, new_parameter_from_exported_parameter(parameter))
                .await?;
        }

        self.touch_tool(tool_id).await
    }

    async fn get_call_log(&self, id: Uuid) -> Result<Option<CallLog>, AppError> {
        let row = sqlx::query("SELECT * FROM call_logs WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        row.as_ref().map(call_log_from_row).transpose()
    }

    async fn require_tool(&self, id: Uuid) -> Result<(), AppError> {
        if self.get_tool(id).await?.is_some() {
            Ok(())
        } else {
            Err(AppError::NotFound(format!("tool `{id}`")))
        }
    }

    async fn touch_tool(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE tools SET updated_at = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn ensure_category(&self, category: Option<&str>) -> Result<(), AppError> {
        let Some(category) = normalize_optional_text(category) else {
            return Ok(());
        };

        sqlx::query(
            "INSERT OR IGNORE INTO tool_categories (name, created_at)
            VALUES (?1, ?2)",
        )
        .bind(category)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn sync_categories_from_tools(&self) -> Result<(), AppError> {
        sqlx::query(
            "INSERT OR IGNORE INTO tool_categories (name, created_at)
            SELECT DISTINCT TRIM(category), ?1
            FROM tools
            WHERE category IS NOT NULL AND TRIM(category) != ''",
        )
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn seed_settings(&self) -> Result<(), AppError> {
        let settings = default_settings();
        sqlx::query(
            "INSERT OR IGNORE INTO app_settings (
                id, default_working_dir, allowed_base_paths_json, default_timeout_ms, mcp_endpoint
            ) VALUES (1, ?1, ?2, ?3, ?4)",
        )
        .bind(settings.default_working_dir)
        .bind(serde_json::to_string(&settings.allowed_base_paths)?)
        .bind(settings.default_timeout_ms)
        .bind(settings.mcp_endpoint)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn seed_default_tools(&self) -> Result<(), AppError> {
        for tool in default_tools() {
            self.insert_default_tool(&tool).await?;
        }

        Ok(())
    }

    async fn insert_default_tool(&self, spec: &DefaultToolSpec) -> Result<(), AppError> {
        if self.get_tool_by_name(spec.name()).await?.is_some() {
            return Ok(());
        }

        let tool = self.create_tool(spec.new_tool()).await?;
        for (position, parameter) in spec.parameters().iter().enumerate() {
            self.create_parameter(tool.id, parameter.new_parameter(position as i64))
                .await?;
        }

        Ok(())
    }

    async fn migrate_tools_schema(&self) -> Result<(), AppError> {
        self.ensure_column("tools", "working_dir", "TEXT").await?;
        self.ensure_column("tools", "working_dir_expose", "INTEGER NOT NULL DEFAULT 0")
            .await?;
        self.ensure_column(
            "tools",
            "working_dir_required",
            "INTEGER NOT NULL DEFAULT 0",
        )
        .await
    }

    async fn migrate_settings_schema(&self) -> Result<(), AppError> {
        self.ensure_column(
            "app_settings",
            "default_timeout_ms",
            "INTEGER NOT NULL DEFAULT 30000",
        )
        .await?;
        self.ensure_column("app_settings", "mcp_endpoint", "TEXT NOT NULL DEFAULT ''")
            .await
    }

    async fn ensure_column(
        &self,
        table: &str,
        column: &str,
        definition: &str,
    ) -> Result<(), AppError> {
        if self.column_exists(table, column).await? {
            return Ok(());
        }

        let statement = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        sqlx::query(AssertSqlSafe(statement))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn column_exists(&self, table: &str, column: &str) -> Result<bool, AppError> {
        let statement = format!("PRAGMA table_info({table})");
        let rows = sqlx::query(AssertSqlSafe(statement))
            .fetch_all(&self.pool)
            .await?;

        for row in rows {
            let name: String = row.try_get("name")?;
            if name == column {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn tools_from_rows(
        &self,
        rows: Vec<sqlx::sqlite::SqliteRow>,
    ) -> Result<Vec<Tool>, AppError> {
        let mut tools = Vec::with_capacity(rows.len());
        for row in rows {
            let id = parse_uuid(row.try_get::<String, _>("id")?.as_str())?;
            let mut tool = tool_from_row(&row)?;
            tool.parameters = self.list_parameters(id).await?;
            tools.push(tool);
        }

        Ok(tools)
    }
}

fn tool_summary_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<ToolSummary, AppError> {
    Ok(ToolSummary {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        category: row.try_get("category")?,
        working_dir: row.try_get("working_dir")?,
        working_dir_expose: row.try_get::<i64, _>("working_dir_expose")? == 1,
        working_dir_required: row.try_get::<i64, _>("working_dir_required")? == 1,
        timeout_ms: row.try_get("timeout_ms")?,
        enabled: row.try_get::<i64, _>("enabled")? == 1,
        last_run_at: parse_optional_datetime(row.try_get("last_run_at")?)?,
    })
}

fn exported_tool_from_tool(tool: &Tool) -> ExportedTool {
    ExportedTool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        category: tool.category.clone(),
        script_body: tool.script_body.clone(),
        script_path: tool.script_path.clone(),
        working_dir: tool.working_dir.clone(),
        working_dir_expose: tool.working_dir_expose,
        working_dir_required: tool.working_dir_required,
        timeout_ms: tool.timeout_ms,
        enabled: tool.enabled,
        parameters: tool
            .parameters
            .iter()
            .map(exported_parameter_from_parameter)
            .collect(),
    }
}

fn exported_parameter_from_parameter(parameter: &ToolParameter) -> ExportedParameter {
    ExportedParameter {
        name: parameter.name.clone(),
        parameter_type: parameter.parameter_type.clone(),
        description: parameter.description.clone(),
        required: parameter.required,
        default_value: parameter.default_value.clone(),
        validation: parameter.validation.clone(),
        position: parameter.position,
    }
}

fn new_tool_from_exported_tool(tool: &ExportedTool) -> NewTool {
    NewTool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        category: tool.category.clone(),
        script_body: tool.script_body.clone(),
        script_path: tool.script_path.clone(),
        working_dir: tool.working_dir.clone(),
        working_dir_expose: tool.working_dir_expose,
        working_dir_required: tool.working_dir_required,
        timeout_ms: tool.timeout_ms,
        enabled: tool.enabled,
    }
}

fn new_parameter_from_exported_parameter(parameter: &ExportedParameter) -> NewParameter {
    NewParameter {
        name: parameter.name.clone(),
        parameter_type: parameter.parameter_type.clone(),
        description: parameter.description.clone(),
        required: parameter.required,
        default_value: parameter.default_value.clone(),
        validation: parameter.validation.clone(),
        position: parameter.position,
    }
}

fn validate_exported_parameters(parameters: &[ExportedParameter]) -> Result<(), AppError> {
    for parameter in parameters {
        validate_parameter(&new_parameter_from_exported_parameter(parameter))?;
    }

    Ok(())
}

fn tool_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Tool, AppError> {
    Ok(Tool {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        category: row.try_get("category")?,
        script_body: row.try_get("script_body")?,
        script_path: row.try_get("script_path")?,
        working_dir: row.try_get("working_dir")?,
        working_dir_expose: row.try_get::<i64, _>("working_dir_expose")? == 1,
        working_dir_required: row.try_get::<i64, _>("working_dir_required")? == 1,
        timeout_ms: row.try_get("timeout_ms")?,
        enabled: row.try_get::<i64, _>("enabled")? == 1,
        created_at: parse_datetime(row.try_get::<String, _>("created_at")?.as_str())?,
        updated_at: parse_datetime(row.try_get::<String, _>("updated_at")?.as_str())?,
        parameters: Vec::new(),
        last_run_at: parse_optional_datetime(row.try_get("last_run_at")?)?,
    })
}

fn parameter_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<ToolParameter, AppError> {
    let default_value = row
        .try_get::<Option<String>, _>("default_value")?
        .map(|value| serde_json::from_str(&value))
        .transpose()?;
    let validation_json = row.try_get::<String, _>("validation_json")?;

    Ok(ToolParameter {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        tool_id: parse_uuid(row.try_get::<String, _>("tool_id")?.as_str())?,
        name: row.try_get("name")?,
        parameter_type: row.try_get::<String, _>("type")?.as_str().try_into()?,
        description: row.try_get("description")?,
        required: row.try_get::<i64, _>("required")? == 1,
        default_value,
        validation: serde_json::from_str::<ValidationRules>(&validation_json)?,
        position: row.try_get("position")?,
    })
}

fn call_log_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<CallLog, AppError> {
    let params_json = serde_json::from_str::<Value>(&row.try_get::<String, _>("params_json")?)?;
    let tool_id = row
        .try_get::<Option<String>, _>("tool_id")?
        .map(|value| parse_uuid(&value))
        .transpose()?;

    Ok(CallLog {
        id: parse_uuid(row.try_get::<String, _>("id")?.as_str())?,
        tool_id,
        tool_name: row.try_get("tool_name")?,
        source: row.try_get::<String, _>("source")?.as_str().try_into()?,
        params_json,
        stdout: row.try_get("stdout")?,
        stderr: row.try_get("stderr")?,
        exit_code: row.try_get("exit_code")?,
        started_at: parse_datetime(row.try_get::<String, _>("started_at")?.as_str())?,
        finished_at: parse_datetime(row.try_get::<String, _>("finished_at")?.as_str())?,
        duration_ms: row.try_get("duration_ms")?,
        status: row.try_get::<String, _>("status")?.as_str().try_into()?,
    })
}

fn settings_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<AppSettings, AppError> {
    let mcp_endpoint =
        normalize_optional_text(Some(row.try_get::<String, _>("mcp_endpoint")?.as_str()))
            .filter(|value| value != "http://127.0.0.1:7878/mcp")
            .unwrap_or_else(default_mcp_endpoint);

    Ok(AppSettings {
        default_working_dir: row.try_get("default_working_dir")?,
        allowed_base_paths: serde_json::from_str(
            &row.try_get::<String, _>("allowed_base_paths_json")?,
        )?,
        default_timeout_ms: row.try_get("default_timeout_ms")?,
        mcp_endpoint,
    })
}

fn category_name_from_input(name: String) -> Result<String, AppError> {
    normalize_optional_text(Some(&name))
        .ok_or_else(|| AppError::Validation("category name is required".to_string()))
}

fn same_text(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn parse_uuid(value: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|error| AppError::Internal(error.to_string()))
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|datetime| datetime.with_timezone(&Utc))
        .map_err(|error| AppError::Internal(error.to_string()))
}

fn parse_optional_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>, AppError> {
    value.map(|datetime| parse_datetime(&datetime)).transpose()
}

fn log_matches_filter(log: &CallLog, filter: &LogFilter) -> Result<bool, AppError> {
    if filter
        .tool_id
        .is_some_and(|tool_id| log.tool_id != Some(tool_id))
    {
        return Ok(false);
    }

    if let Some(status) = &filter.status
        && log.status != ExecutionStatus::try_from(status.as_str())?
    {
        return Ok(false);
    }

    if let Some(from) = &filter.from
        && log.started_at < parse_datetime(from)?
    {
        return Ok(false);
    }

    if let Some(to) = &filter.to
        && log.started_at > parse_datetime(to)?
    {
        return Ok(false);
    }

    if let Some(search) = &filter.search {
        let needle = search.to_ascii_lowercase();
        let haystack = format!(
            "{} {} {} {}",
            log.tool_name, log.stdout, log.stderr, log.params_json
        )
        .to_ascii_lowercase();

        if !haystack.contains(&needle) {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DEFAULT_TIMEOUT_MS;

    #[tokio::test]
    async fn bootstrap_seeds_default_ai_tools() {
        let (_tempdir, database) = bootstrapped_database().await;

        let debate_tool = database
            .get_tool_by_name("ai_cli_debate")
            .await
            .expect("load tool")
            .expect("seeded debate tool");
        let doctor_tool = database
            .get_tool_by_name("ai_cli_doctor")
            .await
            .expect("load tool")
            .expect("seeded doctor tool");

        assert_eq!(debate_tool.category.as_deref(), Some("AI"));
        assert_eq!(debate_tool.parameters[0].name, "topic");
        assert_eq!(doctor_tool.parameters.len(), 0);
    }

    #[tokio::test]
    async fn settings_roundtrip_persists_frontend_controls() {
        let (_tempdir, database) = bootstrapped_database().await;

        let settings = database
            .update_settings(UpdateSettings {
                default_working_dir: Some("/tmp".to_string()),
                allowed_base_paths: vec!["/tmp".to_string()],
                default_timeout_ms: Some(120_000),
                mcp_endpoint: Some("http://127.0.0.1:9999/mcp".to_string()),
            })
            .await
            .expect("settings should save");

        assert_eq!(settings.default_working_dir.as_deref(), Some("/tmp"));
        assert_eq!(settings.allowed_base_paths, vec!["/tmp"]);
        assert_eq!(settings.default_timeout_ms, 120_000);
        assert_eq!(settings.mcp_endpoint, "http://127.0.0.1:9999/mcp");
    }

    #[tokio::test]
    async fn category_rename_and_delete_updates_tools() {
        let (_tempdir, database) = bootstrapped_database().await;

        database
            .create_tool(NewTool {
                name: "category_test".to_string(),
                description: "category mutation test".to_string(),
                category: Some("Ops".to_string()),
                script_body: Some("echo ok".to_string()),
                script_path: None,
                working_dir: None,
                working_dir_expose: false,
                working_dir_required: false,
                timeout_ms: DEFAULT_TIMEOUT_MS,
                enabled: true,
            })
            .await
            .expect("tool should save");

        database
            .rename_category("Ops".to_string(), "Platform".to_string())
            .await
            .expect("category should rename");
        let renamed_tool = database
            .get_tool_by_name("category_test")
            .await
            .expect("load renamed tool")
            .expect("tool exists");

        assert_eq!(renamed_tool.category.as_deref(), Some("Platform"));

        database
            .delete_category("Platform".to_string())
            .await
            .expect("category should delete");
        let uncategorized_tool = database
            .get_tool_by_name("category_test")
            .await
            .expect("load uncategorized tool")
            .expect("tool exists");

        assert_eq!(uncategorized_tool.category, None);
    }

    #[tokio::test]
    async fn tool_export_includes_categories_and_parameters() {
        let (_tempdir, database) = bootstrapped_database().await;

        let tool = database
            .create_tool(NewTool {
                name: "export_test".to_string(),
                description: "export test".to_string(),
                category: Some("Ops".to_string()),
                script_body: Some("echo \"$TARGET\"".to_string()),
                script_path: None,
                working_dir: Some("/tmp".to_string()),
                working_dir_expose: true,
                working_dir_required: false,
                timeout_ms: DEFAULT_TIMEOUT_MS,
                enabled: true,
            })
            .await
            .expect("tool should save");
        database
            .create_parameter(
                tool.id,
                NewParameter {
                    name: "TARGET".to_string(),
                    parameter_type: crate::domain::ParameterType::String,
                    description: Some("target value".to_string()),
                    required: true,
                    default_value: Some(serde_json::json!("world")),
                    validation: ValidationRules::default(),
                    position: 0,
                },
            )
            .await
            .expect("parameter should save");

        let export = database.export_tools().await.expect("export should load");
        let exported_tool = export
            .tools
            .iter()
            .find(|tool| tool.name == "export_test")
            .expect("exported tool should exist");

        assert_eq!(export.version, 1);
        assert!(export.categories.iter().any(|category| category == "Ops"));
        assert_eq!(exported_tool.working_dir.as_deref(), Some("/tmp"));
        assert_eq!(exported_tool.parameters.len(), 1);
        assert_eq!(exported_tool.parameters[0].name, "TARGET");
    }

    #[tokio::test]
    async fn tool_import_merges_by_name_and_replaces_parameters() {
        let (_tempdir, database) = bootstrapped_database().await;

        let existing_tool = database
            .create_tool(NewTool {
                name: "import_test".to_string(),
                description: "old import test".to_string(),
                category: Some("Old".to_string()),
                script_body: Some("echo old".to_string()),
                script_path: Some("/tmp/old.sh".to_string()),
                working_dir: None,
                working_dir_expose: false,
                working_dir_required: false,
                timeout_ms: DEFAULT_TIMEOUT_MS,
                enabled: true,
            })
            .await
            .expect("existing tool should save");
        database
            .create_parameter(
                existing_tool.id,
                NewParameter {
                    name: "OLD_VALUE".to_string(),
                    parameter_type: crate::domain::ParameterType::String,
                    description: None,
                    required: false,
                    default_value: None,
                    validation: ValidationRules::default(),
                    position: 0,
                },
            )
            .await
            .expect("old parameter should save");

        let result = database
            .import_tools(ToolsExport {
                version: 1,
                exported_at: Utc::now(),
                categories: vec!["New".to_string()],
                tools: vec![ExportedTool {
                    name: "import_test".to_string(),
                    description: "new import test".to_string(),
                    category: Some("New".to_string()),
                    script_body: Some("echo new".to_string()),
                    script_path: None,
                    working_dir: Some("/var/tmp".to_string()),
                    working_dir_expose: true,
                    working_dir_required: true,
                    timeout_ms: 42_000,
                    enabled: false,
                    parameters: vec![ExportedParameter {
                        name: "NEXT_VALUE".to_string(),
                        parameter_type: crate::domain::ParameterType::String,
                        description: Some("new value".to_string()),
                        required: true,
                        default_value: Some(serde_json::json!("next")),
                        validation: ValidationRules::default(),
                        position: 0,
                    }],
                }],
            })
            .await
            .expect("import should merge");
        let imported_tool = database
            .get_tool_by_name("import_test")
            .await
            .expect("load imported tool")
            .expect("tool exists");

        assert_eq!(result.imported_tools, 1);
        assert_eq!(result.imported_categories, 1);
        assert_eq!(imported_tool.description, "new import test");
        assert_eq!(imported_tool.category.as_deref(), Some("New"));
        assert_eq!(imported_tool.script_body.as_deref(), Some("echo new"));
        assert_eq!(imported_tool.script_path, None);
        assert_eq!(imported_tool.working_dir.as_deref(), Some("/var/tmp"));
        assert!(!imported_tool.enabled);
        assert_eq!(imported_tool.parameters.len(), 1);
        assert_eq!(imported_tool.parameters[0].name, "NEXT_VALUE");
    }

    async fn bootstrapped_database() -> (tempfile::TempDir, Database) {
        let tempdir = tempfile::tempdir().expect("temp dir");
        let database_path = tempdir.path().join("conjure-test.db");
        let database_url = format!("sqlite://{}?mode=rwc", database_path.display());
        let database = Database::connect(&database_url).await.expect("connect");

        database.bootstrap().await.expect("bootstrap");
        (tempdir, database)
    }
}
