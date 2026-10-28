<script setup lang="ts">
import Icon from "./Icon.vue";

withDefaults(
  defineProps<{
    name: string;
    subtitle?: string;
    enabled: boolean;
    selected?: boolean;
    showSelection?: boolean;
  }>(),
  {
    subtitle: "",
    selected: false,
    showSelection: true,
  },
);

const emit = defineEmits<{
  "update:selected": [value: boolean];
  toggle: [];
  edit: [];
  more: [];
}>();
</script>

<template>
  <div class="rule-row">
    <label v-if="showSelection" class="select-box">
      <input
        type="checkbox"
        :checked="selected"
        @change="emit('update:selected', ($event.target as HTMLInputElement).checked)"
      />
    </label>
    <div class="rule-text">
      <p class="rule-name">{{ name }}</p>
      <p v-if="subtitle" class="rule-subtitle">{{ subtitle }}</p>
    </div>
    <label class="switch">
      <input
        type="checkbox"
        :checked="enabled"
        @change="emit('toggle')"
      />
      <span class="slider" />
    </label>
    <button class="icon-btn" aria-label="编辑" @click="emit('edit')">
      <Icon name="ic_edit" :size="22" />
    </button>
    <button class="icon-btn" aria-label="更多" @click="emit('more')">
      <Icon name="ic_more_vert" :size="22" />
    </button>
  </div>
</template>

<style scoped>
.rule-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 56px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.select-box input {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.rule-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.rule-name {
  margin: 0;
  font-size: 15px;
  color: var(--ac-primary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rule-subtitle {
  margin: 0;
  font-size: 12px;
  color: var(--ac-secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.switch {
  position: relative;
  width: 36px;
  height: 20px;
  flex: none;
  cursor: pointer;
}
.switch input {
  position: absolute;
  opacity: 0;
  inset: 0;
  margin: 0;
  cursor: pointer;
}
.slider {
  position: absolute;
  inset: 0;
  border-radius: 10px;
  background: var(--ac-divider);
  transition: background 0.18s ease;
}
.slider::after {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  top: 2px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  transition: transform 0.18s ease;
}
.switch input:checked + .slider {
  background: var(--ac-primary);
}
.switch input:checked + .slider::after {
  transform: translateX(16px);
}
.icon-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 50%;
}
.icon-btn:hover {
  background: var(--ac-transparent-10);
}
</style>
