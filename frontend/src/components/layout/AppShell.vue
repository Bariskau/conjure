<script setup lang="ts">
import { onMounted } from "vue";

import { useConjureStore } from "@/stores/conjure";

import TopBar from "./TopBar.vue";

const store = useConjureStore();

onMounted(() => {
  store.bootstrap();
});
</script>

<template>
  <div class="app-shell">
    <TopBar />
    <main class="scroll app-shell__main">
      <RouterView />
    </main>

    <div class="app-shell__toasts" aria-live="polite">
      <div v-for="toast in store.toasts" :key="toast.id" class="menu-surface app-shell__toast">
        <span class="app-shell__toast-dot" :class="toast.tone" />
        <span>{{ toast.message }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  position: relative;
  z-index: 2;
  min-height: 100vh;
}

.app-shell__main {
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.app-shell__toasts {
  position: fixed;
  bottom: 22px;
  left: 50%;
  z-index: 300;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  pointer-events: none;
  transform: translateX(-50%);
}

.app-shell__toast {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  border-radius: 12px;
  color: var(--text-primary);
  font-size: 13.5px;
  padding: 11px 16px;
}

.app-shell__toast-dot {
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: var(--text-tertiary);
}

.app-shell__toast-dot.success {
  background: var(--text-primary);
}

.app-shell__toast-dot.error {
  border: 1px solid rgba(var(--error-tint), 1);
  background: transparent;
  box-shadow: 0 0 0 2px var(--error-ring);
}

@media (max-width: 760px) {
  .app-shell__main {
    height: calc(100vh - 118px);
  }
}
</style>
