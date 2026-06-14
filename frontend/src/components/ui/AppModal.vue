<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";

import AppButton from "@/components/ui/AppButton.vue";

withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    width?: number;
  }>(),
  {
    width: 560,
  },
);

const emit = defineEmits<{
  close: [];
}>();

function closeOnEscape(event: KeyboardEvent): void {
  if (event.key === "Escape") {
    emit("close");
  }
}

onMounted(() => document.addEventListener("keydown", closeOnEscape));
onBeforeUnmount(() => document.removeEventListener("keydown", closeOnEscape));
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="app-modal" @click="$emit('close')">
      <div class="menu-surface anim-up app-modal__panel" :style="{ maxWidth: `${width}px` }" @click.stop>
        <div class="app-modal__header">
          <div class="t-h1">{{ title }}</div>
          <AppButton variant="tertiary" icon="close" size="sm" @click="$emit('close')" />
        </div>
        <hr class="hairline" />
        <div class="app-modal__body">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.app-modal {
  position: fixed;
  inset: 0;
  z-index: 500;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.7);
  padding: 24px;
}

.app-modal__panel {
  width: 100%;
  border-radius: 18px;
}

.app-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 18px 20px;
}

.app-modal__body {
  padding: 20px;
}
</style>
