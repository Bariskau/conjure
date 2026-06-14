<script setup lang="ts">
import { ref } from "vue";

import { formatRelativeTime, formatTimeout } from "@/domain/formatters";
import type { ToolSummary } from "@/domain/types";

import AppButton from "@/components/ui/AppButton.vue";
import GlassPanel from "@/components/ui/GlassPanel.vue";
import StatusDot from "@/components/ui/StatusDot.vue";
import ToggleSwitch from "@/components/ui/ToggleSwitch.vue";

defineProps<{
  tool: ToolSummary;
}>();

defineEmits<{
  edit: [tool: ToolSummary];
  test: [tool: ToolSummary];
  toggle: [tool: ToolSummary];
  delete: [tool: ToolSummary];
}>();

const hover = ref(false);
</script>

<template>
  <GlassPanel
    interactive
    :inactive="!tool.enabled"
    class="tool-card"
    @mouseenter="hover = true"
    @mouseleave="hover = false"
  >
    <div class="tool-card__top">
      <div class="tool-card__identity">
        <h3>{{ tool.name }}</h3>
      </div>
      <span class="tool-card__enabled" :class="{ disabled: !tool.enabled }">
        <span />
        {{ tool.enabled ? "Enabled" : "Disabled" }}
      </span>
    </div>

    <p>{{ tool.description }}</p>

    <div class="tool-card__footer">
      <div class="tool-card__meta" :class="{ hidden: hover }">
        <span>
          <StatusDot :status="tool.last_run_at ? 'success' : 'idle'" :size="7" />
          {{ formatRelativeTime(tool.last_run_at) }}
        </span>
        <span>{{ formatTimeout(tool.timeout_ms) }}</span>
      </div>

      <div class="tool-card__actions" :class="{ visible: hover }">
        <AppButton variant="secondary" size="sm" icon="play" @click="$emit('test', tool)">Test</AppButton>
        <AppButton variant="tertiary" size="sm" icon="pencil" @click="$emit('edit', tool)">Edit</AppButton>
        <AppButton variant="tertiary" size="sm" icon="trash" title="Delete" @click="$emit('delete', tool)" />
        <span class="tool-card__action-spacer" />
        <ToggleSwitch :model-value="tool.enabled" size="sm" aria-label="Toggle tool" @update:model-value="$emit('toggle', tool)" />
      </div>
    </div>
  </GlassPanel>
</template>

<style scoped>
.tool-card {
  display: flex;
  width: 100%;
  height: 100%;
  min-height: var(--tool-card-min-height);
  flex-direction: column;
  padding: var(--tool-card-padding);
}

.tool-card__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.tool-card__identity {
  min-width: 0;
}

h3 {
  overflow: hidden;
  margin: 0;
  font-family: var(--font-mono);
  font-size: 15px;
  font-weight: 500;
  line-height: normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tool-card__enabled {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  border-radius: 999px;
  border: 1px solid transparent;
  background: var(--accent);
  color: var(--accent-fg);
  font-size: 12px;
  font-weight: 500;
  line-height: normal;
  padding: 4px 11px 4px 9px;
  white-space: nowrap;
}

.tool-card__enabled.disabled {
  border-color: var(--glass-border);
  background: rgba(28,28,30,0.05);
  color: var(--text-tertiary);
}

.tool-card__enabled span {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: currentColor;
}

p {
  display: -webkit-box;
  overflow: hidden;
  margin: 12px 0 0;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  color: var(--text-secondary);
  text-wrap: pretty;
}

.tool-card__footer {
  position: relative;
  height: 30px;
  margin-top: auto;
  padding-top: 14px;
}

.tool-card__meta,
.tool-card__actions {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  gap: 14px;
  transition: opacity 0.2s var(--ease);
}

.tool-card__meta {
  color: var(--text-tertiary);
  font-size: 12px;
}

.tool-card__meta.hidden {
  opacity: 0;
}

.tool-card__meta span {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.tool-card__actions {
  gap: 8px;
  opacity: 0;
  pointer-events: none;
}

.tool-card__actions.visible {
  opacity: 1;
  pointer-events: auto;
}

.tool-card__action-spacer {
  flex: 1;
}

@media (max-width: 760px) {
  .tool-card__actions {
    position: static;
    opacity: 1;
    pointer-events: auto;
  }

  .tool-card__meta {
    display: none;
  }
}
</style>
