<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import ParameterInput from "@/components/editor/ParameterInput.vue";
import AppButton from "@/components/ui/AppButton.vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import FormField from "@/components/ui/FormField.vue";
import GlassPanel from "@/components/ui/GlassPanel.vue";
import SelectMenu from "@/components/ui/SelectMenu.vue";
import StatusBadge from "@/components/ui/StatusBadge.vue";
import StatusDot from "@/components/ui/StatusDot.vue";
import { draftFromTool } from "@/domain/defaults";
import { createParamsFromDraftValues, validateFormData } from "@/domain/schema";
import type { ExecutionResult, ToolDraft } from "@/domain/types";
import { useConjureStore } from "@/stores/conjure";

const route = useRoute();
const router = useRouter();
const store = useConjureStore();

const draft = ref<ToolDraft | null>(null);
const values = ref<Record<string, unknown>>({});
const workingDir = ref("");
const errors = ref<Record<string, string>>({});
const result = ref<ExecutionResult | null>(null);
const running = ref(false);

const selectedToolId = computed(() => {
  return (route.params.toolId as string | undefined) || store.tools[0]?.id;
});

const toolOptions = computed(() => {
  return store.tools.map((tool) => ({ value: tool.id, label: tool.name }));
});

const resolvedWorkingDir = computed(() => {
  return workingDir.value.trim() || draft.value?.working_dir || store.settings.default_working_dir || "";
});

watch(selectedToolId, loadTool, { immediate: true });

async function loadTool(): Promise<void> {
  if (!selectedToolId.value) {
    return;
  }

  const tool = await store.fetchTool(selectedToolId.value);
  draft.value = draftFromTool(tool);
  values.value = initialValues(draft.value);
  workingDir.value = draft.value.working_dir ?? store.settings.default_working_dir ?? "";
  errors.value = {};
  result.value = null;
}

async function runSelectedTool(): Promise<void> {
  if (!draft.value?.id || !validateInputs()) {
    return;
  }

  running.value = true;
  result.value = null;
  try {
    const params = createParamsFromDraftValues(draft.value.parameters, values.value);
    result.value = await store.runTool(draft.value.id, params, workingDir.value.trim() || null);
  } catch (error) {
    store.toast(error instanceof Error ? error.message : "Run failed", "error");
  } finally {
    running.value = false;
  }
}

function validateInputs(): boolean {
  if (!draft.value) {
    return false;
  }

  const params = createParamsFromDraftValues(draft.value.parameters, values.value);
  const validationData = { ...params };

  if (draft.value.working_dir_expose && resolvedWorkingDir.value) {
    validationData.working_dir = resolvedWorkingDir.value;
  }

  const validation = validateFormData(draft.value, validationData);
  errors.value = validation.errors;
  return validation.valid;
}

function clearError(name: string): void {
  if (errors.value[name]) {
    const nextErrors = { ...errors.value };
    delete nextErrors[name];
    errors.value = nextErrors;
  }
}

function pickTool(toolId: string | number | boolean): void {
  router.push({ name: "test", params: { toolId: String(toolId) } });
}

function initialValues(tool: ToolDraft): Record<string, unknown> {
  return tool.parameters.reduce<Record<string, unknown>>((result, parameter) => {
    result[parameter.name] = parameter.default_value;
    return result;
  }, {});
}
</script>

<template>
  <section class="test-page page-enter">
    <div class="test-page__crumb">
      <button type="button" @click="router.push({ name: 'tools' })">Tools</button>
      <AppIcon name="chevron-right" :size="13" />
      <span>Test</span>
    </div>

    <div class="test-page__header">
      <div>
        <h1>Test</h1>
        <p>Run the tool with real parameters and watch it stream.</p>
      </div>
      <div class="test-page__picker">
        <SelectMenu :model-value="selectedToolId" :options="toolOptions" @update:model-value="pickTool" />
      </div>
    </div>

    <div v-if="draft" class="test-page__grid">
      <GlassPanel class="test-page__params">
        <div class="test-page__panel-title">
          <h2>Parameters</h2>
          <span>{{ draft.parameters.length }} inputs</span>
        </div>

        <div class="test-page__workdir">
          <div class="test-page__workdir-label">
            <label><AppIcon name="folder" :size="15" />Working directory</label>
            <span>{{ draft.working_dir_required ? "required" : "per run" }}</span>
          </div>
          <FormField
            v-model="workingDir"
            mono
            :invalid="Boolean(errors.working_dir)"
            :placeholder="draft.working_dir || store.settings.default_working_dir || 'process default'"
            @update:model-value="clearError('working_dir')"
            @blur="validateInputs"
          />
          <p v-if="errors.working_dir" class="test-page__error">{{ errors.working_dir }}</p>
        </div>

        <div class="test-page__inputs">
          <p v-if="draft.parameters.length === 0" class="test-page__muted">This tool takes no parameters.</p>
          <ParameterInput
            v-for="parameter in draft.parameters"
            :key="parameter.local_id"
            v-model="values[parameter.name]"
            :parameter="parameter"
            :error="errors[parameter.name]"
            @update:model-value="clearError(parameter.name)"
            @blur="validateInputs"
          />
        </div>

        <hr class="hairline" />

        <div class="test-page__buttons">
          <AppButton v-if="running" variant="secondary" icon="spinner" full disabled>Running</AppButton>
          <AppButton v-else variant="primary" icon="play" full @click="runSelectedTool">Run</AppButton>
          <AppButton v-if="draft.id" variant="tertiary" icon="pencil" title="Edit tool" @click="router.push({ name: 'tool-edit', params: { toolId: draft.id } })" />
        </div>
      </GlassPanel>

      <GlassPanel class="test-page__console">
        <div class="test-page__console-head">
          <span class="test-page__dots"><i /><i /><i /></span>
          <span>{{ draft.name }} · output</span>
          <StatusBadge v-if="running" status="running" compact />
          <StatusBadge v-else-if="result" :status="result.status" compact />
        </div>

        <div class="scroll test-page__console-body">
          <div v-if="!running && !result" class="test-page__ready">
            <div>
              <AppIcon name="play" :size="22" />
              <span>Ready to run</span>
              <small>Press Run to stream live output here.</small>
            </div>
          </div>

          <template v-else>
            <p v-if="running" class="line-in">$ conjure run {{ draft.name }}</p>
            <pre v-if="result?.stdout" class="line-in stdout">{{ result.stdout }}</pre>
            <pre v-if="result?.stderr" class="line-in stderr">{{ result.stderr }}</pre>
            <span v-if="running" class="pulse-dot test-page__cursor" />
          </template>
        </div>

        <div v-if="result" class="test-page__result">
          <span><StatusDot :status="result.status" />exit {{ result.exit_code ?? "-" }}</span>
          <span>{{ result.duration_ms }}ms</span>
          <span v-if="result.stdout_truncated || result.stderr_truncated">output truncated</span>
          <AppButton variant="secondary" size="sm" icon="play" @click="runSelectedTool">Run again</AppButton>
        </div>
      </GlassPanel>
    </div>
  </section>
</template>

<style scoped>
.test-page {
  max-width: 1280px;
  margin: 0 auto;
  padding: 32px 24px 80px;
}

.test-page__crumb,
.test-page__workdir-label,
.test-page__panel-title,
.test-page__console-head,
.test-page__result {
  display: flex;
  align-items: center;
}

.test-page__crumb {
  gap: 8px;
  margin-bottom: 14px;
  color: var(--text-tertiary);
  font-size: 13px;
}

.test-page__crumb button {
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
}

.test-page__header {
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

.test-page__header p {
  margin: 8px 0 0;
  color: var(--text-secondary);
}

.test-page__picker {
  width: 240px;
}

.test-page__grid {
  display: grid;
  grid-template-columns: minmax(300px, 360px) minmax(0, 1fr);
  gap: 18px;
  align-items: start;
}

.test-page__params {
  display: grid;
  gap: 18px;
  padding: 20px;
}

.test-page__panel-title {
  justify-content: space-between;
}

h2 {
  margin: 0;
  font-size: 16px;
}

.test-page__panel-title span,
.test-page__workdir-label span,
.test-page__muted {
  color: var(--text-tertiary);
  font-size: 12px;
}

.test-page__workdir {
  border: 1px dashed var(--glass-border-strong);
  border-radius: 12px;
  background: var(--glass-surface);
  padding: 12px;
}

.test-page__workdir-label {
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 7px;
}

label {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 13px;
  font-weight: 500;
}

.test-page__inputs {
  display: grid;
  gap: 16px;
}

.test-page__buttons {
  display: flex;
  gap: 10px;
}

.test-page__error {
  margin: 6px 0 0;
  color: rgba(var(--error-tint), 1);
  font-size: 12px;
  font-weight: 500;
}

.test-page__console {
  display: flex;
  overflow: hidden;
  min-height: 440px;
  flex-direction: column;
  background: var(--console-bg);
}

.test-page__console-head {
  gap: 12px;
  border-bottom: 1px solid var(--divider);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12.5px;
  padding: 12px 16px;
}

.test-page__console-head > span:nth-child(2) {
  flex: 1;
}

.test-page__dots {
  display: flex;
  gap: 6px;
}

.test-page__dots i {
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: var(--glass-border-strong);
}

.test-page__console-body {
  position: relative;
  flex: 1;
  overflow: auto;
  min-height: 300px;
  max-height: 460px;
  padding: 14px 18px;
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.6;
}

.test-page__ready {
  display: grid;
  height: 100%;
  place-items: center;
  color: var(--text-tertiary);
  text-align: center;
}

.test-page__ready svg {
  margin: 0 auto 12px;
  opacity: 0.6;
}

.test-page__ready span {
  display: block;
  font-family: var(--font-ui);
}

.test-page__ready small {
  display: block;
  margin-top: 4px;
  font-size: 12px;
}

pre,
.test-page__console-body p {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.stdout {
  color: var(--text-secondary);
}

.stderr {
  color: rgba(var(--error-tint), 1);
  font-weight: 600;
}

.test-page__cursor {
  display: inline-block;
  width: 8px;
  height: 15px;
  margin-top: 2px;
  background: var(--text-primary);
  vertical-align: text-bottom;
}

.test-page__result {
  gap: 18px;
  border-top: 1px solid var(--divider);
  background: var(--glass-surface);
  color: var(--text-secondary);
  padding: 13px 18px;
}

.test-page__result span {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-mono);
  font-size: 13px;
}

.test-page__result span:first-child {
  color: var(--text-primary);
  font-weight: 600;
}

.test-page__result .app-button {
  margin-left: auto;
}

@media (max-width: 920px) {
  .test-page__grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .test-page {
    padding: 24px 16px 72px;
  }

  .test-page__header {
    align-items: flex-start;
    flex-direction: column;
  }

  .test-page__picker {
    width: 100%;
  }
}
</style>
