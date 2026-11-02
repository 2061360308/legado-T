<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { BookSource } from "@/core/domain";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import Icon from "@/components/android/Icon.vue";

export type SearchScopeType = "all" | "groups" | "source";

export interface SearchScopeValue {
  type: SearchScopeType;
  groups: string[];
  sourceUrl: string;
}

const props = defineProps<{
  modelValue: boolean;
  sources: BookSource[];
  groups: string[];
  scope: SearchScopeValue;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  select: [value: SearchScopeValue];
}>();

const tab = ref<"groups" | "source">("groups");
const selectedGroups = ref<string[]>([]);
const selectedSource = ref("");
const filter = ref("");

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    tab.value = props.scope.type === "source" ? "source" : "groups";
    selectedGroups.value = [...props.scope.groups];
    selectedSource.value = props.scope.type === "source" ? props.scope.sourceUrl : "";
    filter.value = "";
  },
);

const filteredSources = computed(() => {
  const key = filter.value.trim().toLowerCase();
  if (!key) return props.sources;
  return props.sources.filter((source) => {
    const name = String(source.bookSourceName || "").toLowerCase();
    const group = String(source.bookSourceGroup || "").toLowerCase();
    return name.includes(key) || group.includes(key);
  });
});

function toggleGroup(group: string, checked: boolean) {
  const values = new Set(selectedGroups.value);
  if (checked) values.add(group);
  else values.delete(group);
  selectedGroups.value = [...values];
}

function chooseAll() {
  emit("select", { type: "all", groups: [], sourceUrl: "" });
  emit("update:modelValue", false);
}

function confirm() {
  if (tab.value === "source" && selectedSource.value) {
    emit("select", {
      type: "source",
      groups: [],
      sourceUrl: selectedSource.value,
    });
  } else if (tab.value === "groups" && selectedGroups.value.length) {
    emit("select", {
      type: "groups",
      groups: [...selectedGroups.value],
      sourceUrl: "",
    });
  } else {
    emit("select", { type: "all", groups: [], sourceUrl: "" });
  }
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="搜索范围"
    :body-padding="false"
    width="90vw"
    height="80vh"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="scope-body">
      <div class="tabs">
        <button :class="{ active: tab === 'groups' }" @click="tab = 'groups'">分组</button>
        <button :class="{ active: tab === 'source' }" @click="tab = 'source'">书源</button>
      </div>

      <label v-if="tab === 'source'" class="filter">
        <Icon name="ic_search_hint" :size="18" />
        <input v-model="filter" placeholder="筛选书源" />
      </label>

      <div class="list">
        <template v-if="tab === 'groups'">
          <label v-for="group in groups" :key="group" class="row">
            <input
              type="checkbox"
              :checked="selectedGroups.includes(group)"
              @change="toggleGroup(group, ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ group }}</span>
          </label>
          <p v-if="!groups.length" class="empty">暂无分组</p>
        </template>

        <template v-else>
          <label v-for="source in filteredSources" :key="source.bookSourceUrl" class="row">
            <input
              v-model="selectedSource"
              type="radio"
              name="search-scope-source"
              :value="source.bookSourceUrl"
            />
            <span class="source-name">{{ source.bookSourceName || source.bookSourceUrl }}</span>
            <span v-if="source.bookSourceGroup" class="source-group">
              {{ source.bookSourceGroup }}
            </span>
          </label>
          <p v-if="!filteredSources.length" class="empty">没有匹配的书源</p>
        </template>
      </div>
    </div>

    <template #footer>
      <div class="bottom-bar">
        <button class="text-button" @click="chooseAll">全部书源</button>
        <span class="spacer" />
        <button class="text-button muted" @click="emit('update:modelValue', false)">
          取消
        </button>
        <button class="text-button" @click="confirm">确定</button>
      </div>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.scope-body {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.tabs {
  flex: none;
  display: flex;
  gap: 4px;
  padding: 4px 11px;
}
.tabs button {
  flex: 1;
  border: none;
  border-radius: 2px;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 8px 5px;
  font-size: 14px;
  cursor: pointer;
}
.tabs button.active {
  color: var(--ac-primary);
  background: var(--ac-transparent-10);
}
.filter {
  flex: none;
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 6px 16px;
  padding: 0 10px;
  height: 34px;
  border-radius: 17px;
  background: var(--ac-transparent-10);
  color: var(--ac-secondary-text);
}
.filter input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 13px;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  color: var(--ac-primary-text);
  font-size: 14px;
  cursor: pointer;
}
.row:hover {
  background: var(--ac-transparent-10);
}
.row input {
  flex: none;
  accent-color: var(--ac-primary);
}
.source-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.source-group {
  flex: none;
  max-width: 40%;
  color: var(--ac-secondary-text);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.empty {
  margin: 20px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.bottom-bar {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 4px 6px;
}
.spacer {
  flex: 1;
}
.text-button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  padding: 12px 16px;
  font-size: 14px;
  cursor: pointer;
}
.text-button.muted {
  color: var(--ac-secondary-text);
}
</style>
