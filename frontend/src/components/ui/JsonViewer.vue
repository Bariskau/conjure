<script setup lang="ts">
import { computed, ref } from "vue";

import AppIcon from "./AppIcon.vue";

const props = withDefaults(
  defineProps<{
    data: unknown;
    title?: string;
    note?: string;
    maxHeight?: number;
    collapsible?: boolean;
  }>(),
  {
    title: "MCP inputSchema",
    note: "generated from descriptors",
    maxHeight: 440,
    collapsible: true,
  },
);

const copied = ref(false);
const open = ref(true);

const jsonText = computed(() => JSON.stringify(props.data, null, 2));
const highlightedLines = computed(() => jsonText.value.split("\n").map(highlightJson));

async function copyJson(): Promise<void> {
  await navigator.clipboard.writeText(jsonText.value);
  copied.value = true;
  window.setTimeout(() => {
    copied.value = false;
  }, 1400);
}

function toggle(): void {
  if (props.collapsible) {
    open.value = !open.value;
  }
}

function highlightJson(value: string): string {
  const escaped = escapeHtml(value);
  return escaped.replace(jsonTokenPattern(), tokenToHtml);
}

function jsonTokenPattern(): RegExp {
  return /("(\\u[\dA-Fa-f]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+-]?\d+)?)/g;
}

function tokenToHtml(match: string): string {
  const className = jsonTokenClass(match);
  return `<span class="${className}">${match}</span>`;
}

function jsonTokenClass(match: string): string {
  if (match.endsWith(":")) {
    return "jv-key";
  }

  if (match.startsWith('"')) {
    return "jv-str";
  }

  if (match === "true" || match === "false" || match === "null") {
    return "jv-bool";
  }

  return "jv-num";
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
</script>

<template>
  <div class="json-viewer glass">
    <div class="json-viewer__header">
      <button type="button" class="json-viewer__title" @click="toggle">
        <AppIcon
          v-if="collapsible"
          name="chevron-down"
          :size="14"
          class="json-viewer__chevron"
          :class="{ closed: !open }"
        />
        <span>
          <strong>{{ title }}</strong>
          <small>{{ note }}</small>
        </span>
      </button>
      <button type="button" class="json-viewer__copy" title="Copy schema" @click.stop="copyJson">
        <AppIcon :name="copied ? 'check' : 'copy'" :size="13" />
        {{ copied ? "Copied" : "Copy" }}
      </button>
    </div>

    <div v-if="open" class="jv scroll json-viewer__body" :style="{ maxHeight: `${maxHeight}px` }">
      <div v-for="(line, index) in highlightedLines" :key="index" class="jv-line">
        <span class="json-viewer__line" v-html="line || '&nbsp;'" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.json-viewer {
  overflow: hidden;
  background: var(--console-bg);
}

.json-viewer__header {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  border-bottom: 1px solid var(--divider);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  padding: 10px 10px 10px 14px;
  text-align: left;
}

.json-viewer__title {
  display: flex;
  flex: 1;
  min-width: 0;
  align-items: center;
  gap: 10px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  text-align: left;
}

.json-viewer__title strong {
  display: block;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 500;
}

.json-viewer__title small {
  display: block;
  color: var(--text-tertiary);
  font-size: 10.5px;
}

.json-viewer__chevron {
  flex-shrink: 0;
  color: var(--text-tertiary);
  transition: transform 0.2s var(--ease);
}

.json-viewer__chevron.closed {
  transform: rotate(-90deg);
}

.json-viewer__copy {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  background: var(--glass-surface);
  color: var(--text-secondary);
  cursor: pointer;
  flex-shrink: 0;
  font-size: 12px;
  padding: 6px 10px;
}

.json-viewer__body {
  overflow: auto;
  padding: 10px 0;
  color: #d6d6dc;
}

.json-viewer__line {
  white-space: pre;
}

</style>
