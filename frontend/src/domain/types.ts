export type ParameterType = "string" | "number" | "boolean" | "enum" | "path";
export type ExecutionStatus = "success" | "error" | "timeout";
export type ExecutionStatusFilter = ExecutionStatus | "all";
export type ExecutionSource = "manual_test" | "mcp";

export interface ValidationRules {
  min?: number | null;
  max?: number | null;
  regex?: string | null;
  format?: string | null;
  integer: boolean;
  enum_values: string[];
}

export interface ToolParameter {
  id: string;
  tool_id: string;
  name: string;
  type: ParameterType;
  description?: string | null;
  required: boolean;
  default_value?: unknown;
  validation: ValidationRules;
  position: number;
}

export interface ToolSummary {
  id: string;
  name: string;
  description: string;
  category?: string | null;
  working_dir?: string | null;
  working_dir_expose: boolean;
  working_dir_required: boolean;
  timeout_ms: number;
  enabled: boolean;
  last_run_at?: string | null;
}

export interface Tool extends ToolSummary {
  script_body?: string | null;
  script_path?: string | null;
  created_at: string;
  updated_at: string;
  parameters: ToolParameter[];
}

export interface ToolsExport {
  version: number;
  exported_at: string;
  categories: string[];
  tools: ExportedTool[];
}

export interface ExportedTool {
  name: string;
  description: string;
  category?: string | null;
  script_body?: string | null;
  script_path?: string | null;
  working_dir?: string | null;
  working_dir_expose: boolean;
  working_dir_required: boolean;
  timeout_ms: number;
  enabled: boolean;
  parameters: ExportedParameter[];
}

export interface ExportedParameter {
  name: string;
  type: ParameterType;
  description?: string | null;
  required: boolean;
  default_value?: unknown;
  validation: ValidationRules;
  position: number;
}

export interface ToolImportResult {
  imported_tools: number;
  imported_categories: number;
}

export interface ToolDraft {
  id?: string;
  name: string;
  description: string;
  category?: string | null;
  script_body?: string | null;
  script_path?: string | null;
  working_dir?: string | null;
  working_dir_expose: boolean;
  working_dir_required: boolean;
  timeout_ms: number;
  enabled: boolean;
  parameters: ParameterDraft[];
}

export interface ParameterDraft {
  id?: string;
  local_id: string;
  name: string;
  type: ParameterType;
  description?: string | null;
  required: boolean;
  default_value?: unknown;
  validation: ValidationRules;
  position: number;
}

export interface AppSettings {
  default_working_dir?: string | null;
  default_timeout_ms: number;
  mcp_endpoint: string;
}

export interface CallLog {
  id: string;
  tool_id?: string | null;
  tool_name: string;
  source: ExecutionSource;
  params_json: Record<string, unknown>;
  stdout: string;
  stderr: string;
  exit_code?: number | null;
  started_at: string;
  finished_at: string;
  duration_ms: number;
  status: ExecutionStatus;
}

export interface ExecutionResult {
  stdout: string;
  stderr: string;
  exit_code?: number | null;
  duration_ms: number;
  status: ExecutionStatus;
  stdout_truncated: boolean;
  stderr_truncated: boolean;
}

export interface RunToolRequest {
  params: Record<string, unknown>;
  working_dir?: string | null;
}

export interface LogFilter {
  tool_id?: string;
  status?: ExecutionStatus;
  from?: string;
  to?: string;
  search?: string;
}

export interface JsonSchema {
  $schema: string;
  type: "object";
  properties: Record<string, unknown>;
  required?: string[];
  additionalProperties: boolean;
}

export interface ToastMessage {
  id: string;
  message: string;
  tone: "success" | "error" | "neutral";
}
