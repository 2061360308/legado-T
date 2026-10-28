<script setup lang="ts">
import Icon from "./Icon.vue";

withDefaults(
  defineProps<{
    label: string;
    summary?: string;
    icon?: string;
    chevron?: boolean;
    tag?: string;
    disabled?: boolean;
  }>(),
  {
    summary: "",
    icon: "",
    chevron: false,
    tag: "",
    disabled: false,
  },
);

const emit = defineEmits<{ click: [] }>();
</script>

<template>
  <button class="pref-row" :disabled="disabled" @click="emit('click')">
    <Icon v-if="icon" :name="icon" :size="24" class="pref-icon" />
    <span class="pref-text">
      <span class="pref-label">
        {{ label }}
        <span v-if="tag" class="tag">{{ tag }}</span>
      </span>
      <span v-if="summary" class="pref-summary">{{ summary }}</span>
    </span>
    <slot name="right" />
    <Icon v-if="chevron" name="ic_expand_more" :size="20" class="chevron" />
  </button>
</template>

<style scoped>
.pref-row {
  width: 100%;
  min-height: 52px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 20px;
  text-align: left;
  cursor: pointer;
}
.pref-row:hover {
  background: var(--ac-transparent-10);
}
.pref-row:disabled {
  cursor: default;
  opacity: 0.65;
}
.pref-icon {
  color: var(--ac-secondary-text);
}
.pref-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.pref-label {
  font-size: 15px;
  display: flex;
  align-items: center;
  gap: 6px;
}
.pref-summary {
  font-size: 12px;
  color: var(--ac-secondary-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tag {
  font-size: 11px;
  color: var(--ac-secondary-text);
  background: var(--ac-background-card);
  border-radius: 2px;
  padding: 1px 5px;
}
.chevron {
  opacity: 0.5;
}
</style>
