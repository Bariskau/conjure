<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: boolean;
    ariaLabel?: string;
    size?: "sm" | "md";
  }>(),
  {
    size: "md",
  },
);

defineEmits<{
  "update:modelValue": [value: boolean];
}>();
</script>

<template>
  <button
    type="button"
    class="toggle-switch"
    :class="[`toggle-switch--${size}`, { on: modelValue }]"
    role="switch"
    :aria-checked="modelValue"
    :aria-label="ariaLabel"
    @click="$emit('update:modelValue', !modelValue)"
  >
    <span class="toggle-switch__knob" />
  </button>
</template>

<style scoped>
.toggle-switch {
  position: relative;
  flex-shrink: 0;
  border: 1px solid var(--glass-border);
  border-radius: 999px;
  background: var(--glass-surface);
  cursor: pointer;
  transition: background 0.2s var(--ease), border-color 0.2s var(--ease);
}

.toggle-switch--md {
  width: 38px;
  height: 22px;
}

.toggle-switch--sm {
  width: 32px;
  height: 18px;
}

.toggle-switch.on {
  border-color: transparent;
  background: var(--accent);
}

.toggle-switch__knob {
  position: absolute;
  top: 1px;
  left: 2px;
  border-radius: 999px;
  background: var(--text-secondary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
  transition: left 0.2s var(--ease-out), background 0.2s var(--ease);
}

.toggle-switch--md .toggle-switch__knob {
  width: 18px;
  height: 18px;
}

.toggle-switch--sm .toggle-switch__knob {
  width: 15px;
  height: 15px;
}

.toggle-switch--md.on .toggle-switch__knob {
  left: 18px;
}

.toggle-switch--sm.on .toggle-switch__knob {
  left: 15px;
}

.toggle-switch.on .toggle-switch__knob {
  background: var(--accent-fg);
}
</style>
