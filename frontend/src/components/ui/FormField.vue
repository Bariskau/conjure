<script setup lang="ts">
import AppIcon, { type IconName } from "./AppIcon.vue";

withDefaults(
  defineProps<{
    modelValue?: string | number | null;
    placeholder?: string;
    type?: string;
    icon?: IconName;
    mono?: boolean;
    invalid?: boolean;
    min?: string | number;
    max?: string | number;
    disabled?: boolean;
  }>(),
  {
    type: "text",
  },
);

defineEmits<{
  "update:modelValue": [value: string];
  blur: [event: FocusEvent];
  keydown: [event: KeyboardEvent];
}>();
</script>

<template>
  <div class="form-field">
    <span v-if="icon" class="form-field__icon">
      <AppIcon :name="icon" :size="16" />
    </span>
    <input
      class="field"
      :class="{ invalid, mono, 'has-icon': icon }"
      :type="type"
      :value="modelValue ?? ''"
      :placeholder="placeholder"
      :min="min"
      :max="max"
      :disabled="disabled"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @blur="$emit('blur', $event)"
      @keydown="$emit('keydown', $event)"
    />
  </div>
</template>

<style scoped>
.form-field {
  position: relative;
  width: 100%;
}

.form-field__icon {
  position: absolute;
  top: 50%;
  left: 11px;
  z-index: 1;
  color: var(--text-tertiary);
  pointer-events: none;
  transform: translateY(-50%);
}

.has-icon {
  padding-left: 36px;
}

.mono {
  font-family: var(--font-mono);
}

.field:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
