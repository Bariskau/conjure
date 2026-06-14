<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from "vue";

import AppIcon from "./AppIcon.vue";

export interface SelectOption {
  value: string | number | boolean;
  label: string;
}

const props = defineProps<{
  modelValue?: string | number | boolean | null;
  options: Array<SelectOption | string>;
  invalid?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string | number | boolean];
}>();

const open = ref(false);
const trigger = ref<HTMLElement | null>(null);
const menu = ref<HTMLElement | null>(null);
const position = ref<Record<string, string>>({});

const normalizedOptions = computed<SelectOption[]>(() => {
  return props.options.map((option) => (typeof option === "string" ? { value: option, label: option } : option));
});

const currentOption = computed(() => {
  return normalizedOptions.value.find((option) => option.value === props.modelValue);
});

function toggleMenu(): void {
  if (open.value) {
    closeMenu();
    return;
  }

  openMenu();
}

async function openMenu(): Promise<void> {
  open.value = true;
  await nextTick();
  position.value = menuPosition();
  window.addEventListener("scroll", closeMenu, true);
  window.addEventListener("resize", closeMenu);
  document.addEventListener("mousedown", closeOnOutsideClick);
}

function closeMenu(): void {
  open.value = false;
  window.removeEventListener("scroll", closeMenu, true);
  window.removeEventListener("resize", closeMenu);
  document.removeEventListener("mousedown", closeOnOutsideClick);
}

function choose(value: string | number | boolean): void {
  emit("update:modelValue", value);
  closeMenu();
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

  const estimatedHeight = Math.min(normalizedOptions.value.length * 38 + 14, 300);
  const below = window.innerHeight - rect.bottom;
  const opensUp = below < estimatedHeight + 12 && rect.top > below;

  return {
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    top: opensUp ? "auto" : `${rect.bottom + 6}px`,
    bottom: opensUp ? `${window.innerHeight - rect.top + 6}px` : "auto",
    maxHeight: `${Math.min((opensUp ? rect.top : below) - 16, 324)}px`,
  };
}

onBeforeUnmount(closeMenu);
</script>

<template>
  <div class="select-menu">
    <button
      ref="trigger"
      type="button"
      class="field select-menu__trigger"
      :class="{ invalid, open }"
      @click="toggleMenu"
      @keydown.escape.prevent="closeMenu"
    >
      <span :class="{ placeholder: !currentOption }">{{ currentOption?.label ?? "Select" }}</span>
      <AppIcon name="chevron-down" :size="15" class="select-menu__chevron" />
    </button>

    <teleport to="body">
      <div
        v-if="open"
        ref="menu"
        class="menu-surface scroll anim-pop select-menu__options"
        :style="position"
        role="listbox"
      >
        <button
          v-for="option in normalizedOptions"
          :key="String(option.value)"
          type="button"
          class="select-menu__option"
          :class="{ active: option.value === modelValue }"
          @click="choose(option.value)"
        >
          <span>{{ option.label }}</span>
          <AppIcon v-if="option.value === modelValue" name="check" :size="14" :stroke-width="2.2" />
        </button>
      </div>
    </teleport>
  </div>
</template>

<style scoped>
.select-menu {
  position: relative;
}

.select-menu__trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  cursor: pointer;
}

.select-menu__trigger.open {
  border-color: var(--glass-border-strong);
}

.placeholder,
.select-menu__chevron {
  color: var(--text-tertiary);
}

.select-menu__options {
  position: fixed;
  z-index: 4000;
  overflow-y: auto;
  padding: 6px;
  border-radius: 12px;
}

.select-menu__option {
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

.select-menu__option:hover {
  background: var(--menu-item-hover);
}

.select-menu__option.active {
  background: var(--menu-item-active);
}
</style>
