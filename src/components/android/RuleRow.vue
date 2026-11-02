<script setup lang="ts">
import Icon from "./Icon.vue";
import ToggleSwitch from "./ToggleSwitch.vue";

withDefaults(
  defineProps<{
    name: string;
    subtitle?: string;
    enabled: boolean;
    selected?: boolean;
    showSelection?: boolean;
    showMore?: boolean;
    showDelete?: boolean;
  }>(),
  {
    subtitle: "",
    selected: false,
    showSelection: true,
    showMore: true,
    showDelete: false,
  },
);

const emit = defineEmits<{
  "update:selected": [value: boolean];
  toggle: [value: boolean];
  edit: [];
  more: [event: MouseEvent];
  delete: [];
}>();
</script>

<template>
  <div class="rule-row">
    <label v-if="showSelection" class="rule-main">
      <input
        type="checkbox"
        :checked="selected"
        @change="emit('update:selected', ($event.target as HTMLInputElement).checked)"
      />
      <span class="rule-text">
        <span class="rule-name">{{ name }}</span>
        <span v-if="subtitle" class="rule-subtitle">{{ subtitle }}</span>
      </span>
    </label>
    <span v-else class="rule-text">
      <span class="rule-name">{{ name }}</span>
      <span v-if="subtitle" class="rule-subtitle">{{ subtitle }}</span>
    </span>
    <ToggleSwitch :checked="enabled" @change="emit('toggle', $event)" />
    <button class="icon-btn" aria-label="编辑" @click="emit('edit')">
      <Icon name="ic_edit" :size="22" />
    </button>
    <button
      v-if="showDelete"
      class="icon-btn"
      aria-label="删除"
      @click="emit('delete')"
    >
      <Icon name="ic_outline_delete" :size="22" />
    </button>
    <button
      v-else-if="showMore"
      class="icon-btn"
      aria-label="更多"
      @click="emit('more', $event)"
    >
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
.rule-main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}
.rule-main input {
  flex: none;
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
  color: var(--ac-primary-text);
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rule-subtitle {
  color: var(--ac-secondary-text);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.icon-btn {
  width: 36px;
  height: 36px;
  flex: none;
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
