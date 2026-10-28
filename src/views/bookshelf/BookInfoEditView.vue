<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { bookGet, bookUpsert } from "@/api/books";
import type { Book } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const route = useRoute();
const router = useRouter();
const bookUrl = String(route.query.bookUrl || "");
const book = ref<Book | null>(null);
const name = ref("");
const author = ref("");
const intro = ref("");
const message = ref("");

async function load() {
  const value = await bookGet(bookUrl);
  book.value = value;
  if (value) {
    name.value = value.name;
    author.value = value.author;
    intro.value = value.intro ?? "";
  }
}

async function save() {
  if (!book.value || !name.value.trim()) return;
  await bookUpsert({
    ...book.value,
    name: name.value.trim(),
    author: author.value.trim(),
    intro: intro.value,
  });
  message.value = "已保存";
  router.back();
}

onMounted(load);
</script>

<template>
  <div class="edit-page">
    <TitleBar title="书籍信息编辑" @back="router.back()">
      <template #actions>
        <button @click="save">保存</button>
      </template>
    </TitleBar>
    <main v-if="book" class="content">
      <div class="top">
        <CoverImage :name="name || book.name" :width="90" :height="130" :radius="2" />
        <div class="fields">
          <label>
            书名
            <input v-model="name" />
          </label>
          <label>
            作者
            <input v-model="author" />
          </label>
        </div>
      </div>
      <label class="intro">
        简介
        <textarea v-model="intro" />
      </label>
      <p class="message">{{ message }}</p>
    </main>
  </div>
</template>

<style scoped>
.edit-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}
.top {
  display: flex;
  gap: 8px;
}
.fields {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 5px;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
input,
textarea {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 10px;
  font-size: 14px;
}
.intro {
  margin-top: 12px;
}
textarea {
  min-height: 180px;
  resize: vertical;
}
.message {
  margin: 8px 5px;
  color: var(--ac-primary);
  font-size: 12px;
}
</style>
