<script setup lang="ts">
import AppIcon, { type IconName } from "./AppIcon.vue";

withDefaults(
  defineProps<{
    icon?: IconName;
    title: string;
    body: string;
  }>(),
  {
    icon: "zap",
  },
);
</script>

<template>
  <div class="empty-state">
    <div class="empty-state__icon glass">
      <AppIcon :name="icon" :size="26" />
    </div>
    <h2>{{ title }}</h2>
    <p>{{ body }}</p>
    <slot />
  </div>
</template>

<style scoped>
.empty-state {
  display: grid;
  place-items: center;
  padding: 80px 24px;
  text-align: center;
}

.empty-state__icon {
  display: grid;
  width: 64px;
  height: 64px;
  place-items: center;
  margin-bottom: 20px;
  color: var(--text-secondary);
}

h2 {
  margin: 0 0 8px;
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 500;
  letter-spacing: -0.012em;
  line-height: 1.2;
}

p {
  max-width: 360px;
  margin: 0 0 22px;
  color: var(--text-secondary);
}
</style>
