<script setup lang="ts">
withDefaults(defineProps<{ title?: string }>(), { title: "" });
</script>

<template>
  <section class="pref-group">
    <h2 v-if="title">{{ title }}</h2>
    <slot />
  </section>
</template>

<style scoped>
.pref-group {
  margin: 8px 0 12px;
}
h2 {
  margin: 0;
  padding: 10px 20px 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--ac-primary);
}
</style>
