<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue?: string | null;
    placeholder?: string;
    rows?: number;
    mono?: boolean;
  }>(),
  {
    rows: 4,
  },
);

defineEmits<{
  "update:modelValue": [value: string];
}>();
</script>

<template>
  <textarea
    class="field scroll text-area"
    :class="{ mono }"
    :rows="rows"
    :value="modelValue ?? ''"
    :placeholder="placeholder"
    @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>

<style scoped>
.text-area {
  resize: vertical;
  line-height: 1.55;
}

.mono {
  font-family: var(--font-mono);
}
</style>
