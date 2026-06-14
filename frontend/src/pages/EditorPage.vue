<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import EditorSection from "@/components/editor/EditorSection.vue";
import ParameterCard from "@/components/editor/ParameterCard.vue";
import AppButton from "@/components/ui/AppButton.vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import ComboBox from "@/components/ui/ComboBox.vue";
import FormField from "@/components/ui/FormField.vue";
import GlassPanel from "@/components/ui/GlassPanel.vue";
import JsonViewer from "@/components/ui/JsonViewer.vue";
import TextAreaField from "@/components/ui/TextAreaField.vue";
import ToggleSwitch from "@/components/ui/ToggleSwitch.vue";
import { createParameterDraft } from "@/domain/defaults";
import { buildInputSchema } from "@/domain/schema";
import type { ParameterDraft, ToolDraft } from "@/domain/types";
import { useConjureStore } from "@/stores/conjure";

const route = useRoute();
const router = useRouter();
const store = useConjureStore();

const draft = ref<ToolDraft | null>(null);
const expandedIds = ref<string[]>([]);
const saving = ref(false);

const toolId = computed(() => route.params.toolId as string | undefined);
const isNew = computed(() => !toolId.value);
const schema = computed(() => (draft.value ? buildInputSchema(draft.value) : {}));
const canSave = computed(() => draft.value != null && hasRequiredBasics(draft.value) && hasValidEnumParameters(draft.value));

onMounted(loadDraft);
watch(toolId, loadDraft);

async function loadDraft(): Promise<void> {
  draft.value = await store.draftForTool(toolId.value);
  expandedIds.value = [];
}

async function saveDraft(): Promise<void> {
  if (!draft.value || !canSave.value) {
    return;
  }

  saving.value = true;
  try {
    const savedTool = await store.saveTool(normalizeDraft(draft.value));
    router.push({ name: "tools" });
    store.toolDetails[savedTool.id] = savedTool;
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Save failed", "error");
  } finally {
    saving.value = false;
  }
}

function addParameter(): void {
  if (!draft.value) {
    return;
  }

  const parameter = createParameterDraft(draft.value.parameters.length);
  draft.value.parameters.push(parameter);
  expandedIds.value = [parameter.local_id];
}

function updateParameter(index: number, parameter: ParameterDraft): void {
  draft.value?.parameters.splice(index, 1, parameter);
}

function removeParameter(index: number): void {
  draft.value?.parameters.splice(index, 1);
}

function moveParameter(index: number, offset: number): void {
  if (!draft.value) {
    return;
  }

  const targetIndex = index + offset;
  const parameter = draft.value.parameters.splice(index, 1)[0];
  draft.value.parameters.splice(targetIndex, 0, parameter);
}

function toggleParameter(localId: string): void {
  expandedIds.value = expandedIds.value.includes(localId) ? [] : [localId];
}

function setName(value: string): void {
  if (draft.value) {
    draft.value.name = value.replace(/\s+/g, "-").toLowerCase();
  }
}

function setCategory(category: string): void {
  if (draft.value) {
    draft.value.category = category;
  }
}

async function createCategory(category: string): Promise<void> {
  try {
    setCategory(await store.createCategory(category));
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Category creation failed", "error");
  }
}

function normalizeDraft(value: ToolDraft): ToolDraft {
  return {
    ...value,
    parameters: value.parameters.filter((parameter) => parameter.name.trim()),
  };
}

function hasRequiredBasics(value: ToolDraft): boolean {
  const hasName = /^[A-Za-z][A-Za-z0-9_-]{0,63}$/.test(value.name);
  const hasScript = Boolean(value.script_body?.trim() || value.script_path?.trim());
  return hasName && hasScript;
}

function hasValidEnumParameters(value: ToolDraft): boolean {
  return value.parameters.every((parameter) => {
    return parameter.type !== "enum" || !parameter.name || parameter.validation.enum_values.length > 0;
  });
}
</script>

<template>
  <section v-if="draft" class="editor-page page-enter">
    <div class="editor-page__crumb">
      <button type="button" @click="router.push({ name: 'tools' })">Tools</button>
      <AppIcon name="chevron-right" :size="13" />
      <span>{{ isNew ? "New tool" : draft.name }}</span>
    </div>

    <div class="editor-page__header">
      <h1>{{ isNew ? "New tool" : "Edit tool" }}</h1>
      <div class="editor-page__actions">
        <AppButton v-if="!isNew && draft.id" variant="secondary" icon="play" @click="router.push({ name: 'test', params: { toolId: draft.id } })">
          Test
        </AppButton>
        <AppButton variant="tertiary" @click="router.push({ name: 'tools' })">Cancel</AppButton>
        <AppButton variant="primary" icon="save" :disabled="!canSave || saving" @click="saveDraft">
          {{ saving ? "Saving" : "Save changes" }}
        </AppButton>
      </div>
    </div>

    <GlassPanel class="editor-page__panel">
      <EditorSection number="1" title="Basics" description="What the tool is called and what it does.">
        <div class="editor-page__grid two">
          <div>
            <label>Name</label>
            <FormField mono :model-value="draft.name" placeholder="deploy-staging" @update:model-value="setName" />
          </div>
          <div>
            <label>Category</label>
            <ComboBox
              :model-value="draft.category"
              :options="store.sortedCategories"
              @update:model-value="setCategory"
              @create="createCategory"
            />
          </div>
          <div class="editor-page__wide">
            <label>Description</label>
            <TextAreaField v-model="draft.description" :rows="2" placeholder="Roll out the web service and wait for completion." />
          </div>
        </div>
      </EditorSection>

      <hr class="hairline" />

      <EditorSection number="2" title="Script" description="The shell that runs when the tool is called.">
        <CodeEditor v-model="draft.script_body" />
      </EditorSection>

      <hr class="hairline" />

      <EditorSection number="3" title="Parameters" description="The typed inputs your tool accepts. Compiled to one JSON Schema — the single source of truth for the Test form and the backend.">
        <div class="editor-page__parameters">
          <p v-if="draft.parameters.length === 0" class="editor-page__muted">No parameters yet.</p>
          <ParameterCard
            v-for="(parameter, index) in draft.parameters"
            :key="parameter.local_id"
            :parameter="parameter"
            :index="index"
            :open="expandedIds.includes(parameter.local_id)"
            :can-move-up="index > 0"
            :can-move-down="index < draft.parameters.length - 1"
            @toggle="toggleParameter(parameter.local_id)"
            @change="updateParameter(index, $event)"
            @remove="removeParameter(index)"
            @move-up="moveParameter(index, -1)"
            @move-down="moveParameter(index, 1)"
          />

          <div>
            <AppButton variant="secondary" size="sm" icon="plus" @click="addParameter">Add parameter</AppButton>
          </div>

          <JsonViewer :data="schema" collapsible :max-height="420" />
        </div>
      </EditorSection>

      <hr class="hairline" />

      <EditorSection number="4" title="Working directory" description="Where the script runs. A first-class setting — not a script parameter.">
        <div class="editor-page__workdir">
          <div class="editor-page__workdir-path">
            <label>Default path</label>
            <FormField v-model="draft.working_dir" mono icon="folder" placeholder="use global default" />
          </div>

          <hr class="hairline" />

          <div class="editor-page__toggle-row">
            <div>
              <strong>Expose to MCP clients</strong>
              <p>Add working_dir to the tool inputSchema so callers can override it per run.</p>
            </div>
            <ToggleSwitch v-model="draft.working_dir_expose" aria-label="Expose working directory" />
          </div>

          <div v-if="draft.working_dir_expose" class="editor-page__toggle-row">
            <div>
              <strong>Required</strong>
              <p>Callers must provide a working directory when no fallback should be used.</p>
            </div>
            <ToggleSwitch v-model="draft.working_dir_required" aria-label="Require working directory" />
          </div>

          <code>resolution: MCP arg &gt; run override &gt; tool default &gt; global default</code>
        </div>
      </EditorSection>

      <hr class="hairline" />

      <EditorSection number="5" title="Execution" description="Guardrails for how the script runs.">
        <div class="editor-page__execution">
          <div>
            <label>Timeout</label>
            <FormField v-model="draft.timeout_ms" type="number" mono />
            <small>milliseconds</small>
          </div>
          <div>
            <label>Enabled</label>
            <div class="editor-page__enabled">
              <ToggleSwitch v-model="draft.enabled" aria-label="Enabled" />
              <span>{{ draft.enabled ? "Callable by agents" : "Hidden from agents" }}</span>
            </div>
          </div>
        </div>
      </EditorSection>
    </GlassPanel>
  </section>
</template>

<style scoped>
.editor-page {
  max-width: 1280px;
  margin: 0 auto;
  padding: 32px 24px 80px;
}

.editor-page__crumb {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
  color: var(--text-tertiary);
  font-size: 13px;
}

.editor-page__crumb button {
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
}

.editor-page__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 22px;
}

h1 {
  margin: 0;
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 500;
  letter-spacing: -0.015em;
}

.editor-page__actions {
  display: flex;
  gap: 10px;
}

.editor-page__panel {
  padding: 0 28px;
}

.editor-page__grid {
  display: grid;
  gap: 16px;
}

.editor-page__grid.two {
  grid-template-columns: minmax(0, 1fr) 220px;
}

.editor-page__wide {
  grid-column: 1 / -1;
}

label {
  display: block;
  margin-bottom: 7px;
  font-size: 13px;
  font-weight: 500;
}

.editor-page__parameters {
  display: grid;
  gap: 14px;
}

.editor-page__muted {
  margin: 4px 0;
  color: var(--text-secondary);
}

.editor-page__workdir {
  display: grid;
  gap: 16px;
  border: 1px dashed var(--glass-border-strong);
  border-radius: 14px;
  background: var(--glass-surface);
  padding: 18px;
}

.editor-page__workdir-path {
  max-width: 420px;
}

.editor-page__toggle-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.editor-page__toggle-row strong {
  font-size: 15px;
}

.editor-page__toggle-row p {
  max-width: 520px;
  margin: 4px 0 0;
  color: var(--text-secondary);
  font-size: 12px;
}

code {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 12px;
}

.editor-page__execution {
  display: flex;
  flex-wrap: wrap;
  gap: 28px;
}

.editor-page__execution > div:first-child {
  width: 220px;
}

.editor-page__execution small {
  display: block;
  margin-top: 6px;
  color: var(--text-tertiary);
  font-size: 12px;
}

.editor-page__enabled {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 42px;
  color: var(--text-secondary);
}

@media (max-width: 760px) {
  .editor-page {
    padding: 24px 16px 72px;
  }

  .editor-page__header,
  .editor-page__toggle-row {
    align-items: flex-start;
    flex-direction: column;
  }

  .editor-page__grid.two {
    grid-template-columns: 1fr;
  }

  .editor-page__panel {
    padding: 0 18px;
  }
}
</style>
