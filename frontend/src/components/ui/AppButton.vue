<script setup lang="ts">
import AppIcon, { type IconName } from "./AppIcon.vue";

withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "tertiary" | "danger";
    size?: "sm" | "md" | "lg";
    icon?: IconName;
    iconRight?: IconName;
    disabled?: boolean;
    full?: boolean;
    title?: string;
    type?: "button" | "submit";
  }>(),
  {
    variant: "secondary",
    size: "md",
    type: "button",
  },
);

defineEmits<{
  click: [event: MouseEvent];
}>();
</script>

<template>
  <button
    :type="type"
    :title="title"
    :disabled="disabled"
    class="app-button"
    :class="[`app-button--${variant}`, `app-button--${size}`, { 'app-button--full': full, 'app-button--icon-only': icon && !$slots.default }]"
    @click="$emit('click', $event)"
  >
    <AppIcon v-if="icon" :name="icon" :size="size === 'sm' ? 15 : 16" />
    <span v-if="$slots.default" class="app-button__label"><slot /></span>
    <AppIcon v-if="iconRight" :name="iconRight" :size="size === 'sm' ? 15 : 16" />
  </button>
</template>

<style scoped>
.app-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px solid transparent;
  border-radius: var(--radius-control);
  font-family: var(--font-ui);
  font-weight: 500;
  line-height: 1;
  white-space: nowrap;
  cursor: pointer;
  transition: transform 0.12s var(--ease), background 0.18s var(--ease), border-color 0.18s var(--ease);
}

.app-button:active:not(:disabled) {
  transform: scale(0.975);
}

.app-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.app-button--sm {
  padding: 7px 12px;
  font-size: 13px;
}

.app-button--md {
  padding: 9px 16px;
  font-size: 14px;
}

.app-button--lg {
  padding: 12px 22px;
  font-size: 15px;
}

.app-button--full {
  width: 100%;
}

.app-button--primary {
  background: var(--button-primary-bg);
  color: var(--button-primary-fg);
  box-shadow: 0 1px 0 rgba(255, 255, 255, 0.4) inset, 0 8px 20px -8px rgba(0, 0, 0, 0.6);
}

.app-button--primary:hover:not(:disabled) {
  background: var(--button-primary-hover);
}

.app-button--secondary {
  border-color: var(--glass-border);
  background: var(--glass-surface);
  color: var(--text-primary);
  backdrop-filter: blur(12px);
}

.app-button--secondary:hover:not(:disabled) {
  border-color: var(--glass-border-strong);
  background: var(--glass-surface-strong);
}

.app-button--tertiary {
  background: transparent;
  color: var(--text-secondary);
}

.app-button--tertiary:hover:not(:disabled) {
  background: var(--hover-wash);
  color: var(--text-primary);
}

.app-button--danger {
  border-color: var(--error-ring);
  background: var(--glass-surface);
  color: rgba(var(--error-tint), 1);
}

.app-button__label {
  overflow: hidden;
  text-overflow: ellipsis;
}

.app-button--icon-only.app-button--sm {
  padding: 8px;
}

.app-button--icon-only.app-button--md {
  padding: 10px;
}
</style>
