<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { groupList, groupSave } from "@/core/application/books";
import type { BookGroup } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import GroupEditDialog from "./GroupEditDialog.vue";

const router = useRouter();
const groups = ref<BookGroup[]>([]);
const message = ref("");
const editVisible = ref(false);
const editingGroup = ref<BookGroup | null>(null);

async function load() {
  groups.value = (await groupList())
    .filter((group) => group.groupId > 0)
    .sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
}

function addGroup() {
  editingGroup.value = null;
  editVisible.value = true;
}

function editGroup(group: BookGroup) {
  editingGroup.value = group;
  editVisible.value = true;
}

async function toggleShow(group: BookGroup, show: boolean) {
  await groupSave({ ...group, show });
  await load();
}

async function onSaved() {
  editVisible.value = false;
  await load();
  message.value = "分组已更新";
}

async function move(index: number, delta: number) {
  const target = index + delta;
  if (target < 0 || target >= groups.value.length) return;
  const a = groups.value[index];
  const b = groups.value[target];
  await groupSave({ ...a, order: b.order ?? target });
  await groupSave({ ...b, order: a.order ?? index });
  await load();
}

onMounted(load);
</script>

<template>
  <div class="group-manage">
    <div class="group-dialog">
      <TitleBar title="分组管理" @back="router.back()">
        <template #actions>
          <button aria-label="新建分组" @click="addGroup">
            <Icon name="ic_add" :size="24" />
          </button>
        </template>
      </TitleBar>

      <p v-if="message" class="message">{{ message }}</p>

      <main class="list">
        <div v-for="(group, index) in groups" :key="group.groupId" class="group-row">
          <button class="move" :disabled="index === 0" @click="move(index, -1)">↑</button>
          <button class="move" :disabled="index === groups.length - 1" @click="move(index, 1)">↓</button>
          <span class="group-name">{{ group.groupName }}</span>
          <label class="show-switch">
            <input
              type="checkbox"
              :checked="group.show !== false"
              @change="toggleShow(group, ($event.target as HTMLInputElement).checked)"
            />
          </label>
          <button class="edit" @click="editGroup(group)">编辑</button>
        </div>
        <p v-if="groups.length === 0" class="empty">暂无分组</p>
      </main>

      <footer class="dialog-footer">
        <button @click="router.back()">确定</button>
      </footer>
    </div>

    <GroupEditDialog v-model="editVisible" :group="editingGroup" @saved="onSaved" />
  </div>
</template>

<style scoped>
.group-manage {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.16);
}
.group-dialog {
  width: 90vw;
  height: 86vh;
  max-width: 640px;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
  box-shadow: 0 4px 18px rgba(0, 0, 0, 0.25);
}
.dialog-footer {
  flex: none;
  display: flex;
  justify-content: flex-end;
  padding: 6px 12px;
  border-top: 1px solid var(--ac-divider);
}
.dialog-footer button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 14px;
  padding: 8px 12px;
}
.empty {
  margin: 40px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  overflow-y: auto;
}
.group-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 52px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.group-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
}
.show-switch {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.show-switch input {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.move {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  width: 26px;
  height: 26px;
  padding: 0;
  font-size: 12px;
}
.move:disabled {
  opacity: 0.35;
}
.edit {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 13px;
  padding: 6px 8px;
  cursor: pointer;
}
</style>
