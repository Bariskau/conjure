use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::error::AppError;

pub const DEFAULT_TIMEOUT_MS: i64 = 30_000;
pub const OUTPUT_LIMIT_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub script_body: Option<String>,
    pub script_path: Option<String>,
    pub working_dir: Option<String>,
    pub working_dir_expose: bool,
    pub working_dir_required: bool,
    pub timeout_ms: i64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parameters: Vec<ToolParameter>,
    pub last_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSummary {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub working_dir: Option<String>,
    pub working_dir_expose: bool,
    pub working_dir_required: bool,
    pub timeout_ms: i64,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewTool {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub script_body: Option<String>,
    pub script_path: Option<String>,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub working_dir_expose: bool,
    #[serde(default)]
    pub working_dir_required: bool,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: i64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTool {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub script_body: Option<String>,
    pub script_path: Option<String>,
    pub working_dir: Option<String>,
    pub working_dir_expose: Option<bool>,
    pub working_dir_required: Option<bool>,
    pub timeout_ms: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Enum,
    Path,
}

impl ParameterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Enum => "enum",
            Self::Path => "path",
        }
    }
}

impl TryFrom<&str> for ParameterType {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "string" => Ok(Self::String),
            "number" => Ok(Self::Number),
            "boolean" => Ok(Self::Boolean),
            "enum" => Ok(Self::Enum),
            "path" => Ok(Self::Path),
            other => Err(AppError::Validation(format!(
                "unsupported parameter type `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ValidationRules {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub regex: Option<String>,
    pub format: Option<String>,
    #[serde(default)]
    pub integer: bool,
    #[serde(default)]
    pub enum_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolParameter {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<Value>,
    pub validation: ValidationRules,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsExport {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub categories: Vec<String>,
    pub tools: Vec<ExportedTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedTool {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub script_body: Option<String>,
    pub script_path: Option<String>,
    pub working_dir: Option<String>,
    pub working_dir_expose: bool,
    pub working_dir_required: bool,
    pub timeout_ms: i64,
    pub enabled: bool,
    pub parameters: Vec<ExportedParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<Value>,
    pub validation: ValidationRules,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolImportResult {
    pub imported_tools: usize,
    pub imported_categories: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    pub default_value: Option<Value>,
    #[serde(default)]
    pub validation: ValidationRules,
    #[serde(default)]
    pub position: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateParameter {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub parameter_type: Option<ParameterType>,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub default_value: Option<Value>,
    pub validation: Option<ValidationRules>,
    pub position: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_working_dir: Option<String>,
    #[serde(default)]
    pub allowed_base_paths: Vec<String>,
    pub default_timeout_ms: i64,
    pub mcp_endpoint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettings {
    pub default_working_dir: Option<String>,
    #[serde(default)]
    pub allowed_base_paths: Vec<String>,
    pub default_timeout_ms: Option<i64>,
    pub mcp_endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSource {
    ManualTest,
    Mcp,
}

impl ExecutionSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManualTest => "manual_test",
            Self::Mcp => "mcp",
        }
    }
}

impl TryFrom<&str> for ExecutionSource {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "manual_test" => Ok(Self::ManualTest),
            "mcp" => Ok(Self::Mcp),
            other => Err(AppError::Validation(format!(
                "unsupported execution source `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Success,
    Error,
    Timeout,
}

impl ExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Timeout => "timeout",
        }
    }
}

impl TryFrom<&str> for ExecutionStatus {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        match value {
            "success" => Ok(Self::Success),
            "error" => Ok(Self::Error),
            "timeout" => Ok(Self::Timeout),
            other => Err(AppError::Validation(format!(
                "unsupported execution status `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunToolRequest {
    #[serde(default)]
    pub params: Value,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: i64,
    pub status: ExecutionStatus,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallLog {
    pub id: Uuid,
    pub tool_id: Option<Uuid>,
    pub tool_name: String,
    pub source: ExecutionSource,
    pub params_json: Value,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone)]
pub struct NewCallLog {
    pub tool_id: Option<Uuid>,
    pub tool_name: String,
    pub source: ExecutionSource,
    pub params_json: Value,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogFilter {
    pub tool_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub env: HashMap<String, String>,
    pub params_json: Value,
    pub working_dir: Option<PathBuf>,
}

pub fn default_timeout_ms() -> i64 {
    DEFAULT_TIMEOUT_MS
}

fn default_enabled() -> bool {
    true
}

pub fn validate_new_tool(tool: &NewTool) -> Result<(), AppError> {
    validate_tool_name(&tool.name)?;
    validate_script_source(tool.script_body.as_deref(), tool.script_path.as_deref())?;
    validate_timeout(tool.timeout_ms)
}

pub fn validate_update_tool(tool: &UpdateTool) -> Result<(), AppError> {
    if let Some(name) = &tool.name {
        validate_tool_name(name)?;
    }

    if let Some(timeout_ms) = tool.timeout_ms {
        validate_timeout(timeout_ms)?;
    }

    Ok(())
}

pub fn validate_parameter(parameter: &NewParameter) -> Result<(), AppError> {
    validate_parameter_name(&parameter.name)?;
    validate_parameter_rules(&parameter.parameter_type, &parameter.validation)
}

pub fn validate_parameter_update(parameter: &UpdateParameter) -> Result<(), AppError> {
    if let Some(name) = &parameter.name {
        validate_parameter_name(name)?;
    }

    if let (Some(parameter_type), Some(validation)) =
        (&parameter.parameter_type, &parameter.validation)
    {
        validate_parameter_rules(parameter_type, validation)?;
    }

    Ok(())
}

pub fn validate_tool_name(name: &str) -> Result<(), AppError> {
    let is_valid = Regex::new(r"^[A-Za-z][A-Za-z0-9_-]{0,63}$")
        .map_err(|error| AppError::Internal(error.to_string()))?
        .is_match(name);

    if is_valid {
        Ok(())
    } else {
        Err(AppError::Validation(
            "tool name must start with a letter and contain only letters, numbers, `_`, or `-`"
                .to_string(),
        ))
    }
}

pub fn validate_parameter_name(name: &str) -> Result<(), AppError> {
    let is_valid = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]{0,63}$")
        .map_err(|error| AppError::Internal(error.to_string()))?
        .is_match(name);

    if is_valid {
        Ok(())
    } else {
        Err(AppError::Validation(
            "parameter name must be a valid environment variable name".to_string(),
        ))
    }
}

pub fn validate_script_source(
    script_body: Option<&str>,
    script_path: Option<&str>,
) -> Result<(), AppError> {
    let has_body = script_body.is_some_and(|body| !body.trim().is_empty());
    let has_path = script_path.is_some_and(|path| !path.trim().is_empty());

    if has_body || has_path {
        Ok(())
    } else {
        Err(AppError::Validation(
            "tool must include script_body or script_path".to_string(),
        ))
    }
}

pub fn validate_timeout(timeout_ms: i64) -> Result<(), AppError> {
    if (1..=3_600_000).contains(&timeout_ms) {
        Ok(())
    } else {
        Err(AppError::Validation(
            "timeout_ms must be between 1 and 3600000".to_string(),
        ))
    }
}

pub fn input_schema(tool: &Tool) -> Value {
    input_schema_for_parts(
        &tool.parameters,
        tool.working_dir_expose,
        tool.working_dir_required,
    )
}

pub fn input_schema_for_parts(
    parameters: &[ToolParameter],
    working_dir_expose: bool,
    working_dir_required: bool,
) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for parameter in parameters {
        if parameter.required {
            required.push(Value::String(parameter.name.clone()));
        }

        properties.insert(parameter.name.clone(), parameter_schema(parameter));
    }

    if working_dir_expose {
        properties.insert(
            "working_dir".to_string(),
            json!({
                "type": "string",
                "format": "path",
                "description": "Working directory used to execute the tool"
            }),
        );

        if working_dir_required {
            required.push(Value::String("working_dir".to_string()));
        }
    }

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

pub fn build_execution_plan(
    tool: &Tool,
    params: &Value,
    run_working_dir: Option<&str>,
    settings: &AppSettings,
) -> Result<ExecutionPlan, AppError> {
    let object = params.as_object().ok_or_else(|| {
        AppError::Validation("params must be a JSON object keyed by parameter name".to_string())
    })?;

    let mut validation_object = object.clone();
    let mut normalized = Map::new();
    let mut env = HashMap::new();

    for parameter in &tool.parameters {
        let value = resolve_parameter_value(parameter, object)?;
        let Some(value) = value else {
            validation_object.remove(&parameter.name);
            continue;
        };

        validate_value(parameter, &value)?;
        validation_object.insert(parameter.name.clone(), value.clone());
        normalized.insert(parameter.name.clone(), value.clone());

        let env_value = value_to_env_string(&parameter.parameter_type, &value)?;
        env.insert(parameter.name.clone(), env_value.clone());
        env.insert(tool_prefixed_env_name(&parameter.name), env_value);
    }

    let working_dir = resolve_working_dir(tool, object, run_working_dir, settings)?;
    if tool.working_dir_expose {
        if let Some(path) = &working_dir {
            validation_object.insert(
                "working_dir".to_string(),
                Value::String(path.to_string_lossy().to_string()),
            );
        }
    } else {
        validation_object.remove("working_dir");
    }

    validate_with_json_schema(tool, &Value::Object(validation_object))?;

    if let Some(path) = &working_dir {
        normalized.insert(
            "working_dir".to_string(),
            Value::String(path.to_string_lossy().to_string()),
        );
    }

    Ok(ExecutionPlan {
        env,
        params_json: Value::Object(normalized),
        working_dir,
    })
}

fn parameter_schema(parameter: &ToolParameter) -> Value {
    let mut schema = Map::new();
    schema.insert(
        "type".to_string(),
        Value::String(json_schema_type(parameter).to_string()),
    );

    if let Some(description) = &parameter.description {
        schema.insert(
            "description".to_string(),
            Value::String(description.clone()),
        );
    }

    if let Some(default_value) = &parameter.default_value {
        schema.insert("default".to_string(), default_value.clone());
    }

    match parameter.parameter_type {
        ParameterType::Number => {
            if let Some(minimum) = parameter.validation.min {
                schema.insert("minimum".to_string(), json!(minimum));
            }
            if let Some(maximum) = parameter.validation.max {
                schema.insert("maximum".to_string(), json!(maximum));
            }
        }
        ParameterType::String | ParameterType::Path => {
            if let Some(minimum) = length_limit(parameter.validation.min) {
                schema.insert("minLength".to_string(), json!(minimum));
            }
            if let Some(maximum) = length_limit(parameter.validation.max) {
                schema.insert("maxLength".to_string(), json!(maximum));
            }
        }
        ParameterType::Boolean | ParameterType::Enum => {}
    }

    if let Some(pattern) = &parameter.validation.regex {
        schema.insert("pattern".to_string(), Value::String(pattern.clone()));
    }

    if parameter.parameter_type == ParameterType::Path {
        schema.insert("format".to_string(), Value::String("path".to_string()));
    } else if let Some(format) = &parameter.validation.format {
        schema.insert("format".to_string(), Value::String(format.clone()));
    }

    if parameter.parameter_type == ParameterType::Enum
        || !parameter.validation.enum_values.is_empty()
    {
        schema.insert(
            "enum".to_string(),
            Value::Array(
                parameter
                    .validation
                    .enum_values
                    .iter()
                    .map(|value| Value::String(value.clone()))
                    .collect(),
            ),
        );
    }

    Value::Object(schema)
}

fn json_schema_type(parameter: &ToolParameter) -> &'static str {
    match parameter.parameter_type {
        ParameterType::String | ParameterType::Enum | ParameterType::Path => "string",
        ParameterType::Number if parameter.validation.integer => "integer",
        ParameterType::Number => "number",
        ParameterType::Boolean => "boolean",
    }
}

fn length_limit(value: Option<f64>) -> Option<u64> {
    let value = value?;
    if value.is_finite() && value >= 0.0 {
        Some(value.round() as u64)
    } else {
        None
    }
}

fn resolve_parameter_value(
    parameter: &ToolParameter,
    object: &Map<String, Value>,
) -> Result<Option<Value>, AppError> {
    if let Some(value) = object.get(&parameter.name) {
        return Ok(Some(coerce_value(parameter, value)?));
    }

    if let Some(default_value) = &parameter.default_value {
        return Ok(Some(coerce_value(parameter, default_value)?));
    }

    if parameter.required {
        return Err(AppError::Validation(format!(
            "missing required parameter `{}`",
            parameter.name
        )));
    }

    Ok(None)
}

fn coerce_value(parameter: &ToolParameter, value: &Value) -> Result<Value, AppError> {
    match parameter.parameter_type {
        ParameterType::String | ParameterType::Enum | ParameterType::Path => {
            coerce_string(&parameter.name, value)
        }
        ParameterType::Number => {
            coerce_number(&parameter.name, value, parameter.validation.integer)
        }
        ParameterType::Boolean => coerce_boolean(&parameter.name, value),
    }
}

fn coerce_string(parameter_name: &str, value: &Value) -> Result<Value, AppError> {
    match value {
        Value::String(text) => Ok(Value::String(text.clone())),
        Value::Number(number) => Ok(Value::String(number.to_string())),
        Value::Bool(boolean) => Ok(Value::String(boolean.to_string())),
        _ => Err(AppError::Validation(format!(
            "parameter `{parameter_name}` must be string-compatible"
        ))),
    }
}

fn coerce_number(parameter_name: &str, value: &Value, integer: bool) -> Result<Value, AppError> {
    let number = if let Some(number) = value.as_f64() {
        number
    } else if let Some(text) = value.as_str() {
        text.parse::<f64>().map_err(|_| {
            AppError::Validation(format!("parameter `{parameter_name}` must be a number"))
        })?
    } else {
        return Err(AppError::Validation(format!(
            "parameter `{parameter_name}` must be a number"
        )));
    };

    if integer {
        if number.fract() == 0.0 {
            return Ok(json!(number as i64));
        }

        return Err(AppError::Validation(format!(
            "parameter `{parameter_name}` must be an integer"
        )));
    }

    Ok(json!(number))
}

fn coerce_boolean(parameter_name: &str, value: &Value) -> Result<Value, AppError> {
    if let Some(boolean) = value.as_bool() {
        return Ok(Value::Bool(boolean));
    }

    if let Some(text) = value.as_str() {
        return match text {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(AppError::Validation(format!(
                "parameter `{parameter_name}` must be a boolean"
            ))),
        };
    }

    Err(AppError::Validation(format!(
        "parameter `{parameter_name}` must be a boolean"
    )))
}

fn validate_value(parameter: &ToolParameter, value: &Value) -> Result<(), AppError> {
    validate_range(parameter, value)?;
    validate_regex(parameter, value)?;
    validate_enum(parameter, value)
}

fn validate_range(parameter: &ToolParameter, value: &Value) -> Result<(), AppError> {
    match parameter.parameter_type {
        ParameterType::Number => validate_numeric_range(parameter, value),
        ParameterType::String | ParameterType::Path => validate_length_range(parameter, value),
        ParameterType::Boolean | ParameterType::Enum => Ok(()),
    }
}

fn validate_numeric_range(parameter: &ToolParameter, value: &Value) -> Result<(), AppError> {
    let number = value.as_f64().ok_or_else(|| {
        AppError::Validation(format!("parameter `{}` must be a number", parameter.name))
    })?;

    if let Some(minimum) = parameter.validation.min
        && number < minimum
    {
        return Err(AppError::Validation(format!(
            "parameter `{}` must be at least {minimum}",
            parameter.name
        )));
    }

    if let Some(maximum) = parameter.validation.max
        && number > maximum
    {
        return Err(AppError::Validation(format!(
            "parameter `{}` must be at most {maximum}",
            parameter.name
        )));
    }

    Ok(())
}

fn validate_length_range(parameter: &ToolParameter, value: &Value) -> Result<(), AppError> {
    let Some(text) = value.as_str() else {
        return Ok(());
    };

    if let Some(minimum) = length_limit(parameter.validation.min)
        && text.chars().count() < minimum as usize
    {
        return Err(AppError::Validation(format!(
            "parameter `{}` must be at least {minimum} characters",
            parameter.name
        )));
    }

    if let Some(maximum) = length_limit(parameter.validation.max)
        && text.chars().count() > maximum as usize
    {
        return Err(AppError::Validation(format!(
            "parameter `{}` must be at most {maximum} characters",
            parameter.name
        )));
    }

    Ok(())
}

fn validate_regex(parameter: &ToolParameter, value: &Value) -> Result<(), AppError> {
    let Some(pattern) = &parameter.validation.regex else {
        return Ok(());
    };

    let Some(text) = value.as_str() else {
        return Ok(());
    };

    let regex = Regex::new(pattern).map_err(|error| {
        AppError::Validation(format!("invalid regex for `{}`: {error}", parameter.name))
    })?;

    if regex.is_match(text) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "parameter `{}` does not match required pattern",
            parameter.name
        )))
    }
}

fn validate_enum(parameter: &ToolParameter, value: &Value) -> Result<(), AppError> {
    if parameter.validation.enum_values.is_empty() {
        return Ok(());
    }

    let Some(text) = value.as_str() else {
        return Err(AppError::Validation(format!(
            "parameter `{}` must be one of the allowed values",
            parameter.name
        )));
    };

    if parameter
        .validation
        .enum_values
        .iter()
        .any(|allowed| allowed == text)
    {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "parameter `{}` must be one of: {}",
            parameter.name,
            parameter.validation.enum_values.join(", ")
        )))
    }
}

fn value_to_env_string(parameter_type: &ParameterType, value: &Value) -> Result<String, AppError> {
    match parameter_type {
        ParameterType::String | ParameterType::Enum | ParameterType::Path => value
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| AppError::Validation("expected string env value".to_string())),
        ParameterType::Number => value
            .as_f64()
            .map(|number| number.to_string())
            .ok_or_else(|| AppError::Validation("expected number env value".to_string())),
        ParameterType::Boolean => value
            .as_bool()
            .map(|boolean| boolean.to_string())
            .ok_or_else(|| AppError::Validation("expected boolean env value".to_string())),
    }
}

fn resolve_working_dir(
    tool: &Tool,
    params: &Map<String, Value>,
    run_working_dir: Option<&str>,
    settings: &AppSettings,
) -> Result<Option<PathBuf>, AppError> {
    let mcp_working_dir = if tool.working_dir_expose {
        working_dir_argument(params)?
    } else {
        None
    };
    let requested = mcp_working_dir
        .or_else(|| normalize_optional_text(run_working_dir))
        .or_else(|| normalize_optional_text(tool.working_dir.as_deref()))
        .or_else(|| normalize_optional_text(settings.default_working_dir.as_deref()));

    let Some(requested) = requested else {
        if tool.working_dir_required {
            return Err(AppError::Validation(
                "working directory is required for this tool".to_string(),
            ));
        }

        return Ok(None);
    };

    Ok(Some(canonical_allowed_working_dir(
        &requested,
        &settings.allowed_base_paths,
    )?))
}

fn working_dir_argument(params: &Map<String, Value>) -> Result<Option<String>, AppError> {
    let Some(value) = params.get("working_dir") else {
        return Ok(None);
    };

    match value {
        Value::String(text) => Ok(normalize_optional_text(Some(text))),
        Value::Null => Ok(None),
        _ => Err(AppError::Validation(
            "working_dir must be a string path".to_string(),
        )),
    }
}

pub fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn canonical_allowed_working_dir(
    requested: &str,
    allowed_base_paths: &[String],
) -> Result<PathBuf, AppError> {
    let requested_path = std::fs::canonicalize(requested).map_err(|error| {
        AppError::Validation(format!(
            "working directory `{requested}` is not accessible: {error}"
        ))
    })?;

    if !requested_path.is_dir() {
        return Err(AppError::Validation(format!(
            "working directory `{}` is not a directory",
            requested_path.display()
        )));
    }

    if allowed_base_paths.is_empty() {
        return Err(AppError::Validation(
            "no allowed base paths are configured".to_string(),
        ));
    }

    let mut has_valid_base = false;
    for allowed_base in allowed_base_paths {
        let Some(allowed_base) = normalize_optional_text(Some(allowed_base)) else {
            continue;
        };

        let Ok(canonical_base) = std::fs::canonicalize(&allowed_base) else {
            continue;
        };

        has_valid_base = true;
        if requested_path.starts_with(&canonical_base) {
            return Ok(requested_path);
        }
    }

    if has_valid_base {
        Err(AppError::Validation(format!(
            "working directory `{}` is outside the allowed base paths",
            requested_path.display()
        )))
    } else {
        Err(AppError::Validation(
            "allowed base paths do not contain an accessible directory".to_string(),
        ))
    }
}

fn validate_with_json_schema(tool: &Tool, params: &Value) -> Result<(), AppError> {
    let schema = input_schema(tool);
    let validator = jsonschema::validator_for(&schema)
        .map_err(|error| AppError::Internal(format!("invalid generated JSON Schema: {error}")))?;
    let errors = validator
        .iter_errors(params)
        .map(|error| {
            let path = error.instance_path().to_string();
            if path.is_empty() {
                error.to_string()
            } else {
                format!("{path}: {error}")
            }
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errors.join("; ")))
    }
}

fn validate_parameter_rules(
    parameter_type: &ParameterType,
    validation: &ValidationRules,
) -> Result<(), AppError> {
    if parameter_type == &ParameterType::Enum && validation.enum_values.is_empty() {
        return Err(AppError::Validation(
            "enum parameters must define enum_values".to_string(),
        ));
    }

    if validation.integer && parameter_type != &ParameterType::Number {
        return Err(AppError::Validation(
            "integer validation only applies to number parameters".to_string(),
        ));
    }

    let has_range = validation.min.is_some() || validation.max.is_some();
    if has_range && matches!(parameter_type, ParameterType::Boolean | ParameterType::Enum) {
        return Err(AppError::Validation(
            "min/max validation only applies to string, path, or number parameters".to_string(),
        ));
    }

    if let Some(minimum) = validation.min
        && !minimum.is_finite()
    {
        return Err(AppError::Validation("min must be finite".to_string()));
    }

    if let Some(maximum) = validation.max
        && !maximum.is_finite()
    {
        return Err(AppError::Validation("max must be finite".to_string()));
    }

    if let (Some(minimum), Some(maximum)) = (validation.min, validation.max)
        && minimum > maximum
    {
        return Err(AppError::Validation(
            "min must be less than or equal to max".to_string(),
        ));
    }

    if let Some(pattern) = &validation.regex {
        Regex::new(pattern)
            .map_err(|error| AppError::Validation(format!("invalid regex: {error}")))?;
    }

    Ok(())
}

fn tool_prefixed_env_name(name: &str) -> String {
    format!("TOOL_{}", name.to_ascii_uppercase())
}

pub fn default_settings() -> AppSettings {
    let current_dir = std::env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(ToString::to_string));

    AppSettings {
        default_working_dir: current_dir.clone(),
        allowed_base_paths: current_dir.into_iter().collect(),
        default_timeout_ms: DEFAULT_TIMEOUT_MS,
        mcp_endpoint: default_mcp_endpoint(),
    }
}

pub fn sanitize_settings(settings: UpdateSettings) -> Result<AppSettings, AppError> {
    let default_timeout_ms = settings.default_timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS);
    validate_timeout(default_timeout_ms)?;

    Ok(AppSettings {
        default_working_dir: normalize_optional_text(settings.default_working_dir.as_deref()),
        allowed_base_paths: settings
            .allowed_base_paths
            .iter()
            .filter_map(|path| normalize_optional_text(Some(path)))
            .fold(Vec::new(), |mut paths, path| {
                if !paths.iter().any(|existing| same_text(existing, &path)) {
                    paths.push(path);
                }
                paths
            }),
        default_timeout_ms,
        mcp_endpoint: normalize_optional_text(settings.mcp_endpoint.as_deref())
            .unwrap_or_else(default_mcp_endpoint),
    })
}

pub fn default_mcp_endpoint() -> String {
    std::env::var("CONJURE_MCP_COMMAND")
        .or_else(|_| std::env::var("CONJURE_MCP_ENDPOINT"))
        .unwrap_or_else(|_| "conjure --mcp".to_string())
}

pub fn sanitize_category(value: Option<String>) -> Option<String> {
    normalize_optional_text(value.as_deref())
}

fn same_text(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parameter(
        name: &str,
        parameter_type: ParameterType,
        required: bool,
        default_value: Option<Value>,
        validation: ValidationRules,
    ) -> ToolParameter {
        ToolParameter {
            id: Uuid::new_v4(),
            tool_id: Uuid::new_v4(),
            name: name.to_string(),
            parameter_type,
            description: None,
            required,
            default_value,
            validation,
            position: 0,
        }
    }

    fn tool(parameters: Vec<ToolParameter>) -> Tool {
        Tool {
            id: Uuid::new_v4(),
            name: "sample_tool".to_string(),
            description: "Sample".to_string(),
            category: None,
            script_body: Some("echo ok".to_string()),
            script_path: None,
            working_dir: None,
            working_dir_expose: false,
            working_dir_required: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            parameters,
            last_run_at: None,
        }
    }

    fn settings() -> AppSettings {
        default_settings()
    }

    #[test]
    fn required_parameter_is_rejected_when_missing() {
        let tool = tool(vec![parameter(
            "name",
            ParameterType::String,
            true,
            None,
            ValidationRules::default(),
        )]);

        let result = build_execution_plan(&tool, &json!({}), None, &settings());

        assert!(result.is_err());
    }

    #[test]
    fn default_value_is_applied_to_optional_parameter() {
        let tool = tool(vec![parameter(
            "greeting",
            ParameterType::String,
            false,
            Some(json!("hello")),
            ValidationRules::default(),
        )]);

        let result =
            build_execution_plan(&tool, &json!({}), None, &settings()).expect("plan should build");

        assert_eq!(result.env.get("greeting"), Some(&"hello".to_string()));
        assert_eq!(result.env.get("TOOL_GREETING"), Some(&"hello".to_string()));
    }

    #[test]
    fn validation_rules_reject_out_of_range_values() {
        let tool = tool(vec![parameter(
            "count",
            ParameterType::Number,
            true,
            None,
            ValidationRules {
                min: Some(2.0),
                max: Some(5.0),
                ..ValidationRules::default()
            },
        )]);

        let result = build_execution_plan(&tool, &json!({ "count": 9 }), None, &settings());

        assert!(result.is_err());
    }

    #[test]
    fn enum_values_are_reflected_in_input_schema() {
        let tool = tool(vec![parameter(
            "mode",
            ParameterType::Enum,
            true,
            None,
            ValidationRules {
                enum_values: vec!["fast".to_string(), "safe".to_string()],
                ..ValidationRules::default()
            },
        )]);

        let schema = input_schema(&tool);

        assert_eq!(
            schema["properties"]["mode"]["enum"],
            json!(["fast", "safe"])
        );
        assert_eq!(schema["required"], json!(["mode"]));
    }

    #[test]
    fn working_dir_is_reflected_in_input_schema_when_exposed() {
        let mut tool = tool(Vec::new());
        tool.working_dir_expose = true;
        tool.working_dir_required = true;

        let schema = input_schema(&tool);

        assert_eq!(schema["properties"]["working_dir"]["format"], json!("path"));
        assert_eq!(schema["required"], json!(["working_dir"]));
    }
}
