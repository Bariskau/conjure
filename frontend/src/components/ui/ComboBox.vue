<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from "vue";

import AppIcon from "./AppIcon.vue";

const props = withDefaults(
  defineProps<{
    modelValue?: string | null;
    options: string[];
    maxLength?: number;
  }>(),
  {
    maxLength: 32,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  create: [value: string];
}>();

const activeIndex = ref(0);
const open = ref(false);
const position = ref<Record<string, string>>({});
const query = ref("");
const search = ref<HTMLInputElement | null>(null);
const trigger = ref<HTMLElement | null>(null);
const menu = ref<HTMLElement | null>(null);

const filteredOptions = computed(() => {
  const needle = query.value.trim().toLowerCase();
  return props.options.filter((option) => option.toLowerCase().includes(needle));
});

const normalizedQuery = computed(() => query.value.trim().replace(/\s+/g, " ").slice(0, props.maxLength));
const exactMatch = computed(() => props.options.find((option) => option.toLowerCase() === normalizedQuery.value.toLowerCase()));
const canCreate = computed(() => normalizedQuery.value.length > 0 && !exactMatch.value);
const rowCount = computed(() => filteredOptions.value.length + (canCreate.value ? 1 : 0));

async function openMenu(): Promise<void> {
  open.value = true;
  activeIndex.value = Math.max(0, props.options.indexOf(props.modelValue ?? ""));
  await nextTick();
  position.value = menuPosition();
  search.value?.focus();
  window.addEventListener("scroll", closeMenu, true);
  window.addEventListener("resize", closeMenu);
  document.addEventListener("mousedown", closeOnOutsideClick);
}

function closeMenu(): void {
  open.value = false;
  query.value = "";
  window.removeEventListener("scroll", closeMenu, true);
  window.removeEventListener("resize", closeMenu);
  document.removeEventListener("mousedown", closeOnOutsideClick);
}

function choose(value: string): void {
  emit("update:modelValue", value);
  closeMenu();
}

function createOption(): void {
  if (!normalizedQuery.value) {
    return;
  }

  if (exactMatch.value) {
    choose(exactMatch.value);
    return;
  }

  emit("create", normalizedQuery.value);
  choose(normalizedQuery.value);
}

function commitActiveRow(): void {
  if (activeIndex.value < filteredOptions.value.length) {
    choose(filteredOptions.value[activeIndex.value]);
    return;
  }

  if (canCreate.value) {
    createOption();
  }
}

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "ArrowDown") {
    activeIndex.value = Math.min(activeIndex.value + 1, rowCount.value - 1);
    event.preventDefault();
  }

  if (event.key === "ArrowUp") {
    activeIndex.value = Math.max(activeIndex.value - 1, 0);
    event.preventDefault();
  }

  if (event.key === "Enter") {
    commitActiveRow();
    event.preventDefault();
  }

  if (event.key === "Escape") {
    closeMenu();
    event.preventDefault();
  }
}

function closeOnOutsideClick(event: MouseEvent): void {
  const target = event.target as Node;
  if (!trigger.value?.contains(target) && !menu.value?.contains(target)) {
    closeMenu();
  }
}

function menuPosition(): Record<string, string> {
  const rect = trigger.value?.getBoundingClientRect();
  if (!rect) {
    return {};
  }

  return {
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    top: `${rect.bottom + 6}px`,
    maxHeight: `${Math.min(window.innerHeight - rect.bottom - 16, 324)}px`,
  };
}

onBeforeUnmount(closeMenu);
</script>

<template>
  <div class="combo-box">
    <button
      ref="trigger"
      type="button"
      class="field combo-box__trigger"
      :class="{ open }"
      @click="open ? closeMenu() : openMenu()"
      @keydown.enter.prevent="openMenu"
    >
      <span :class="{ placeholder: !modelValue }">{{ modelValue || "Select or create" }}</span>
      <AppIcon name="chevron-down" :size="15" class="combo-box__chevron" />
    </button>

    <teleport to="body">
      <div
        v-if="open"
        ref="menu"
        class="menu-surface anim-pop combo-box__menu"
        :style="position"
      >
        <div class="combo-box__search">
          <AppIcon name="search" :size="15" class="combo-box__search-icon" />
          <input
            ref="search"
            v-model="query"
            class="combo-box__input"
            placeholder="Search or create"
            @input="activeIndex = 0"
            @keydown="handleKeydown"
          />
        </div>

        <div class="scroll combo-box__rows">
          <button
            v-for="(option, index) in filteredOptions"
            :key="option"
            type="button"
            class="combo-box__row"
            :class="{ active: index === activeIndex }"
            @mouseenter="activeIndex = index"
            @click="choose(option)"
          >
            <span>{{ option }}</span>
            <AppIcon v-if="option === modelValue" name="check" :size="14" :stroke-width="2.2" />
          </button>

          <p v-if="filteredOptions.length === 0 && !canCreate" class="combo-box__empty">No matches</p>

          <button
            v-if="canCreate"
            type="button"
            class="combo-box__row combo-box__create"
            :class="{ active: activeIndex === filteredOptions.length }"
            @mouseenter="activeIndex = filteredOptions.length"
            @click="createOption"
          >
            <AppIcon name="plus" :size="14" />
            <span>Create "{{ normalizedQuery }}"</span>
          </button>
        </div>
      </div>
    </teleport>
  </div>
</template>

<style scoped>
.combo-box {
  position: relative;
}

.combo-box__trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  cursor: pointer;
}

.combo-box__trigger.open {
  border-color: var(--glass-border-strong);
}

.combo-box__chevron,
.placeholder {
  color: var(--text-tertiary);
}

.combo-box__menu {
  position: fixed;
  z-index: 4000;
  display: flex;
  overflow: hidden;
  flex-direction: column;
  border-radius: 12px;
}

.combo-box__search {
  position: relative;
  padding: 8px;
  border-bottom: 1px solid var(--divider);
}

.combo-box__search-icon {
  position: absolute;
  top: 50%;
  left: 18px;
  color: var(--text-tertiary);
  transform: translateY(-50%);
}

.combo-box__input {
  width: 100%;
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  outline: none;
  background: var(--glass-surface);
  color: var(--text-primary);
  padding: 8px 10px 8px 32px;
}

.combo-box__rows {
  overflow-y: auto;
  padding: 6px;
}

.combo-box__row {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  border-radius: 8px;
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  padding: 9px 10px;
  text-align: left;
}

.combo-box__row:hover,
.combo-box__row.active {
  background: var(--menu-item-active);
}

.combo-box__create {
  justify-content: flex-start;
  border-top: 1px solid var(--divider);
  margin-top: 4px;
  padding-top: 11px;
}

.combo-box__empty {
  margin: 0;
  padding: 12px 10px;
  color: var(--text-tertiary);
  font-size: 13px;
  text-align: center;
}
</style>
