import type { ParameterDraft, ParameterType, Tool, ToolDraft, ToolParameter, ValidationRules } from "./types";

export const MILLISECONDS_PER_SECOND = 1000;
export const DEFAULT_TIMEOUT_SECONDS = 30;
export const MAX_TIMEOUT_SECONDS = 21_600;
export const DEFAULT_TIMEOUT_MS = DEFAULT_TIMEOUT_SECONDS * MILLISECONDS_PER_SECOND;
export const MAX_TIMEOUT_MS = MAX_TIMEOUT_SECONDS * MILLISECONDS_PER_SECOND;

export function timeoutSecondsFromMs(timeoutMs: number): number {
  return Math.round(timeoutMs / MILLISECONDS_PER_SECOND);
}

export function timeoutMsFromSeconds(value: string | number): number | null {
  const seconds = Number(value);
  if (!Number.isFinite(seconds)) {
    return null;
  }

  return normalizeTimeoutSeconds(seconds) * MILLISECONDS_PER_SECOND;
}

export function createValidationRules(): ValidationRules {
  return {
    min: null,
    max: null,
    regex: null,
    format: null,
    integer: false,
    enum_values: [],
  };
}

export function createParameterDraft(position: number): ParameterDraft {
  return {
    local_id: crypto.randomUUID(),
    name: "",
    type: "string",
    description: "",
    required: false,
    default_value: "",
    validation: createValidationRules(),
    position,
  };
}

export function createToolDraft(categories: string[], timeoutMs = DEFAULT_TIMEOUT_MS): ToolDraft {
  return {
    name: "",
    description: "",
    category: categories[0] ?? null,
    script_body: defaultScriptBody(),
    script_path: null,
    working_dir: null,
    working_dir_expose: false,
    working_dir_required: false,
    timeout_ms: timeoutMs,
    enabled: true,
    parameters: [createParameterDraft(0)],
  };
}

export function draftFromTool(tool: Tool): ToolDraft {
  return {
    id: tool.id,
    name: tool.name,
    description: tool.description,
    category: tool.category,
    script_body: tool.script_body ?? "",
    script_path: tool.script_path,
    working_dir: tool.working_dir,
    working_dir_expose: tool.working_dir_expose,
    working_dir_required: tool.working_dir_required,
    timeout_ms: tool.timeout_ms,
    enabled: tool.enabled,
    parameters: tool.parameters.map(parameterDraftFromToolParameter),
  };
}

export function defaultValueForType(type: ParameterType): unknown {
  if (type === "boolean") {
    return false;
  }

  if (type === "number") {
    return null;
  }

  return "";
}

function parameterDraftFromToolParameter(parameter: ToolParameter): ParameterDraft {
  return {
    id: parameter.id,
    local_id: parameter.id,
    name: parameter.name,
    type: parameter.type,
    description: parameter.description ?? "",
    required: parameter.required,
    default_value: parameter.default_value ?? defaultValueForType(parameter.type),
    validation: normalizeValidationRules(parameter.validation),
    position: parameter.position,
  };
}

function normalizeValidationRules(validation: ValidationRules): ValidationRules {
  return {
    min: validation.min ?? null,
    max: validation.max ?? null,
    regex: validation.regex ?? null,
    format: validation.format ?? null,
    integer: validation.integer,
    enum_values: validation.enum_values ?? [],
  };
}

function defaultScriptBody(): string {
  return `#!/usr/bin/env bash
set -euo pipefail

echo "hello from $TOOL_NAME"
`;
}

function normalizeTimeoutSeconds(seconds: number): number {
  return Math.min(MAX_TIMEOUT_SECONDS, Math.max(1, Math.round(seconds)));
}
