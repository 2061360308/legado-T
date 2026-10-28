<script setup lang="ts">
defineProps<{ title?: string }>();
</script>

<template>
  <header class="title-bar">
    <span class="title">{{ title ?? "本地阅读器" }}</span>
    <slot name="actions" />
  </header>
</template>

<style scoped>
.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 12px;
  background: var(--lr-color-surface);
  color: var(--lr-color-on-surface);
}
</style>