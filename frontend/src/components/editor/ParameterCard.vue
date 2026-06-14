<script setup lang="ts">
import { computed } from "vue";

import { defaultValueForType } from "@/domain/defaults";
import type { ParameterDraft, ParameterType } from "@/domain/types";

import AppButton from "@/components/ui/AppButton.vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import FormField from "@/components/ui/FormField.vue";
import SelectMenu from "@/components/ui/SelectMenu.vue";
import TextAreaField from "@/components/ui/TextAreaField.vue";
import ToggleSwitch from "@/components/ui/ToggleSwitch.vue";

import OptionChips from "./OptionChips.vue";

const props = defineProps<{
  parameter: ParameterDraft;
  index: number;
  open: boolean;
  canMoveUp: boolean;
  canMoveDown: boolean;
}>();

const emit = defineEmits<{
  change: [parameter: ParameterDraft];
  remove: [];
  toggle: [];
  moveUp: [];
  moveDown: [];
}>();

const typeOptions: ParameterType[] = ["string", "number", "boolean", "enum", "path"];

const summary = computed(() => summarizeParameter(props.parameter));

function setField<K extends keyof ParameterDraft>(key: K, value: ParameterDraft[K]): void {
  emit("change", { ...props.parameter, [key]: value });
}

function setType(type: ParameterType): void {
  emit("change", {
    ...props.parameter,
    type,
    default_value: defaultValueForType(type),
    validation: {
      ...props.parameter.validation,
      min: null,
      max: null,
      regex: null,
      format: null,
      integer: false,
      enum_values: type === "enum" ? props.parameter.validation.enum_values : [],
    },
  });
}

function setValidation(key: keyof ParameterDraft["validation"], value: unknown): void {
  emit("change", {
    ...props.parameter,
    validation: { ...props.parameter.validation, [key]: value },
  });
}

function setNumberValidation(key: "min" | "max", value: string): void {
  setValidation(key, value.trim() ? Number(value) : null);
}

function summarizeParameter(parameter: ParameterDraft): string {
  if (!parameter.name) {
    return "unnamed parameter";
  }

  const bits = [parameter.required ? "required" : "optional", parameter.type];
  if (parameter.type === "enum") {
    bits.push(`${parameter.validation.enum_values.length} options`);
  }

  if (parameter.default_value != null && parameter.default_value !== "") {
    bits.push(`default ${String(parameter.default_value)}`);
  }

  return bits.join(" - ");
}
</script>

<template>
  <div class="parameter-card glass">
    <div class="parameter-card__summary" @click="$emit('toggle')">
      <div class="parameter-card__move" @click.stop>
        <AppButton variant="tertiary" size="sm" icon="chevron-down" title="Move up" :disabled="!canMoveUp" @click="$emit('moveUp')" />
        <AppButton variant="tertiary" size="sm" icon="chevron-down" title="Move down" :disabled="!canMoveDown" @click="$emit('moveDown')" />
      </div>

      <div class="parameter-card__name" @click.stop>
        <FormField
          mono
          :model-value="parameter.name"
          placeholder="parameter_name"
          @update:model-value="setField('name', $event.replace(/\\s+/g, '_'))"
        />
      </div>

      <div class="parameter-card__type" @click.stop>
        <SelectMenu :model-value="parameter.type" :options="typeOptions" @update:model-value="setType($event as ParameterType)" />
      </div>

      <div class="parameter-card__required" @click.stop>
        <span>Required</span>
        <ToggleSwitch :model-value="parameter.required" size="sm" aria-label="Required" @update:model-value="setField('required', $event)" />
      </div>

      <AppButton variant="tertiary" size="sm" icon="trash" title="Remove" @click.stop="$emit('remove')" />
      <button type="button" class="parameter-card__toggle" :aria-expanded="open" @click.stop="$emit('toggle')">
        <AppIcon name="chevron-down" :size="16" :class="{ closed: !open }" />
      </button>
    </div>

    <p v-if="!open" class="parameter-card__closed" @click="$emit('toggle')">{{ summary }}</p>

    <div class="parameter-card__body" :class="{ open }">
      <div class="parameter-card__body-inner">
        <hr class="hairline" />
        <div class="parameter-card__details">
          <div class="parameter-card__fields">
            <div>
              <label>Description</label>
              <TextAreaField
                :rows="2"
                :model-value="parameter.description"
                placeholder="Shown to MCP clients"
                @update:model-value="setField('description', $event)"
              />
            </div>

            <div v-if="parameter.type === 'boolean'" class="parameter-card__inline">
              <label>Default state</label>
              <ToggleSwitch
                :model-value="parameter.default_value === true"
                aria-label="Default state"
                @update:model-value="setField('default_value', $event)"
              />
            </div>

            <div v-else-if="parameter.type === 'enum'">
              <label>Default value</label>
              <SelectMenu
                :model-value="String(parameter.default_value ?? '')"
                :options="parameter.validation.enum_values.length ? parameter.validation.enum_values : ['']"
                @update:model-value="setField('default_value', $event)"
              />
            </div>

            <div v-else>
              <label>Default value</label>
              <FormField
                :type="parameter.type === 'number' ? 'number' : 'text'"
                mono
                :model-value="parameter.default_value as string | number | null"
                placeholder="default"
                @update:model-value="setField('default_value', $event)"
              />
            </div>

            <OptionChips
              v-if="parameter.type === 'enum'"
              :options="parameter.validation.enum_values"
              @update:options="setValidation('enum_values', $event)"
            />

            <div v-if="parameter.type === 'number'" class="parameter-card__grid">
              <div>
                <label>Min</label>
                <FormField
                  type="number"
                  mono
                  :model-value="parameter.validation.min"
                  placeholder="none"
                  @update:model-value="setNumberValidation('min', $event)"
                />
              </div>
              <div>
                <label>Max</label>
                <FormField
                  type="number"
                  mono
                  :model-value="parameter.validation.max"
                  placeholder="none"
                  @update:model-value="setNumberValidation('max', $event)"
                />
              </div>
              <div class="parameter-card__inline">
                <label>Integer</label>
                <ToggleSwitch
                  :model-value="parameter.validation.integer"
                  aria-label="Integer only"
                  @update:model-value="setValidation('integer', $event)"
                />
              </div>
            </div>

            <div v-if="parameter.type === 'string' || parameter.type === 'path'" class="parameter-card__grid">
              <div>
                <label>Min length</label>
                <FormField
                  type="number"
                  mono
                  :model-value="parameter.validation.min"
                  placeholder="none"
                  @update:model-value="setNumberValidation('min', $event)"
                />
              </div>
              <div>
                <label>Max length</label>
                <FormField
                  type="number"
                  mono
                  :model-value="parameter.validation.max"
                  placeholder="none"
                  @update:model-value="setNumberValidation('max', $event)"
                />
              </div>
              <div>
                <label>Pattern</label>
                <FormField
                  mono
                  :model-value="parameter.validation.regex"
                  placeholder="^[a-z0-9-]+$"
                  @update:model-value="setValidation('regex', $event)"
                />
              </div>
              <div v-if="parameter.type === 'string'">
                <label>Format</label>
                <FormField
                  mono
                  :model-value="parameter.validation.format"
                  placeholder="email, uri, date-time"
                  @update:model-value="setValidation('format', $event)"
                />
              </div>
            </div>
          </div>

          <aside>
            <span>Preview</span>
            <div class="parameter-card__preview">
              <strong>{{ parameter.name || "unnamed" }}</strong>
              <small>{{ summary }}</small>
            </div>
          </aside>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.parameter-card {
  padding: 12px 16px;
}

.parameter-card__summary {
  display: grid;
  grid-template-columns: auto minmax(180px, 1fr) 132px auto auto auto;
  align-items: center;
  gap: 12px;
  cursor: pointer;
}

.parameter-card__move {
  display: grid;
  gap: 2px;
}

.parameter-card__move :deep(.app-button:first-child svg) {
  transform: rotate(180deg);
}

.parameter-card__required,
.parameter-card__inline {
  display: flex;
  align-items: center;
  gap: 8px;
}

.parameter-card__required span,
label {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
}

.parameter-card__toggle {
  display: grid;
  width: 30px;
  height: 30px;
  place-items: center;
  border-radius: 8px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.parameter-card__toggle .closed {
  transform: rotate(-90deg);
}

.parameter-card__closed {
  overflow: hidden;
  margin: 7px 0 0 46px;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 12.5px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.parameter-card__body {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.28s var(--ease-out);
}

.parameter-card__body.open {
  grid-template-rows: 1fr;
}

.parameter-card__body-inner {
  overflow: hidden;
  min-height: 0;
}

.parameter-card__details {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 230px;
  gap: 22px;
  align-items: start;
  padding: 14px 0 4px;
}

.parameter-card__fields {
  display: grid;
  gap: 14px;
}

.parameter-card__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

label {
  display: block;
  margin-bottom: 7px;
  color: var(--text-primary);
  font-size: 13px;
}

aside {
  align-self: stretch;
  border-left: 1px solid var(--divider);
  padding-left: 22px;
}

aside > span {
  display: block;
  margin-bottom: 10px;
  color: var(--text-tertiary);
  font-size: 12px;
  font-weight: 400;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.parameter-card__preview {
  border: 1px dashed var(--glass-border-strong);
  border-radius: 12px;
  background: var(--glass-surface);
  padding: 12px;
}

.parameter-card__preview strong {
  display: block;
  font-family: var(--font-mono);
  font-size: 13px;
}

.parameter-card__preview small {
  display: block;
  margin-top: 6px;
  color: var(--text-tertiary);
}

@media (max-width: 880px) {
  .parameter-card__summary {
    grid-template-columns: 1fr auto;
  }

  .parameter-card__move,
  .parameter-card__type,
  .parameter-card__required {
    grid-column: 1 / -1;
  }

  .parameter-card__details,
  .parameter-card__grid {
    grid-template-columns: 1fr;
  }

  aside {
    border-left: 0;
    padding-left: 0;
  }
}
</style>
