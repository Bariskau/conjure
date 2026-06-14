<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";

import AppButton from "@/components/ui/AppButton.vue";
import FormField from "@/components/ui/FormField.vue";
import GlassPanel from "@/components/ui/GlassPanel.vue";
import StatusDot from "@/components/ui/StatusDot.vue";
import type { AppSettings, ToolsExport } from "@/domain/types";
import { useConjureStore } from "@/stores/conjure";

const store = useConjureStore();

const defaultCategoryNames = new Set([
  "AI",
  "Deploy",
  "Database",
  "Observability",
  "Media",
  "Network",
  "Build",
]);
const millisecondsPerSecond = 1000;
const settingsSaveDelayMs = 300;
const workingDirectorySaveDelayMs = 2000;

const settingsDraft = ref<AppSettings>(cloneSettings(store.settings));
const importInput = ref<HTMLInputElement | null>(null);
const editingCategory = ref<string | null>(null);
const categoryDraft = ref("");
let saveSettingsTimer: number | undefined;
let workingDirectorySaveTimer: number | undefined;

const categoryCounts = computed(() => {
  return store.tools.reduce<Record<string, number>>((counts, tool) => {
    const category = tool.category ?? "Uncategorized";
    counts[category] = (counts[category] ?? 0) + 1;
    return counts;
  }, {});
});

const defaultTimeoutSeconds = computed(() => Math.round(settingsDraft.value.default_timeout_ms / millisecondsPerSecond));

watch(
  () => store.settings,
  (settings) => {
    settingsDraft.value = cloneSettings(settings);
  },
);

async function saveSettingsQuietly(): Promise<void> {
  window.clearTimeout(saveSettingsTimer);

  try {
    await store.saveSettingsQuietly(cloneSettings(settingsDraft.value));
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Settings save failed", "error");
  }
}

async function saveSettingsWithToast(): Promise<void> {
  window.clearTimeout(workingDirectorySaveTimer);

  try {
    await store.saveSettings(cloneSettings(settingsDraft.value));
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Settings save failed", "error");
  }
}

function copyMcpCommand(): void {
  navigator.clipboard.writeText(settingsDraft.value.mcp_endpoint);
  store.toast("MCP command copied", "success");
}

function openImportTools(): void {
  importInput.value?.click();
}

async function importTools(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) {
    return;
  }

  try {
    const exportPayload = parseToolsExport(await file.text());
    await store.importTools(exportPayload);
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Import failed", "error");
  } finally {
    input.value = "";
  }
}

async function exportTools(): Promise<void> {
  try {
    downloadToolsExport(await store.exportTools());
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Export failed", "error");
  }
}

function setDefaultTimeout(value: string): void {
  if (!value.trim()) {
    return;
  }

  const seconds = Number(value);
  if (!Number.isFinite(seconds)) {
    return;
  }

  const normalizedSeconds = Math.min(3600, Math.max(1, Math.round(seconds)));
  settingsDraft.value.default_timeout_ms = normalizedSeconds * millisecondsPerSecond;
  scheduleSettingsSave();
}

function startCategoryRename(category: string): void {
  editingCategory.value = category;
  categoryDraft.value = category;
}

async function saveCategoryRename(category: string): Promise<void> {
  try {
    await store.renameCategory(category, categoryDraft.value);
    editingCategory.value = null;
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Category rename failed", "error");
  }
}

function cancelCategoryRename(): void {
  editingCategory.value = null;
  categoryDraft.value = "";
}

function isDefaultCategory(category: string): boolean {
  return defaultCategoryNames.has(category);
}

function toolCountLabel(category: string): string {
  const count = categoryCounts.value[category] ?? 0;
  return `${count} tool${count === 1 ? "" : "s"}`;
}

function setDefaultWorkingDirectory(value: string): void {
  settingsDraft.value.default_working_dir = value;
  scheduleWorkingDirectorySave();
}

function scheduleSettingsSave(): void {
  window.clearTimeout(saveSettingsTimer);
  saveSettingsTimer = window.setTimeout(() => {
    void saveSettingsQuietly();
  }, settingsSaveDelayMs);
}

function scheduleWorkingDirectorySave(): void {
  window.clearTimeout(workingDirectorySaveTimer);
  workingDirectorySaveTimer = window.setTimeout(() => {
    void saveSettingsWithToast();
  }, workingDirectorySaveDelayMs);
}

async function removeCategory(category: string): Promise<void> {
  try {
    await store.removeCategory(category);
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Category removal failed", "error");
  }
}

function cloneSettings(settings: AppSettings): AppSettings {
  return { ...settings };
}

function parseToolsExport(contents: string): ToolsExport {
  const value = JSON.parse(contents) as unknown;
  if (!isToolsExport(value)) {
    throw new Error("Import file is not a tools export");
  }

  return value;
}

function isToolsExport(value: unknown): value is ToolsExport {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Partial<ToolsExport>;
  return typeof candidate.version === "number" && Array.isArray(candidate.categories) && Array.isArray(candidate.tools);
}

function downloadToolsExport(exportPayload: ToolsExport): void {
  const blob = new Blob([`${JSON.stringify(exportPayload, null, 2)}\n`], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");

  link.href = url;
  link.download = exportFileName(exportPayload.exported_at);
  link.click();
  URL.revokeObjectURL(url);
}

function exportFileName(exportedAt: string): string {
  const timestamp = exportedAt.replace(/\D/g, "").slice(0, 14) || new Date().toISOString().replace(/\D/g, "").slice(0, 14);
  return `conjure-tools-${timestamp}.json`;
}

onBeforeUnmount(() => {
  window.clearTimeout(saveSettingsTimer);
  window.clearTimeout(workingDirectorySaveTimer);
});
</script>

<template>
  <section class="settings-page page-enter">
    <div class="settings-page__header">
      <h1>Settings</h1>
      <p>Local server, defaults, and categories.</p>
    </div>

    <GlassPanel class="settings-page__panel">
      <div class="settings-row">
        <div>
          <h2>MCP command</h2>
          <p>Use this stdio command in agent clients to expose enabled tools.</p>
        </div>
        <div class="settings-page__code-action">
          <code class="settings-page__code--endpoint">{{ settingsDraft.mcp_endpoint }}</code>
          <AppButton variant="secondary" size="sm" icon="copy" title="Copy MCP command" @click="copyMcpCommand" />
        </div>
      </div>
      <hr class="hairline" />

      <div class="settings-row">
        <div>
          <h2>Server status</h2>
          <p>The local bridge that exposes your tools over MCP.</p>
        </div>
        <span class="settings-page__status"><StatusDot status="success" />Running</span>
      </div>
    </GlassPanel>

    <GlassPanel class="settings-page__panel">
      <div class="settings-row">
        <div>
          <h2>Default timeout</h2>
          <p>Applied to new tools unless overridden.</p>
        </div>
        <div class="settings-page__timeout">
          <FormField
            :model-value="defaultTimeoutSeconds"
            type="number"
            mono
            min="1"
            max="3600"
            @update:model-value="setDefaultTimeout"
            @blur="saveSettingsQuietly"
          />
          <span>sec</span>
        </div>
      </div>
      <hr class="hairline" />

      <div class="settings-row">
        <div>
          <h2>Default working directory</h2>
          <p>Global fallback when a tool or run doesn’t specify one.</p>
        </div>
        <div class="settings-page__wide-control">
          <FormField
            :model-value="settingsDraft.default_working_dir"
            mono
            icon="folder"
            placeholder="process default"
            @update:model-value="setDefaultWorkingDirectory"
          />
        </div>
      </div>
      <hr class="hairline" />

      <div class="settings-row">
        <div>
          <h2>Import / export tools</h2>
          <p>Back up every tool and category to a JSON file, or restore from one. Imported tools merge by name.</p>
        </div>
        <div class="settings-page__import-export">
          <input
            ref="importInput"
            class="settings-page__file-input"
            type="file"
            accept="application/json,.json"
            @change="importTools"
          />
          <AppButton variant="secondary" size="sm" icon="copy" @click="openImportTools">Import</AppButton>
          <AppButton variant="secondary" size="sm" icon="arrow-right" @click="exportTools">Export</AppButton>
        </div>
      </div>
    </GlassPanel>

    <GlassPanel class="settings-page__panel categories">
      <div class="settings-page__category-title">
        <h2>Categories</h2>
        <p>Rename or remove custom categories. Removing one moves its tools to “Uncategorized”.</p>
      </div>

      <div v-for="category in store.sortedCategories" :key="category" class="settings-page__category-row">
        <div v-if="editingCategory === category" class="settings-page__category-edit">
          <FormField v-model="categoryDraft" @keydown.enter="saveCategoryRename(category)" />
        </div>
        <span v-else class="settings-page__category-summary">
          <span>{{ category }}</span>
          <small>{{ toolCountLabel(category) }}</small>
        </span>
        <span v-if="isDefaultCategory(category)" class="settings-page__default-pill">default</span>
        <div v-else-if="editingCategory === category" class="settings-page__category-actions">
          <AppButton variant="secondary" size="sm" @click="saveCategoryRename(category)">Save</AppButton>
          <AppButton variant="tertiary" size="sm" @click="cancelCategoryRename">Cancel</AppButton>
        </div>
        <div v-else class="settings-page__category-actions">
          <AppButton
            variant="tertiary"
            size="sm"
            icon="pencil"
            title="Rename category"
            @click="startCategoryRename(category)"
          />
          <AppButton
            variant="tertiary"
            size="sm"
            icon="trash"
            title="Remove category"
            @click="removeCategory(category)"
          />
        </div>
      </div>
    </GlassPanel>
  </section>
</template>

<style scoped>
.settings-page {
  max-width: 1280px;
  margin: 0 auto;
  padding: 32px 24px 80px;
}

.settings-page__header,
.settings-page__panel {
  max-width: 760px;
}

.settings-page__header {
  margin-bottom: 22px;
}

h1 {
  margin: 0;
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 500;
  letter-spacing: -0.015em;
  line-height: 1.12;
}

.settings-page__header p {
  margin: 8px 0 0;
  color: var(--text-secondary);
}

.settings-row p,
.settings-page__category-title p {
  margin: 4px 0 0;
  color: var(--text-secondary);
  text-wrap: pretty;
}

.settings-page__panel {
  margin-bottom: 16px;
  padding: 6px 24px;
}

.settings-page__panel.categories {
  padding-bottom: 16px;
}

.settings-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 20px;
  padding: 18px 0;
}

h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
  letter-spacing: -0.006em;
}

.settings-row p,
.settings-page__category-title p {
  font-size: 12px;
}

.settings-page__code-action,
.settings-page__import-export,
.settings-page__status {
  display: flex;
  align-items: center;
  gap: 8px;
}

code {
  border: 1px solid var(--divider);
  border-radius: 8px;
  background: var(--console-bg);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 13px;
  padding: 8px 12px;
}

.settings-page__code--endpoint {
  color: var(--text-primary);
}

.settings-page__wide-control {
  width: 280px;
}

.settings-page__timeout {
  display: flex;
  width: 160px;
  align-items: center;
  gap: 8px;
}

.settings-page__timeout span {
  color: var(--text-tertiary);
  font-size: 12px;
}

.settings-page__file-input {
  display: none;
}

.settings-page__category-title {
  padding: 16px 0 6px;
}

.settings-page__category-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border-top: 1px solid var(--divider);
  padding: 12px 0;
}

.settings-page__category-summary {
  display: flex;
  flex: 1;
  align-items: baseline;
  gap: 10px;
}

.settings-page__category-edit {
  flex: 1;
  max-width: 280px;
}

.settings-page__category-row small {
  color: var(--text-tertiary);
  font-size: 12px;
}

.settings-page__default-pill {
  flex: 0 0 auto !important;
  border: 1px solid var(--glass-border);
  border-radius: 999px;
  color: var(--text-tertiary);
  font-size: 12px;
  padding: 2px 8px;
}

.settings-page__category-actions {
  display: flex;
  gap: 6px;
}

@media (max-width: 760px) {
  .settings-page {
    padding: 24px 16px 72px;
  }

  .settings-row {
    grid-template-columns: 1fr;
  }

  .settings-page__wide-control {
    width: 100%;
  }
}
</style>
