import { defineStore } from "pinia";

import { DEFAULT_TIMEOUT_MS, createToolDraft, draftFromTool } from "@/domain/defaults";
import type {
  AppSettings,
  CallLog,
  LogFilter,
  ToastMessage,
  Tool,
  ToolDraft,
  ToolImportResult,
  ToolSummary,
  ToolsExport,
} from "@/domain/types";
import * as api from "@/services/api";

interface ConjureState {
  tools: ToolSummary[];
  toolDetails: Record<string, Tool>;
  categories: string[];
  settings: AppSettings;
  logs: CallLog[];
  query: string;
  loading: boolean;
  error: string | null;
  toasts: ToastMessage[];
}

const defaultAllowedBasePaths = ["/srv/app", "/var/www", "/backups", "/tmp"];
const defaultCategoryNames = ["AI", "Deploy", "Database", "Observability", "Media", "Network", "Build"];
const defaultMcpCommand = "conjure --mcp";

export const useConjureStore = defineStore("conjure", {
  state: (): ConjureState => ({
    tools: [],
    toolDetails: {},
    categories: [...defaultCategoryNames],
    settings: createDefaultSettings(),
    logs: [],
    query: "",
    loading: false,
    error: null,
    toasts: [],
  }),

  getters: {
    toolById: (state) => (toolId: string): ToolSummary | undefined => {
      return state.tools.find((tool) => tool.id === toolId) ?? state.toolDetails[toolId];
    },
    detailsById: (state) => (toolId: string): Tool | undefined => state.toolDetails[toolId],
    sortedCategories: (state): string[] => [...state.categories].sort((left, right) => left.localeCompare(right)),
    filteredTools: (state): ToolSummary[] => {
      const query = state.query.trim().toLowerCase();
      if (!query) {
        return state.tools;
      }

      return state.tools.filter((tool) => textForTool(tool).includes(query));
    },
  },

  actions: {
    async bootstrap(): Promise<void> {
      await this.withLoading(async () => {
        await Promise.all([this.refreshTools(), this.refreshCategories(), this.refreshSettings(), this.refreshLogs({})]);
      });
    },

    async refreshTools(): Promise<void> {
      this.tools = await api.listTools();
    },

    async refreshCategories(): Promise<void> {
      try {
        this.categories = await api.listCategories();
      } catch (error) {
        if (isMissingEndpoint(error)) {
          this.categories = categoriesFromTools(this.tools);
          return;
        }

        throw error;
      }
    },

    async refreshSettings(): Promise<void> {
      try {
        this.settings = withSettingsDefaults(await api.getSettings());
      } catch (error) {
        if (isMissingEndpoint(error)) {
          this.settings = createDefaultSettings();
          return;
        }

        throw error;
      }
    },

    async refreshLogs(filter: LogFilter): Promise<void> {
      this.logs = await api.listLogs(filter);
    },

    async fetchTool(toolId: string): Promise<Tool> {
      const tool = await api.getTool(toolId);
      this.toolDetails[tool.id] = tool;
      return tool;
    },

    createDraft(): ToolDraft {
      return createToolDraft(this.sortedCategories, this.settings.default_timeout_ms);
    },

    async draftForTool(toolId?: string): Promise<ToolDraft> {
      if (!toolId) {
        return this.createDraft();
      }

      const cached = this.toolDetails[toolId];
      const tool = cached ?? (await this.fetchTool(toolId));
      return draftFromTool(tool);
    },

    async saveTool(draft: ToolDraft): Promise<Tool> {
      const savedTool = draft.id ? await api.updateTool(draft) : await api.createTool(draft);
      await api.replaceParameters(savedTool.id, draft.parameters);
      const completeTool = await this.fetchTool(savedTool.id);
      await Promise.all([this.refreshTools(), this.refreshCategories()]);
      this.toast(`Saved ${completeTool.name}`, "success");
      return completeTool;
    },

    async removeTool(toolId: string): Promise<void> {
      await api.deleteTool(toolId);
      delete this.toolDetails[toolId];
      await this.refreshTools();
      this.toast("Tool deleted", "neutral");
    },

    async toggleTool(tool: ToolSummary): Promise<void> {
      const updatedTool = await api.setToolEnabled(tool.id, !tool.enabled);
      this.toolDetails[updatedTool.id] = updatedTool;
      await this.refreshTools();
      this.toast(`${updatedTool.name} ${updatedTool.enabled ? "enabled" : "disabled"}`, "success");
    },

    async exportTools(): Promise<ToolsExport> {
      const exportPayload = await api.exportTools();
      this.toast(`${toolCountText(exportPayload.tools.length)} exported`, "success");
      return exportPayload;
    },

    async importTools(payload: ToolsExport): Promise<ToolImportResult> {
      const result = await api.importTools(payload);
      this.toolDetails = {};
      await Promise.all([this.refreshTools(), this.refreshCategories()]);
      this.toast(`${toolCountText(result.imported_tools)} imported`, "success");
      return result;
    },

    async runTool(toolId: string, params: Record<string, unknown>, workingDir?: string | null) {
      const result = await api.runTool(toolId, { params, working_dir: workingDir });
      await this.refreshLogs({});
      return result;
    },

    async saveSettings(settings: AppSettings): Promise<void> {
      this.settings = await api.updateSettings(settings);
      this.toast("Settings saved", "success");
    },

    async saveSettingsQuietly(settings: AppSettings): Promise<void> {
      this.settings = await api.updateSettings(settings);
    },

    async createCategory(name: string): Promise<string> {
      const result = await api.createCategory(name);
      this.categories = result.categories;
      return result.category;
    },

    async renameCategory(previousName: string, nextName: string): Promise<void> {
      const result = await api.renameCategory(previousName, nextName);
      this.categories = result.categories;
      await this.refreshTools();
      this.toast("Category renamed", "success");
    },

    async removeCategory(name: string): Promise<void> {
      this.categories = await api.deleteCategory(name);
      await this.refreshTools();
      this.toast("Category removed", "neutral");
    },

    setQuery(query: string): void {
      this.query = query;
    },

    toast(message: string, tone: ToastMessage["tone"]): void {
      const id = crypto.randomUUID();
      this.toasts.push({ id, message, tone });
      window.setTimeout(() => this.dismissToast(id), 2600);
    },

    dismissToast(toastId: string): void {
      this.toasts = this.toasts.filter((toast) => toast.id !== toastId);
    },

    async withLoading(task: () => Promise<void>): Promise<void> {
      this.loading = true;
      this.error = null;
      try {
        await task();
      } catch (error) {
        this.error = error instanceof Error ? error.message : "Unexpected error";
        this.toast(this.error, "error");
      } finally {
        this.loading = false;
      }
    },
  },
});

function createDefaultSettings(): AppSettings {
  return {
    default_working_dir: "/srv/app/current",
    allowed_base_paths: [...defaultAllowedBasePaths],
    default_timeout_ms: DEFAULT_TIMEOUT_MS,
    mcp_endpoint: defaultMcpCommand,
  };
}

function withSettingsDefaults(settings: AppSettings): AppSettings {
  const defaults = createDefaultSettings();

  return {
    default_working_dir: settings.default_working_dir ?? defaults.default_working_dir,
    allowed_base_paths: settings.allowed_base_paths.length
      ? settings.allowed_base_paths
      : defaults.allowed_base_paths,
    default_timeout_ms: settings.default_timeout_ms ?? defaults.default_timeout_ms,
    mcp_endpoint: normalizeMcpCommand(settings.mcp_endpoint ?? defaults.mcp_endpoint),
  };
}

function normalizeMcpCommand(value: string): string {
  return value === "http://127.0.0.1:7878/mcp" ? defaultMcpCommand : value;
}

function categoriesFromTools(tools: ToolSummary[]): string[] {
  const categories = tools
    .map((tool) => tool.category)
    .filter((category): category is string => Boolean(category));

  return categories.length ? [...new Set(categories)].sort() : [...defaultCategoryNames];
}

function isMissingEndpoint(error: unknown): boolean {
  return error instanceof api.ApiError && error.status === 404;
}

function textForTool(tool: ToolSummary): string {
  return `${tool.name} ${tool.description} ${tool.category ?? ""}`.toLowerCase();
}

function toolCountText(count: number): string {
  return `${count} tool${count === 1 ? "" : "s"}`;
}
