<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";

import AppButton from "@/components/ui/AppButton.vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import FormField from "@/components/ui/FormField.vue";
import { useConjureStore } from "@/stores/conjure";

const route = useRoute();
const router = useRouter();
const store = useConjureStore();

const activeTab = computed(() => {
  if (route.name === "logs") {
    return "logs";
  }

  if (route.name === "settings") {
    return null;
  }

  return "tools";
});

function setQuery(query: string): void {
  store.setQuery(query);
  if (route.name !== "tools") {
    router.push({ name: "tools" });
  }
}
</script>

<template>
  <header class="top-bar">
    <div class="top-bar__surface glass">
      <div class="top-bar__inner">
        <RouterLink :to="{ name: 'tools' }" class="top-bar__brand" aria-label="Conjure tools">
          <span class="top-bar__mark">
            <AppIcon name="bolt" :size="15" fill />
          </span>
          <span class="top-bar__wordmark">Conjure</span>
        </RouterLink>

        <nav class="top-bar__nav" aria-label="Primary">
          <RouterLink class="top-bar__nav-item" :class="{ active: activeTab === 'tools' }" :to="{ name: 'tools' }">Tools</RouterLink>
          <RouterLink class="top-bar__nav-item" :class="{ active: activeTab === 'logs' }" :to="{ name: 'logs' }">Logs</RouterLink>
        </nav>

        <div class="top-bar__spacer" />

        <div class="top-bar__search">
          <FormField icon="search" placeholder="Search tools…" :model-value="store.query" @update:model-value="setQuery" />
        </div>

        <AppButton variant="tertiary" size="sm" icon="settings" title="Settings" @click="router.push({ name: 'settings' })" />
        <AppButton variant="primary" size="sm" icon="plus" @click="router.push({ name: 'tool-create' })">
          New tool
        </AppButton>
      </div>
    </div>
  </header>
</template>

<style scoped>
.top-bar {
  position: sticky;
  top: 0;
  z-index: 100;
}

.top-bar__surface {
  border-top: 0;
  border-right: 0;
  border-left: 0;
  border-radius: 0;
  box-shadow: 0 1px 0 var(--divider), 0 8px 24px -16px rgba(0, 0, 0, 0.6);
}

.top-bar__inner {
  display: flex;
  height: 60px;
  max-width: 1280px;
  align-items: center;
  gap: 18px;
  margin: 0 auto;
  padding: 0 24px;
}

.top-bar__brand {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--text-primary);
  text-decoration: none;
}

.top-bar__mark {
  display: grid;
  width: 26px;
  height: 26px;
  flex-shrink: 0;
  place-items: center;
  border-radius: 8px;
  background: var(--button-primary-bg);
  color: var(--button-primary-fg);
}

.top-bar__wordmark {
  font-family: var(--font-display);
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: normal;
}

.top-bar__nav {
  display: flex;
  gap: 2px;
  margin-left: 8px;
}

.top-bar__nav-item {
  position: relative;
  display: inline-flex;
  align-items: center;
  border-radius: 8px;
  color: var(--text-secondary);
  font-family: var(--font-ui);
  font-size: 14px;
  font-weight: 500;
  line-height: normal;
  padding: 8px 14px;
  text-decoration: none;
}

.top-bar__nav-item.active,
.top-bar__nav-item:hover {
  color: var(--text-primary);
}

.top-bar__nav-item.active::after {
  position: absolute;
  right: 14px;
  bottom: -1px;
  left: 14px;
  height: 2px;
  border-radius: 2px;
  background: var(--accent);
  content: "";
}

.top-bar__spacer {
  flex: 1;
}

.top-bar__search {
  width: min(240px, 32vw);
}

@media (max-width: 760px) {
  .top-bar__inner {
    height: auto;
    flex-wrap: wrap;
    gap: 10px;
    padding: 12px 16px;
  }

  .top-bar__spacer {
    display: none;
  }

  .top-bar__search {
    order: 3;
    width: 100%;
  }
}
</style>
