<script setup lang="ts">
withDefaults(
  defineProps<{
    status?: "success" | "error" | "timeout" | "running" | "idle";
    size?: number;
  }>(),
  {
    status: "idle",
    size: 9,
  },
);
</script>

<template>
  <span class="status-dot" :class="[`status-dot--${status}`]" :style="{ width: `${size}px`, height: `${size}px` }" />
</template>

<style scoped>
.status-dot {
  display: inline-block;
  flex-shrink: 0;
  border-radius: 999px;
}

.status-dot--success {
  background: var(--text-primary);
}

.status-dot--running {
  background: var(--text-primary);
  animation: pulse-dot 1.2s var(--ease) infinite;
}

.status-dot--idle {
  background: var(--text-tertiary);
}

.status-dot--timeout {
  border: 1.5px dashed var(--text-secondary);
  background: transparent;
}

.status-dot--error {
  border: 1.5px solid rgba(var(--error-tint), 1);
  background: transparent;
  box-shadow: 0 0 0 2px var(--error-ring);
}
</style>
