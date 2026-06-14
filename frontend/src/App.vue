<script setup lang="ts">
import { onMounted } from "vue";

import AppShell from "@/components/layout/AppShell.vue";

onMounted(() => {
  requestAnimationFrame(() => requestAnimationFrame(() => document.documentElement.classList.add("anim-on")));
});
</script>

<template>
  <div class="atmosphere">
    <div class="glow-3" />
  </div>
  <div class="grain" />
  <AppShell />
</template>
