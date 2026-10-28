<script setup lang="ts">
import Icon from "./Icon.vue";

withDefaults(
  defineProps<{
    icon?: string;
    text: string;
    action?: string;
  }>(),
  {
    icon: "ic_bottom_books_e",
    action: "",
  },
);

defineEmits<{ action: [] }>();
</script>

<template>
  <div class="empty-state">
    <Icon v-if="icon" :name="icon" :size="48" />
    <p>{{ text }}</p>
    <button v-if="action" @click="$emit('action')">{{ action }}</button>
  </div>
</template>

<style scoped>
.empty-state {
  min-height: 280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--ac-secondary-text);
}
p {
  margin: 0;
  font-size: 14px;
}
button {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 8px 18px;
  cursor: pointer;
}
</style>
