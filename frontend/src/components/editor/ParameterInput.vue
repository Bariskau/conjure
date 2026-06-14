<script setup lang="ts">
import { computed } from "vue";

import type { ParameterDraft } from "@/domain/types";

import FormField from "@/components/ui/FormField.vue";
import SelectMenu from "@/components/ui/SelectMenu.vue";
import ToggleSwitch from "@/components/ui/ToggleSwitch.vue";

const props = defineProps<{
  parameter: ParameterDraft;
  modelValue?: unknown;
  error?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: unknown];
  blur: [];
}>();

const enumOptions = computed(() => props.parameter.validation.enum_values);
const booleanValue = computed(() => props.modelValue === true || props.modelValue === "true");
</script>

<template>
  <div class="parameter-input" @focusout="$emit('blur')">
    <div class="parameter-input__label">
      <label>{{ parameter.name || "unnamed" }}</label>
      <span>{{ parameter.required ? "required" : parameter.type }}</span>
    </div>

    <div v-if="parameter.type === 'boolean'" class="parameter-input__boolean">
      <ToggleSwitch :model-value="booleanValue" @update:model-value="$emit('update:modelValue', $event)" />
      <span>{{ booleanValue ? "true" : "false" }}</span>
    </div>

    <SelectMenu
      v-else-if="parameter.type === 'enum'"
      :model-value="String(modelValue ?? enumOptions[0] ?? '')"
      :options="enumOptions"
      :invalid="Boolean(error)"
      @update:model-value="$emit('update:modelValue', $event)"
    />

    <FormField
      v-else
      mono
      :type="parameter.type === 'number' ? 'number' : 'text'"
      :model-value="modelValue as string | number | null"
      :placeholder="String(parameter.default_value ?? parameter.type)"
      :invalid="Boolean(error)"
      @update:model-value="$emit('update:modelValue', $event)"
      @blur="$emit('blur')"
    />

    <p v-if="error" class="parameter-input__error">{{ error }}</p>
    <p v-else-if="parameter.description" class="parameter-input__hint">{{ parameter.description }}</p>
  </div>
</template>

<style scoped>
.parameter-input__label {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 7px;
}

label {
  overflow: hidden;
  font-size: 13px;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.parameter-input__label span,
.parameter-input__hint {
  color: var(--text-tertiary);
  font-size: 12px;
}

.parameter-input__boolean {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 38px;
  color: var(--text-secondary);
}

.parameter-input__error {
  margin: 6px 0 0;
  color: rgba(var(--error-tint), 1);
  font-size: 12px;
  font-weight: 500;
}

.parameter-input__hint {
  margin: 6px 0 0;
}
</style>
