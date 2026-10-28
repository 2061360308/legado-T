<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const router = useRouter();
const type = ref("文本");
const enabled = ref(true);
const discovery = ref(true);
const cookie = ref(true);
const tab = ref(0);
const name = ref("");
const url = ref("");
const content = ref("");
const message = ref("");
const tabs = ["正文", "发现", "详情", "目录", "其他"];

function save() {
  message.value = "书源保存功能暂未接入";
}
</script>

<template>
  <div class="source-edit">
    <TitleBar title="编辑书源" @back="router.back()">
      <template #actions>
        <button @click="save"><Icon name="ic_save" :size="24" /></button>
      </template>
    </TitleBar>

    <div class="top-row">
      <span class="label">书源类型</span>
      <select v-model="type">
        <option>文本</option>
        <option>音频</option>
        <option>图片</option>
      </select>
      <label><input v-model="enabled" type="checkbox" /> 启用</label>
      <label><input v-model="discovery" type="checkbox" /> 发现</label>
      <label><input v-model="cookie" type="checkbox" /> 自动保存 Cookie</label>
    </div>

    <nav class="tabs">
      <button
        v-for="(item, index) in tabs"
        :key="item"
        :class="{ active: tab === index }"
        @click="tab = index"
      >
        {{ item }}
      </button>
    </nav>

    <main class="form">
      <label>书源名称<input v-model="name" placeholder="书源名称" /></label>
      <label>书源 URL<input v-model="url" placeholder="https://" /></label>
      <label class="code">
        规则内容
        <textarea v-model="content" spellcheck="false" placeholder="规则代码" />
      </label>
      <p v-if="message" class="message">{{ message }}</p>
    </main>
  </div>
</template>

<style scoped>
.source-edit {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.top-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
  font-size: 13px;
  white-space: nowrap;
}
.label {
  color: var(--ac-secondary-text);
}
select {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 4px 6px;
}
.top-row label {
  display: flex;
  align-items: center;
  gap: 4px;
}
input[type="checkbox"] {
  accent-color: var(--ac-primary);
}
.tabs {
  display: flex;
  gap: 2px;
  padding: 0 8px;
  border-bottom: 1px solid var(--ac-divider);
}
.tabs button {
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 10px 12px;
  font-size: 13px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
}
.tabs button.active {
  color: var(--ac-primary);
  border-bottom-color: var(--ac-primary);
}
.form {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px 24px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
input:not([type="checkbox"]),
textarea {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px 10px;
  border-radius: 2px;
  font-size: 14px;
}
.code {
  flex: 1;
}
textarea {
  min-height: 260px;
  resize: vertical;
  font-family: Consolas, monospace;
  font-size: 13px;
}
.message {
  margin: 0;
  color: var(--ac-primary);
  font-size: 12px;
}
</style>
