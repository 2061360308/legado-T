<script setup lang="ts">
withDefaults(
  defineProps<{
    count?: number;
  }>(),
  {
    count: 0,
  },
);
</script>

<template>
  <span v-if="count > 0" class="badge">{{ count }}</span>
</template>

<style scoped>
.badge {
  display: inline-flex;
  min-width: 18px;
  height: 18px;
  align-items: center;
  justify-content: center;
  padding: 0 5px;
  border-radius: 9px;
  background: var(--ac-error);
  color: #fff;
  font-size: 11px;
  line-height: 1;
}
</style>
