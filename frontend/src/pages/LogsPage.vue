<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import LogRow from "@/components/log/LogRow.vue";
import AppButton from "@/components/ui/AppButton.vue";
import EmptyState from "@/components/ui/EmptyState.vue";
import FormField from "@/components/ui/FormField.vue";
import GlassPanel from "@/components/ui/GlassPanel.vue";
import SelectMenu from "@/components/ui/SelectMenu.vue";
import StatusDot from "@/components/ui/StatusDot.vue";
import type { ExecutionStatus, LogFilter } from "@/domain/types";
import { useConjureStore } from "@/stores/conjure";

const store = useConjureStore();

const openLogId = ref<string | null>(null);
const selectedToolId = ref("all");
const selectedStatus = ref("all");
const selectedRange = ref("7d");
const search = ref("");
const currentPage = ref(1);

const PAGE_SIZE = 12;

const statusOptions = [
  { value: "all", label: "All statuses" },
  { value: "success", label: "Success" },
  { value: "error", label: "Failed" },
  { value: "timeout", label: "Timed out" },
];

const rangeOptions = ["24h", "7d", "30d"];

const toolOptions = computed(() => {
  return [{ value: "all", label: "All" }, ...store.tools.map((tool) => ({ value: tool.id, label: tool.name }))];
});

const summary = computed(() => {
  const counts = { total: store.logs.length, success: 0, failed: 0, running: 0 };
  for (const log of store.logs) {
    if (log.status === "success") {
      counts.success += 1;
    } else if (log.status === "error" || log.status === "timeout") {
      counts.failed += 1;
    } else {
      counts.running += 1;
    }
  }
  return counts;
});

const pageCount = computed(() => Math.max(1, Math.ceil(store.logs.length / PAGE_SIZE)));
const safePage = computed(() => Math.min(currentPage.value, pageCount.value));
const pageStart = computed(() => (store.logs.length === 0 ? 0 : (safePage.value - 1) * PAGE_SIZE + 1));
const pageEnd = computed(() => Math.min(safePage.value * PAGE_SIZE, store.logs.length));
const pageLogs = computed(() => store.logs.slice(pageStart.value - 1, pageEnd.value));

onMounted(refreshLogs);
watch([selectedToolId, selectedStatus, selectedRange, search], refreshLogs);
watch(pageCount, keepPageInBounds);

function refreshLogs(): void {
  currentPage.value = 1;
  openLogId.value = null;
  store.refreshLogs(currentFilter());
}

function currentFilter(): LogFilter {
  return {
    tool_id: selectedToolId.value === "all" ? undefined : selectedToolId.value,
    status: selectedStatus.value === "all" ? undefined : (selectedStatus.value as ExecutionStatus),
    search: search.value.trim() || undefined,
    from: fromDateForRange(selectedRange.value),
  };
}

function fromDateForRange(range: string): string {
  const hours = range === "24h" ? 24 : range === "30d" ? 24 * 30 : 24 * 7;
  return new Date(Date.now() - hours * 60 * 60 * 1000).toISOString();
}

function searchLogs(): void {
  refreshLogs();
}

function resetFilters(): void {
  selectedToolId.value = "all";
  selectedStatus.value = "all";
  selectedRange.value = "7d";
  search.value = "";
  refreshLogs();
}

function keepPageInBounds(): void {
  currentPage.value = safePage.value;
}
</script>

<template>
  <section class="logs-page page-enter">
    <div class="logs-page__header">
      <h1>Logs</h1>
      <p>Every call, with parameters, streams, and exit codes.</p>
    </div>

    <div class="logs-page__summary">
      <GlassPanel class="logs-page__stat plain">
        <div>
          <strong>{{ summary.total }}</strong>
          <span>Total</span>
        </div>
      </GlassPanel>
      <GlassPanel class="logs-page__stat">
        <StatusDot status="success" />
        <div>
          <strong>{{ summary.success }}</strong>
          <span>Succeeded</span>
        </div>
      </GlassPanel>
      <GlassPanel class="logs-page__stat">
        <StatusDot status="error" />
        <div>
          <strong>{{ summary.failed }}</strong>
          <span>Failed</span>
        </div>
      </GlassPanel>
      <GlassPanel class="logs-page__stat">
        <StatusDot status="running" />
        <div>
          <strong>{{ summary.running }}</strong>
          <span>Running</span>
        </div>
      </GlassPanel>
    </div>

    <GlassPanel class="logs-page__filters">
      <div class="logs-page__tool-filter">
        <SelectMenu v-model="selectedToolId" :options="toolOptions" />
      </div>
      <div class="logs-page__status-filter">
        <SelectMenu v-model="selectedStatus" :options="statusOptions" />
      </div>
      <div class="logs-page__segmented" role="radiogroup" aria-label="Log range">
        <button
          v-for="range in rangeOptions"
          :key="range"
          type="button"
          :class="{ active: selectedRange === range }"
          @click="selectedRange = range"
        >
          {{ range }}
        </button>
      </div>
      <div class="logs-page__search">
        <FormField v-model="search" icon="search" placeholder="Search runs…" @keydown.enter="searchLogs" />
      </div>
    </GlassPanel>

    <div class="logs-page__head">
      <span />
      <span>Tool</span>
      <span>Status</span>
      <span>Duration</span>
      <span>Exit</span>
      <span>When</span>
    </div>

    <EmptyState
      v-if="store.logs.length === 0"
      icon="logs"
      title="No runs match"
      body="Loosen the filters or widen the time range to see more history."
    >
      <button type="button" class="logs-page__reset" @click="resetFilters">Reset filters</button>
    </EmptyState>

    <template v-else>
      <div class="logs-page__rows">
        <LogRow
          v-for="(log, index) in pageLogs"
          :key="log.id"
          class="anim-rise"
          :style="{ animationDelay: `${Math.min(index * 28, 320)}ms` }"
          :log="log"
          :expanded="openLogId === log.id"
          @toggle="openLogId = openLogId === log.id ? null : log.id"
        />
      </div>

      <div class="logs-page__pagination">
        <span>{{ pageStart }}–{{ pageEnd }} of {{ store.logs.length }}</span>
        <div>
          <AppButton
            variant="secondary"
            size="sm"
            icon="chevron-right"
            class="logs-page__pager-prev"
            title="Previous"
            :disabled="safePage <= 1"
            @click="currentPage = safePage - 1"
          />
          <button
            v-for="pageNumber in pageCount"
            :key="pageNumber"
            type="button"
            class="logs-page__page-button"
            :class="{ active: pageNumber === safePage }"
            @click="currentPage = pageNumber"
          >
            {{ pageNumber }}
          </button>
          <AppButton
            variant="secondary"
            size="sm"
            icon="chevron-right"
            title="Next"
            :disabled="safePage >= pageCount"
            @click="currentPage = safePage + 1"
          />
        </div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.logs-page {
  max-width: 1280px;
  margin: 0 auto;
  padding: 32px 24px 80px;
}

.logs-page__header {
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

.logs-page__header p {
  margin: 8px 0 0;
  color: var(--text-secondary);
}

.logs-page__summary {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
  margin-bottom: 18px;
}

.logs-page__stat {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 76px;
  border-radius: 14px;
  padding: 14px 16px;
}

.logs-page__stat.plain {
  align-items: center;
}

.logs-page__stat strong {
  display: block;
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 500;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.logs-page__stat span {
  display: block;
  margin-top: 5px;
  color: var(--text-tertiary);
  font-size: 12px;
}

.logs-page__filters {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  border-radius: 14px;
  margin-bottom: 16px;
  padding: 12px;
}

.logs-page__tool-filter {
  width: 200px;
}

.logs-page__status-filter {
  width: 150px;
}

.logs-page__search {
  flex: 1;
  min-width: 180px;
}

.logs-page__segmented {
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--glass-border);
  border-radius: var(--r-control);
  background: var(--glass-surface);
  padding: 3px;
}

.logs-page__segmented button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 38px;
  border-radius: 7px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-family: var(--font-ui);
  font-size: 13px;
  font-weight: 500;
  line-height: normal;
  padding: 5px 10px;
  transition: background .16s var(--ease), color .16s var(--ease);
}

.logs-page__segmented button.active {
  background: var(--btn-primary-bg);
  color: var(--btn-primary-fg);
}

.logs-page__head {
  display: grid;
  grid-template-columns: 20px 1.4fr 1fr 0.8fr 0.8fr 0.6fr;
  gap: 14px;
  color: var(--text-tertiary);
  font-size: 11px;
  letter-spacing: 0.06em;
  padding: 0 16px 8px;
  text-transform: uppercase;
}

.logs-page__rows {
  display: grid;
  gap: 8px;
}

.logs-page__pagination {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: 18px;
}

.logs-page__pagination > span {
  color: var(--text-tertiary);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.logs-page__pagination > div {
  display: flex;
  align-items: center;
  gap: 8px;
}

.logs-page__pager-prev {
  transform: scaleX(-1);
}

.logs-page__page-button {
  min-width: 32px;
  height: 32px;
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  background: var(--glass-surface);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  padding: 0 8px;
}

.logs-page__page-button.active {
  border-color: transparent;
  background: var(--btn-primary-bg);
  color: var(--btn-primary-fg);
}

.logs-page__reset {
  border: 1px solid var(--glass-border);
  border-radius: var(--r-control);
  background: var(--glass-surface);
  color: var(--text-primary);
  cursor: pointer;
  font-weight: 500;
  padding: 9px 16px;
}

@media (max-width: 920px) {
  .logs-page__summary {
    grid-template-columns: repeat(2, 1fr);
  }

  .logs-page__tool-filter,
  .logs-page__status-filter,
  .logs-page__search {
    width: 100%;
    flex: 1 1 100%;
  }

  .logs-page__head {
    display: none;
  }
}

@media (max-width: 760px) {
  .logs-page {
    padding: 24px 16px 72px;
  }
}
</style>
