<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";

import CategoryChip from "@/components/tool/CategoryChip.vue";
import ToolCard from "@/components/tool/ToolCard.vue";
import AppButton from "@/components/ui/AppButton.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import type { ToolSummary } from "@/domain/types";
import { useConjureStore } from "@/stores/conjure";

const router = useRouter();
const store = useConjureStore();
const activeCategory = ref("All");

const categoryCounts = computed(() => {
  return store.tools.reduce<Record<string, number>>((counts, tool) => {
    const category = tool.category ?? "Uncategorized";
    counts[category] = (counts[category] ?? 0) + 1;
    return counts;
  }, {});
});

const categories = computed(() => ["All", ...store.sortedCategories]);

const filteredTools = computed(() => {
  return store.filteredTools.filter((tool) => {
    return activeCategory.value === "All" || tool.category === activeCategory.value;
  });
});

function editTool(tool: ToolSummary): void {
  router.push({ name: "tool-edit", params: { toolId: tool.id } });
}

function testTool(tool: ToolSummary): void {
  router.push({ name: "test", params: { toolId: tool.id } });
}
</script>

<template>
  <section class="page page-enter">
    <div class="page__header">
      <div>
        <h1>Tools</h1>
        <p>Shell scripts, conjured into MCP tools your agents can call.</p>
      </div>
      <AppButton variant="primary" icon="plus" @click="router.push({ name: 'tool-create' })">New tool</AppButton>
    </div>

    <div class="scroll tools-page__chips">
      <CategoryChip
        v-for="category in categories"
        :key="category"
        :active="activeCategory === category"
        :count="category === 'All' ? store.tools.length : categoryCounts[category] ?? 0"
        @click="activeCategory = category"
      >
        {{ category }}
      </CategoryChip>
    </div>

    <EmptyState
      v-if="filteredTools.length === 0"
      icon="search"
      title="No matching tools"
      body="Nothing matches the current filter. Try a different search."
    >
      <AppButton variant="secondary" @click="activeCategory = 'All'; store.setQuery('')">Clear filter</AppButton>
    </EmptyState>

    <div v-else class="tools-page__grid">
      <ToolCard
        v-for="(tool, index) in filteredTools"
        :key="tool.id"
        class="anim-rise"
        :style="{ animationDelay: `${Math.min(index * 45, 400)}ms` }"
        :tool="tool"
        @edit="editTool"
        @test="testTool"
        @toggle="store.toggleTool"
      />
    </div>
  </section>
</template>

<style scoped>
.page {
  max-width: 1280px;
  margin: 0 auto;
  padding: 32px 24px 80px;
}

.page__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 22px;
}

h1 {
  margin: 0;
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 500;
  letter-spacing: -0.015em;
  line-height: 1.12;
}

p {
  margin: 8px 0 0;
  color: var(--text-secondary);
}

.tools-page__chips {
  display: flex;
  overflow-x: auto;
  gap: 8px;
  margin-bottom: 22px;
  padding-bottom: 4px;
}

.tools-page__grid {
  display: grid;
  align-items: stretch;
  grid-template-columns: repeat(auto-fill, minmax(var(--tools-grid-min), 1fr));
  gap: 16px;
}

@media (max-width: 760px) {
  .page {
    padding: 24px 16px 72px;
  }

  .page__header {
    align-items: flex-start;
    flex-direction: column;
  }

  .tools-page__grid {
    grid-template-columns: 1fr;
  }
}
</style>
