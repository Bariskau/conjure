<script setup lang="ts">
import { computed } from "vue";

import AppIcon from "./AppIcon.vue";

const props = withDefaults(
  defineProps<{
    status?: "success" | "error" | "timeout" | "running" | "idle";
    compact?: boolean;
  }>(),
  {
    status: "idle",
    compact: false,
  },
);

const label = computed(() => {
  return {
    success: "Success",
    error: "Failed",
    running: "Running",
    timeout: "Timed out",
    idle: "Idle",
  }[props.status];
});
</script>

<template>
  <span class="status-badge" :class="[`status-badge--${status}`, { compact }]">
    <AppIcon v-if="status === 'running'" name="spinner" :size="12" class="spin" />
    <AppIcon v-else-if="status === 'success'" name="check" :size="12" :stroke-width="2.2" />
    <AppIcon v-else-if="status === 'error'" name="x" :size="12" :stroke-width="2.4" />
    <AppIcon v-else-if="status === 'timeout'" name="clock" :size="12" />
    <span v-else class="status-badge__dot" />
    <span>{{ label }}</span>
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  border: 1px solid var(--glass-border);
  border-radius: 999px;
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 500;
  line-height: 1;
  padding: 4px 10px 4px 8px;
  white-space: nowrap;
}

.status-badge.compact {
  padding: 3px 9px 3px 8px;
}

.status-badge--idle {
  background: rgba(28,28,30,0.04);
  color: var(--text-tertiary);
}

.status-badge--success {
  border-color: rgba(120,170,40,0.45);
  background: rgba(190,242,100,0.30);
  color: var(--text-primary);
}

.status-badge--error {
  border-color: var(--error-ring);
  background: rgba(201,64,64,0.12);
  color: rgba(var(--error-tint), 1);
  font-weight: 600;
}

.status-badge--running {
  background: rgba(28,28,30,0.06);
  color: var(--text-primary);
}

.status-badge--timeout {
  background: rgba(28,28,30,0.05);
  color: var(--text-secondary);
}

.status-badge__dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: var(--text-tertiary);
}
</style>
