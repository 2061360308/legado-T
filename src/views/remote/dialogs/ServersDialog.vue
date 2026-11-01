<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import { DEFAULT_REMOTE_SERVER_ID, type RemoteServer } from "@/core/domain";

const props = defineProps<{
  modelValue: boolean;
  servers: RemoteServer[];
  selectedId: number;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  add: [];
  edit: [server: RemoteServer];
  delete: [server: RemoteServer];
  select: [id: number];
  default: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});

const choice = ref(props.selectedId);

watch(
  () => props.modelValue,
  (open) => {
    if (open) choice.value = props.selectedId;
  },
);

watch(
  () => props.servers,
  (servers) => {
    if (!servers.some((server) => server.id === choice.value)) {
      choice.value = servers[0]?.id ?? DEFAULT_REMOTE_SERVER_ID;
    }
  },
  { deep: true },
);

function setDefault() {
  emit("default");
  visible.value = false;
}

function confirm() {
  emit("select", choice.value);
  visible.value = false;
}
</script>

<template>
  <AndroidDialog
    v-model="visible"
    title="服务器配置"
    mode="center"
    width="min(100%, 560px)"
  >
    <template #actions>
      <button class="header-action" aria-label="新增服务器" @click="emit('add')">
        <Icon name="ic_add" :size="22" />
      </button>
    </template>
    <div class="server-list">
      <p v-if="!servers.length" class="empty">
        还没有服务器，请点击右上角添加。
      </p>
      <label
        v-for="server in servers"
        :key="server.id"
        class="server-row"
        :class="{ active: choice === server.id }"
      >
        <input v-model="choice" type="radio" :value="server.id" />
        <span class="server-name">{{ server.name }}</span>
        <span class="server-type">{{ server.type === "s3" ? "S3" : "WEBDAV" }}</span>
        <button
          type="button"
          class="icon-button"
          aria-label="编辑"
          @click.stop="emit('edit', server)"
        >
          <Icon name="ic_edit" :size="18" />
        </button>
        <button
          v-if="server.id !== DEFAULT_REMOTE_SERVER_ID"
          type="button"
          class="icon-button danger"
          aria-label="删除"
          @click.stop="emit('delete', server)"
        >
          <Icon name="ic_outline_delete" :size="18" />
        </button>
      </label>
    </div>
    <template #footer>
      <div class="servers-footer">
        <button @click="setDefault">默认</button>
        <span class="spacer" />
        <button @click="visible = false">取消</button>
        <button class="accent" @click="confirm">确定</button>
      </div>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.server-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.server-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 44px;
  padding: 0 4px;
  border-radius: 4px;
  cursor: pointer;
}
.server-row.active {
  background: var(--ac-transparent-10);
}
.server-row input {
  margin: 0;
}
.server-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.server-type {
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--ac-primary-text);
}
.icon-button:hover {
  background: var(--ac-transparent-10);
}
.icon-button.danger {
  color: var(--ac-error);
}
.empty {
  margin: 16px;
  color: var(--ac-secondary-text);
  text-align: center;
}
.servers-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}
.spacer {
  flex: 1;
}
.servers-footer button {
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 6px 10px;
  border-radius: 4px;
}
.servers-footer button.accent {
  color: var(--ac-primary);
}
.servers-footer button:hover {
  background: var(--ac-transparent-10);
}
.header-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
</style>
