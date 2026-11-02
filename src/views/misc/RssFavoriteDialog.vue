<script setup lang="ts">
import { ref, watch } from "vue";
import {
  rssStarDelete,
  rssStarGet,
  rssStarSave,
  rssStarUpdate,
} from "@/core/application/rss";
import type { RssArticle, RssStar } from "@/core/domain";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const props = defineProps<{
  modelValue: boolean;
  article: RssArticle | null;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  changed: [];
}>();
const title = ref("");
const group = ref("默认分组");
const star = ref<RssStar | null>(null);
const message = ref("");

async function load() {
  const article = props.article;
  if (!article) return;
  star.value = await rssStarGet(article.origin, article.link);
  title.value = star.value?.title || article.title || "";
  group.value = star.value?.group || article.group || "默认分组";
}

async function save() {
  const article = props.article;
  if (!article) return;
  try {
    if (star.value) {
      await rssStarUpdate(article.origin, article.link, title.value, group.value);
    } else {
      await rssStarSave({ ...article, title: title.value, group: group.value }, group.value);
    }
    emit("changed");
    emit("update:modelValue", false);
  } catch (error) {
    message.value = String(error);
  }
}

async function remove() {
  const article = props.article;
  if (!article) return;
  if (!confirm("删除这条收藏？")) return;
  await rssStarDelete(article.origin, article.link);
  emit("changed");
  emit("update:modelValue", false);
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      message.value = "";
      void load();
    }
  },
);
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="收藏"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="form">
      <label>标题<input v-model="title" /></label>
      <label>分组<input v-model="group" placeholder="默认分组" /></label>
      <p v-if="message" class="message">{{ message }}</p>
    </div>
    <template #footer>
      <button v-if="star" class="danger" @click="remove">删除</button>
      <span class="spacer" />
      <button @click="emit('update:modelValue', false)">取消</button>
      <button @click="save">确定</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.form input {
  padding: 9px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
}
.message {
  margin: 0;
  color: var(--ac-error);
  font-size: 12px;
}
.spacer {
  flex: 1;
}
.danger {
  color: var(--ac-error);
}
</style>
