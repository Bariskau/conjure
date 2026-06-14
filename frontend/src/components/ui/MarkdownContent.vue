<script setup lang="ts">
import { computed } from "vue";

import { renderMarkdown } from "@/domain/markdown";

const props = defineProps<{
  content: string | null | undefined;
}>();

const html = computed(() => renderMarkdown(props.content));
</script>

<template>
  <div class="md" v-html="html" />
</template>
