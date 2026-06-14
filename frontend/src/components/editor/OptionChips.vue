<script setup lang="ts">
import { ref } from "vue";

import AppButton from "@/components/ui/AppButton.vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import FormField from "@/components/ui/FormField.vue";

const props = defineProps<{
  options: string[];
}>();

const emit = defineEmits<{
  "update:options": [options: string[]];
}>();

const nextOption = ref("");

function addOption(): void {
  const value = nextOption.value.trim();
  if (!value) {
    return;
  }

  if (!props.options.some((option) => option.toLowerCase() === value.toLowerCase())) {
    emit("update:options", [...props.options, value]);
  }

  nextOption.value = "";
}

function removeOption(index: number): void {
  emit(
    "update:options",
    props.options.filter((_, optionIndex) => optionIndex !== index),
  );
}
</script>

<template>
  <div class="option-chips">
    <div class="option-chips__header">
      <label>Options</label>
      <span>Enter to add</span>
    </div>

    <div v-if="options.length" class="option-chips__items">
      <span v-for="(option, index) in options" :key="option" class="option-chips__chip">
        {{ option }}
        <button type="button" :aria-label="`Remove ${option}`" @click="removeOption(index)">
          <AppIcon name="x" :size="9" :stroke-width="2.6" />
        </button>
      </span>
    </div>

    <div class="option-chips__input">
      <FormField
        v-model="nextOption"
        mono
        placeholder="add option"
        @keydown.enter.prevent="addOption"
      />
      <AppButton variant="secondary" size="sm" icon="plus" title="Add option" @click="addOption" />
    </div>
  </div>
</template>

<style scoped>
.option-chips__header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 7px;
}

label {
  font-size: 13px;
  font-weight: 500;
}

.option-chips__header span {
  color: var(--text-tertiary);
  font-size: 12px;
}

.option-chips__items {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.option-chips__chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--glass-border);
  border-radius: 999px;
  background: var(--glass-surface);
  font-family: var(--font-mono);
  font-size: 12.5px;
  padding: 4px 5px 4px 11px;
}

.option-chips__chip button {
  display: grid;
  width: 16px;
  height: 16px;
  place-items: center;
  border-radius: 999px;
  background: var(--hover-wash);
  color: var(--text-secondary);
  cursor: pointer;
}

.option-chips__input {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
}
</style>
