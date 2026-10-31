<script setup lang="ts">
import { ref, watch } from "vue";
import { groupList } from "@/core/application/books";
import type { BookGroup } from "@/core/domain";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import GroupEditDialog from "./GroupEditDialog.vue";

const props = defineProps<{
  modelValue: boolean;
  groupId: number;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [groupId: number];
}>();

const groups = ref<BookGroup[]>([]);
const draft = ref(0);
const editVisible = ref(false);
const editingGroup = ref<BookGroup | null>(null);
const error = ref("");

function hasBit(value: number, bit: number): boolean {
  if (!bit) return false;
  return Math.floor(value / bit) % 2 === 1;
}

function toggle(groupId: number) {
  if (hasBit(draft.value, groupId)) {
    draft.value -= groupId;
  } else {
    draft.value += groupId;
  }
}

async function loadGroups() {
  try {
    groups.value = (await groupList()).filter((group) => group.groupId > 0);
    error.value = "";
  } catch (e) {
    error.value = String(e);
  }
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    draft.value = props.groupId;
    error.value = "";
    loadGroups();
  },
  { immediate: true },
);

function openAdd() {
  editingGroup.value = null;
  editVisible.value = true;
}

function openEdit(group: BookGroup) {
  editingGroup.value = group;
  editVisible.value = true;
}

async function onSaved(group: BookGroup | null) {
  if (group === null && editingGroup.value) {
    const removed = editingGroup.value.groupId;
    if (hasBit(draft.value, removed)) {
      draft.value -= removed;
    }
  }
  await loadGroups();
  if (group) {
    const updated = groups.value.find((item) => item.groupId === group.groupId);
    if (updated) editingGroup.value = updated;
  }
}

function cancel() {
  emit("update:modelValue", false);
}

function confirm() {
  emit("confirm", draft.value);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    :show-header="false"
    :body-padding="false"
    width="90vw"
    height="86vh"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="group-select">
      <header class="group-select-bar">
        <span class="title">分组选择</span>
        <button aria-label="新建分组" @click="openAdd">
          <Icon name="ic_add" :size="22" />
        </button>
      </header>

      <div class="group-select-list">
        <div v-for="group in groups" :key="group.groupId" class="group-row">
          <label class="group-check">
            <input
              type="checkbox"
              :checked="hasBit(draft, group.groupId)"
              @change="toggle(group.groupId)"
            />
            <span>{{ group.groupName }}</span>
          </label>
          <button class="edit" @click="openEdit(group)">编辑</button>
        </div>
        <p v-if="error" class="error">{{ error }}</p>
      </div>

      <footer class="group-select-actions">
        <span class="spacer" />
        <button @click="cancel">取消</button>
        <button class="accent" @click="confirm">确认</button>
      </footer>
    </div>
  </AndroidDialog>

  <GroupEditDialog v-model="editVisible" :group="editingGroup" @saved="onSaved" />
</template>

<style scoped>
.group-select {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ac-background);
  color: var(--ac-primary-text);
}
.group-select-bar {
  flex: none;
  min-height: 56px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 8px 0 16px;
  background: var(--ac-primary);
  color: #fff;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
}
.group-select-bar .title {
  flex: 1;
  font-size: 17px;
}
.group-select-bar button {
  width: 44px;
  height: 44px;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
}
.group-select-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  background: var(--ac-background);
}
.group-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 44px;
  padding: 4px 8px;
  background: var(--ac-background);
  border-bottom: 1px solid var(--ac-divider);
}
.group-check {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  cursor: pointer;
}
.group-check input {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.group-check span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.edit {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px;
  font-size: 14px;
  cursor: pointer;
}
.group-select-actions {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px 8px;
  background: var(--ac-background);
}
.group-select-actions button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 12px;
  font-size: 14px;
  cursor: pointer;
}
.group-select-actions button.accent {
  color: var(--ac-primary);
}
.spacer {
  flex: 1;
}
.error {
  margin: 8px;
  color: var(--ac-error);
  font-size: 12px;
}
</style>
