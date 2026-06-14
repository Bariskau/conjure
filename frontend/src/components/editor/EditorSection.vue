<script setup lang="ts">
defineProps<{
  number: string;
  title: string;
  description?: string;
}>();
</script>

<template>
  <section class="editor-section">
    <div class="editor-section__intro">
      <div class="editor-section__heading">
        <span>{{ number }}</span>
        <h2>{{ title }}</h2>
      </div>
      <p v-if="description">{{ description }}</p>
    </div>

    <div>
      <slot />
    </div>
  </section>
</template>

<style scoped>
.editor-section {
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  gap: 32px;
  padding: 28px 0;
}

.editor-section__heading {
  display: flex;
  align-items: center;
  gap: 10px;
}

.editor-section__heading span {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  border: 1px solid var(--glass-border);
  border-radius: 7px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
}

h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.006em;
}

p {
  margin: 10px 0 0 34px;
  color: var(--text-secondary);
  font-size: 12px;
}

@media (max-width: 900px) {
  .editor-section {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  p {
    margin-left: 0;
  }
}
</style>
