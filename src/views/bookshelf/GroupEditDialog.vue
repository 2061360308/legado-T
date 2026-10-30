<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { groupCoverSave, groupDelete, groupSave } from "@/api/books";
import type { BookGroup } from "@/api/bindings";
import CoverImage from "@/components/android/CoverImage.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const props = defineProps<{
  modelValue: boolean;
  group: BookGroup | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  saved: [group: BookGroup | null];
}>();

const SORT_OPTIONS = [
  { value: -1, label: "默认排序" },
  { value: 0, label: "最近阅读" },
  { value: 1, label: "更新时间" },
  { value: 2, label: "书名" },
  { value: 3, label: "手动" },
  { value: 4, label: "综合" },
  { value: 5, label: "作者" },
];

const name = ref("");
const coverUrl = ref("");
const bookSort = ref(-1);
const enableRefresh = ref(true);
const error = ref("");
const busy = ref(false);
const title = computed(() => (props.group ? "编辑分组" : "新建分组"));

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    name.value = props.group?.groupName ?? "";
    coverUrl.value = props.group?.coverUrl ?? "";
    bookSort.value = props.group?.bookSort ?? -1;
    enableRefresh.value = props.group?.enableRefresh ?? true;
    error.value = "";
    busy.value = false;
  },
  { immediate: true },
);

async function selectCover() {
  const picked = await open({
    multiple: false,
    filters: [{ name: "图片", extensions: ["jpg", "jpeg", "png", "webp", "gif", "bmp"] }],
  });
  if (typeof picked !== "string" || !picked) return;
  try {
    coverUrl.value = await groupCoverSave(picked);
  } catch (e) {
    error.value = String(e);
  }
}

async function save() {
  if (busy.value) return;
  const groupName = name.value.trim();
  if (!groupName) {
    error.value = "分组名称不能为空";
    return;
  }
  busy.value = true;
  try {
    const saved = await groupSave({
      groupId: props.group?.groupId ?? 0,
      groupName,
      coverUrl: coverUrl.value || null,
      order: props.group?.order ?? 0,
      bookSort: bookSort.value,
      show: props.group?.show ?? true,
      enableRefresh: enableRefresh.value,
    });
    emit("saved", saved);
    emit("update:modelValue", false);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function remove() {
  if (!props.group || busy.value) return;
  if (!confirm(`删除分组「${props.group.groupName}」？`)) return;
  busy.value = true;
  try {
    await groupDelete(props.group.groupId);
    emit("saved", null);
    emit("update:modelValue", false);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    :show-header="false"
    :body-padding="false"
    width="min(90vw, 420px)"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="group-edit">
      <header class="group-edit-bar">
        <span>{{ title }}</span>
      </header>
      <div class="group-edit-body">
        <button class="cover-button" type="button" @click="selectCover">
          <CoverImage
            :name="name || '分组'"
            :src="coverUrl"
            :width="90"
            :height="126"
            :radius="2"
          />
        </button>
        <div class="fields">
          <label class="name-field">
            <span>分组名称</span>
            <input v-model="name" maxlength="80" placeholder="分组名称" />
          </label>
          <div class="sort-row">
            <span>排序</span>
            <select v-model.number="bookSort">
              <option v-for="item in SORT_OPTIONS" :key="item.value" :value="item.value">
                {{ item.label }}
              </option>
            </select>
          </div>
          <label class="refresh-row">
            <input v-model="enableRefresh" type="checkbox" />
            <span>允许下拉刷新</span>
          </label>
        </div>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
      <footer class="group-edit-actions">
        <button v-if="group" class="delete" :disabled="busy" @click="remove">删除</button>
        <span class="spacer" />
        <button :disabled="busy" @click="emit('update:modelValue', false)">取消</button>
        <button class="accent" :disabled="busy" @click="save">确认</button>
      </footer>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.group-edit {
  display: flex;
  flex-direction: column;
  background: var(--ac-background-menu);
  color: var(--ac-primary-text);
}
.group-edit-bar {
  min-height: 52px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  background: var(--ac-background-menu);
  border-bottom: 1px solid var(--ac-divider);
  font-size: 17px;
}
.group-edit-body {
  display: flex;
  gap: 6px;
  padding: 6px;
}
.cover-button {
  flex: none;
  border: none;
  background: transparent;
  padding: 0;
  cursor: pointer;
}
.fields {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.name-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.name-field input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  font-size: 14px;
}
.sort-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
.sort-row select {
  flex: 1;
  min-width: 0;
  padding: 5px 6px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
}
.refresh-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}
.refresh-row input {
  accent-color: var(--ac-primary);
}
.error {
  margin: 0;
  padding: 4px 12px;
  color: var(--ac-error);
  font-size: 12px;
}
.group-edit-actions {
  display: flex;
  align-items: center;
  padding: 6px 12px 10px;
}
.group-edit-actions button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px 10px;
  font-size: 14px;
  cursor: pointer;
}
.group-edit-actions button.delete {
  color: var(--ac-error);
}
.group-edit-actions button.accent {
  color: var(--ac-primary);
}
.group-edit-actions button:disabled {
  opacity: 0.5;
}
.spacer {
  flex: 1;
}
</style>
