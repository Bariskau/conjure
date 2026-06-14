import type {
  AppSettings,
  CallLog,
  ExecutionResult,
  LogFilter,
  ParameterDraft,
  RunToolRequest,
  Tool,
  ToolDraft,
  ToolImportResult,
  ToolParameter,
  ToolSummary,
  ToolsExport,
} from "@/domain/types";

import { draftParameterToPayload } from "@/domain/schema";

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "";

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export async function listTools(): Promise<ToolSummary[]> {
  const response = await getJson<{ tools: ToolSummary[] }>("/api/tools");
  return response.tools;
}

export async function getTool(toolId: string): Promise<Tool> {
  const response = await getJson<{ tool: Tool }>(`/api/tools/${toolId}`);
  return response.tool;
}

export async function createTool(draft: ToolDraft): Promise<Tool> {
  const response = await sendJson<{ tool: Tool }>("/api/tools", "POST", toolPayload(draft));
  return response.tool;
}

export async function updateTool(draft: ToolDraft): Promise<Tool> {
  if (!draft.id) {
    throw new ApiError("Cannot update a tool without an id", 0);
  }

  const response = await sendJson<{ tool: Tool }>(`/api/tools/${draft.id}`, "PUT", toolPayload(draft));
  return response.tool;
}

export async function deleteTool(toolId: string): Promise<void> {
  await sendEmpty(`/api/tools/${toolId}`, "DELETE");
}

export async function setToolEnabled(toolId: string, enabled: boolean): Promise<Tool> {
  const response = await sendJson<{ tool: Tool }>(`/api/tools/${toolId}/enabled`, "PATCH", { enabled });
  return response.tool;
}

export async function exportTools(): Promise<ToolsExport> {
  const response = await getJson<{ export: ToolsExport }>("/api/tools/export");
  return response.export;
}

export async function importTools(payload: ToolsExport): Promise<ToolImportResult> {
  const response = await sendJson<{ result: ToolImportResult }>("/api/tools/import", "POST", payload);
  return response.result;
}

export async function listParameters(toolId: string): Promise<ToolParameter[]> {
  const response = await getJson<{ parameters: ToolParameter[] }>(`/api/tools/${toolId}/parameters`);
  return response.parameters;
}

export async function createParameter(toolId: string, parameter: ParameterDraft): Promise<ToolParameter> {
  const response = await sendJson<{ parameter: ToolParameter }>(
    `/api/tools/${toolId}/parameters`,
    "POST",
    draftParameterToPayload(parameter),
  );
  return response.parameter;
}

export async function deleteParameter(toolId: string, parameterId: string): Promise<void> {
  await sendEmpty(`/api/tools/${toolId}/parameters/${parameterId}`, "DELETE");
}

export async function replaceParameters(toolId: string, parameters: ParameterDraft[]): Promise<void> {
  const currentParameters = await listParameters(toolId);
  await Promise.all(currentParameters.map((parameter) => deleteParameter(toolId, parameter.id)));

  for (const parameter of positionedParameters(parameters)) {
    if (parameter.name.trim()) {
      await createParameter(toolId, parameter);
    }
  }
}

export async function runTool(toolId: string, request: RunToolRequest): Promise<ExecutionResult> {
  const response = await sendJson<{ result: ExecutionResult }>(`/api/tools/${toolId}/run`, "POST", request);
  return response.result;
}

export async function listLogs(filter: LogFilter): Promise<CallLog[]> {
  const response = await getJson<{ logs: CallLog[] }>(`/api/logs${queryString(filter)}`);
  return response.logs;
}

export async function listCategories(): Promise<string[]> {
  const response = await getJson<{ categories: string[] }>("/api/categories");
  return response.categories;
}

export async function createCategory(name: string): Promise<{ category: string; categories: string[] }> {
  return sendJson<{ category: string; categories: string[] }>("/api/categories", "POST", { name });
}

export async function renameCategory(
  previousName: string,
  name: string,
): Promise<{ category: string; categories: string[] }> {
  return sendJson<{ category: string; categories: string[] }>(categoryPath(previousName), "PATCH", { name });
}

export async function deleteCategory(name: string): Promise<string[]> {
  const response = await requestJson<{ categories: string[] }>(categoryPath(name), { method: "DELETE" });
  return response.categories;
}

export async function getSettings(): Promise<AppSettings> {
  const response = await getJson<{ settings: AppSettings }>("/api/settings");
  return response.settings;
}

export async function updateSettings(settings: AppSettings): Promise<AppSettings> {
  const response = await sendJson<{ settings: AppSettings }>("/api/settings", "PUT", settings);
  return response.settings;
}

function toolPayload(draft: ToolDraft): Record<string, unknown> {
  return {
    name: draft.name,
    description: draft.description,
    category: draft.category ?? "",
    script_body: draft.script_body ?? "",
    script_path: draft.script_path ?? "",
    working_dir: draft.working_dir ?? "",
    working_dir_expose: draft.working_dir_expose,
    working_dir_required: draft.working_dir_required,
    timeout_ms: draft.timeout_ms,
    enabled: draft.enabled,
  };
}

function positionedParameters(parameters: ParameterDraft[]): ParameterDraft[] {
  return parameters.map((parameter, position) => ({ ...parameter, position }));
}

function categoryPath(name: string): string {
  return `/api/categories/${encodeURIComponent(name)}`;
}

async function getJson<T>(path: string): Promise<T> {
  return requestJson<T>(path, { method: "GET" });
}

async function sendJson<T>(path: string, method: string, body: unknown): Promise<T> {
  return requestJson<T>(path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

async function sendEmpty(path: string, method: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}${path}`, { method });
  if (!response.ok) {
    throw await apiErrorFromResponse(response);
  }
}

async function requestJson<T>(path: string, init: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, init);
  if (!response.ok) {
    throw await apiErrorFromResponse(response);
  }

  return response.json() as Promise<T>;
}

async function apiErrorFromResponse(response: Response): Promise<ApiError> {
  const body = await safeResponseJson(response);
  const message = messageFromBody(body) ?? `${response.status} ${response.statusText}`;
  return new ApiError(message, response.status);
}

async function safeResponseJson(response: Response): Promise<unknown> {
  try {
    return await response.json();
  } catch {
    return null;
  }
}

function messageFromBody(body: unknown): string | null {
  if (body && typeof body === "object" && "error" in body) {
    return String((body as { error: unknown }).error);
  }

  return null;
}

function queryString(filter: LogFilter): string {
  const params = new URLSearchParams();

  for (const [key, value] of Object.entries(filter)) {
    if (value) {
      params.set(key, value);
    }
  }

  return params.size ? `?${params.toString()}` : "";
}
