<script setup lang="ts">
import { computed } from "vue";

import { formatDuration, formatRelativeTime } from "@/domain/formatters";
import type { CallLog } from "@/domain/types";

import AppIcon from "@/components/ui/AppIcon.vue";
import StatusDot from "@/components/ui/StatusDot.vue";

const props = defineProps<{
  log: CallLog;
  expanded: boolean;
}>();

defineEmits<{
  toggle: [];
}>();

const relativeStartedAt = computed(() => formatRelativeTime(props.log.started_at));
const startedAtLabel = computed(() => new Date(props.log.started_at).toLocaleString());
const exitLabel = computed(() => (props.log.exit_code == null ? "—" : `exit ${props.log.exit_code}`));

function statusLabel(status: CallLog["status"]): string {
  return {
    success: "Success",
    error: "Failed",
    timeout: "Timed out",
  }[status];
}
</script>

<template>
  <div class="log-row glass" :class="{ expanded }">
    <button type="button" class="log-row__summary" @click="$emit('toggle')">
      <AppIcon name="chevron-right" :size="15" class="log-row__chevron" :class="{ expanded }" />
      <span class="log-row__tool">{{ log.tool_name }}</span>
      <span class="log-row__status" :class="{ error: log.status === 'error' || log.status === 'timeout' }">
        <StatusDot :status="log.status" :size="8" />
        <span>{{ statusLabel(log.status) }}</span>
      </span>
      <span>{{ formatDuration(log.duration_ms) }}</span>
      <span :class="{ error: log.exit_code && log.exit_code !== 0 }">{{ exitLabel }}</span>
      <span class="log-row__when">{{ relativeStartedAt }}</span>
    </button>

    <div v-if="expanded" class="log-row__details page-enter">
      <hr class="hairline" />
      <dl>
        <dt>Run id</dt>
        <dd>{{ log.id }}</dd>
        <dt>Started</dt>
        <dd>{{ startedAtLabel }}</dd>
        <dt>Parameters</dt>
        <dd>
          <span v-for="(value, key) in log.params_json" :key="key" class="log-row__param">
            {{ key }}=<strong>{{ value }}</strong>
          </span>
        </dd>
      </dl>

      <div class="log-row__streams">
        <div>
          <span>stdout</span>
          <pre class="scroll">{{ log.stdout || "(empty)" }}</pre>
        </div>
        <div v-if="log.stderr">
          <span class="error">stderr</span>
          <pre class="scroll error">{{ log.stderr }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-row {
  overflow: hidden;
  border-radius: 12px;
}

.log-row.expanded {
  border-color: var(--glass-border-strong);
}

.log-row__summary {
  display: grid;
  width: 100%;
  grid-template-columns: 20px 1.4fr 1fr 0.8fr 0.8fr 0.6fr;
  align-items: center;
  gap: 14px;
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  line-height: normal;
  padding: 13px 16px;
  text-align: left;
}

.log-row__chevron {
  color: var(--text-tertiary);
  transition: transform 0.2s var(--ease);
}

.log-row__chevron.expanded {
  transform: rotate(90deg);
}

.log-row__tool {
  overflow: hidden;
  font-family: var(--font-mono);
  font-size: 13.5px;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-row__status {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  color: var(--text-secondary);
  font-size: 12.5px;
  font-weight: 500;
}

.log-row__summary > span:not(.log-row__tool):not(.log-row__status) {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12.5px;
}

.log-row__when {
  color: var(--text-tertiary) !important;
  font-family: var(--font-ui) !important;
  font-size: 12px !important;
  text-align: right;
}

.error {
  color: rgba(var(--error-tint), 1) !important;
}

.log-row__details {
  padding: 0 16px 16px 50px;
}

dl {
  display: grid;
  grid-template-columns: 150px minmax(0, 1fr);
  gap: 10px 18px;
  margin: 14px 0 16px;
}

dt {
  color: var(--text-tertiary);
  font-size: 12px;
}

dd {
  margin: 0;
  font-family: var(--font-mono);
  font-size: 12.5px;
}

.log-row__param {
  display: inline-flex;
  gap: 4px;
  border: 1px solid var(--glass-border);
  border-radius: 999px;
  color: var(--text-secondary);
  margin: 0 6px 6px 0;
  padding: 3px 9px;
}

.log-row__param strong {
  color: var(--text-primary);
  font-weight: 500;
}

.log-row__streams {
  display: grid;
  gap: 12px;
}

.log-row__streams span {
  display: block;
  margin-bottom: 6px;
  color: var(--text-tertiary);
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

pre {
  overflow: auto;
  max-height: 160px;
  margin: 0;
  border: 1px solid var(--divider);
  border-radius: 10px;
  background: var(--console-bg);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  padding: 12px 14px;
  white-space: pre-wrap;
}

@media (max-width: 880px) {
  .log-row__summary {
    grid-template-columns: 20px minmax(0, 1fr) auto;
  }

  .log-row__summary span:nth-last-child(-n + 3) {
    display: none;
  }

  .log-row__details {
    padding-left: 16px;
  }

  dl {
    grid-template-columns: 1fr;
  }
}
</style>
