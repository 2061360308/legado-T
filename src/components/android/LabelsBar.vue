<script setup lang="ts">
withDefaults(
  defineProps<{
    labels?: string[];
  }>(),
  {
    labels: () => [],
  },
);
</script>

<template>
  <div v-if="labels.length" class="labels">
    <span v-for="label in labels" :key="label" class="label">{{ label }}</span>
  </div>
</template>

<style scoped>
.labels {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  min-width: 0;
  overflow: hidden;
  max-height: 20px;
}
.label {
  flex: none;
  max-width: 100%;
  padding: 1px 6px;
  border-radius: 2px;
  background: var(--ac-transparent-10);
  color: var(--ac-secondary-text);
  font-size: 11px;
  line-height: 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
