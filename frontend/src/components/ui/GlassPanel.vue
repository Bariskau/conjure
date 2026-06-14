<script setup lang="ts">
withDefaults(
  defineProps<{
    strong?: boolean;
    interactive?: boolean;
    inactive?: boolean;
  }>(),
  {
    strong: false,
    interactive: false,
    inactive: false,
  },
);
</script>

<template>
  <div class="glass-panel glass" :class="{ 'glass-strong': strong, interactive, inactive }">
    <slot />
  </div>
</template>

<style scoped>
.glass-panel {
  transition: transform 0.26s var(--ease-out), border-color 0.26s var(--ease), box-shadow 0.26s var(--ease);
}

.glass-panel.interactive:hover {
  border-color: var(--glass-border-strong);
  box-shadow: var(--shadow-pop), 0 0 0 1px var(--glass-highlight);
  transform: translateY(-3px);
}

.glass-panel.inactive {
  opacity: 0.62;
}
</style>
