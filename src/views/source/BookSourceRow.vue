<script setup lang="ts">
import { computed } from "vue";
import type { BookSource } from "@/core/domain";
import Icon from "@/components/android/Icon.vue";
import ToggleSwitch from "@/components/android/ToggleSwitch.vue";

const props = withDefaults(
  defineProps<{
    source: BookSource;
    selected: boolean;
    host?: string;
    showHost?: boolean;
    checkMessage?: string;
    checking?: boolean;
  }>(),
  {
    host: "",
    showHost: false,
    checkMessage: "",
    checking: false,
  },
);

const emit = defineEmits<{
  select: [value: boolean];
  toggle: [value: boolean];
  edit: [];
  more: [event: MouseEvent];
}>();

const displayName = computed(() => {
  const name = props.source.bookSourceName || props.source.bookSourceUrl;
  const group = String(props.source.bookSourceGroup || "").trim();
  return group ? `${name} (${group})` : name;
});

const hasExplore = computed(() =>
  Boolean(String(props.source.exploreUrl || "").trim()),
);

const exploreEnabled = computed(
  () => hasExplore.value && props.source.enabledExplore !== false,
);
</script>

<template>
  <div class="source-row">
    <div v-if="showHost" class="host">{{ host }}</div>
    <div class="main-row">
      <label class="source-main">
        <input
          type="checkbox"
          :checked="selected"
          @change="emit('select', ($event.target as HTMLInputElement).checked)"
        />
        <span class="source-name">{{ displayName }}</span>
      </label>
      <ToggleSwitch :checked="source.enabled" @change="emit('toggle', $event)" />
      <button class="icon-btn" aria-label="编辑" @click="emit('edit')">
        <Icon name="ic_edit" :size="22" />
      </button>
      <span class="more-wrap">
        <button class="icon-btn" aria-label="更多" @click="emit('more', $event)">
          <Icon name="ic_more_vert" :size="22" />
        </button>
        <span
          v-if="hasExplore"
          class="explore-dot"
          :class="{ off: !exploreEnabled }"
        />
      </span>
    </div>
    <div v-if="checking || checkMessage" class="debug-row">
      <span class="debug-text">{{ checkMessage }}</span>
      <span v-if="checking" class="spinner" />
    </div>
  </div>
</template>

<style scoped>
.source-row {
  padding: 16px;
  border-bottom: 1px solid var(--ac-divider);
  background: var(--ac-background);
}
.host {
  margin: -8px 0 8px;
  color: var(--ac-primary);
  font-size: 16px;
  line-height: 1.2;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.main-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.source-main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}
.source-main input {
  flex: none;
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.source-name {
  min-width: 0;
  color: var(--ac-primary-text);
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.icon-btn {
  width: 36px;
  height: 36px;
  flex: none;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--ac-primary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.icon-btn:hover {
  background: var(--ac-transparent-10);
}
.more-wrap {
  position: relative;
  display: inline-flex;
}
.explore-dot {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #4caf50;
  pointer-events: none;
}
.explore-dot.off {
  background: #f44336;
}
.debug-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
}
.debug-text {
  flex: 1;
  min-width: 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.spinner {
  width: 18px;
  height: 18px;
  flex: none;
  border: 2px solid var(--ac-divider);
  border-top-color: var(--ac-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
