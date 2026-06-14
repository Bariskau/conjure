<script setup lang="ts">
defineProps<{
  active: boolean;
  count?: number;
}>();

defineEmits<{
  click: [];
}>();
</script>

<template>
  <button type="button" class="category-chip" :class="{ active }" @click="$emit('click')">
    <slot />
    <span v-if="count != null">{{ count }}</span>
  </button>
</template>

<style scoped>
.category-chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  border: 1px solid var(--glass-border);
  border-radius: 999px;
  background: rgba(255,255,255,0.5);
  color: var(--text-secondary);
  cursor: pointer;
  font-family: var(--font-ui);
  font-size: 12.5px;
  font-weight: 500;
  line-height: normal;
  padding: 6px 13px;
  white-space: nowrap;
  transition: background 0.16s var(--ease), color 0.16s var(--ease), border-color 0.16s var(--ease);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.category-chip.active {
  border-color: transparent;
  background: var(--accent);
  color: var(--accent-fg);
  font-weight: 600;
}

.category-chip span {
  opacity: 0.6;
  font-variant-numeric: tabular-nums;
}
</style>
