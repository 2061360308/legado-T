<script setup lang="ts">
import Icon from "./Icon.vue";

export interface PopupMenuItem {
  key: string;
  label: string;
  icon?: string;
  danger?: boolean;
  muted?: boolean;
  checked?: boolean;
}

withDefaults(
  defineProps<{
    modelValue: boolean;
    items: PopupMenuItem[];
    top?: number;
    right?: number;
    minWidth?: number;
  }>(),
  {
    top: 52,
    right: 8,
    minWidth: 180,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  select: [key: string];
}>();

function choose(key: string) {
  emit("select", key);
  emit("update:modelValue", false);
}
</script>

<template>
  <div v-if="modelValue" class="popup-mask" @click.self="emit('update:modelValue', false)">
    <div
      class="popup-menu"
      :style="{ top: `${top}px`, right: `${right}px`, minWidth: `${minWidth}px` }"
    >
      <button
        v-for="item in items"
        :key="item.key"
        :class="{ danger: item.danger, muted: item.muted }"
        @click="choose(item.key)"
      >
        <Icon v-if="item.icon" :name="item.icon" :size="20" />
        <span class="popup-label">{{ item.label }}</span>
        <Icon v-if="item.checked" name="ic_check" :size="18" class="popup-check" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.popup-mask {
  position: fixed;
  inset: 0;
  z-index: 55;
}
.popup-menu {
  position: absolute;
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.24);
  padding: 4px 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 16px;
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
}
button:hover {
  background: var(--ac-transparent-10);
}
button.danger {
  color: var(--ac-error);
}
button.muted {
  color: var(--ac-secondary-text);
}
.popup-label {
  flex: 1;
  min-width: 0;
}
.popup-check {
  color: var(--ac-primary);
}
</style>
