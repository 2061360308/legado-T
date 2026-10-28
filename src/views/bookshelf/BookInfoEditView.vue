<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import {
  bookCoverRefresh,
  bookCoverSave,
  bookGet,
  bookUpsert,
} from "@/api/books";
import type { Book } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const route = useRoute();
const router = useRouter();
const bookUrl = String(route.query.bookUrl || "");
const book = ref<Book | null>(null);
const name = ref("");
const author = ref("");
const typeIndex = ref(0);
const coverUrl = ref("");
const intro = ref("");
const message = ref("");

const typeOptions = ["Text", "Audio", "Image"];

function displayCover(value: Book): string {
  return value.customCoverUrl || value.coverUrl || "";
}

function displayIntro(value: Book): string {
  return value.customIntro || value.intro || "";
}

function typeOf(value: Book): number {
  const type = value.type ?? 0;
  if (type & 4) return 2;
  if (type & 2) return 1;
  return 0;
}

async function load() {
  const value = await bookGet(bookUrl);
  book.value = value;
  if (value) {
    name.value = value.name;
    author.value = value.author;
    typeIndex.value = typeOf(value);
    coverUrl.value = displayCover(value);
    intro.value = displayIntro(value);
  }
}

async function selectCover() {
  const picked = await open({
    multiple: false,
    filters: [{ name: "图片", extensions: ["jpg", "jpeg", "png", "webp", "gif", "bmp"] }],
  });
  if (typeof picked === "string" && picked) {
    try {
      coverUrl.value = await bookCoverSave(bookUrl, picked);
    } catch (error) {
      message.value = String(error);
    }
  }
}

async function refreshCover() {
  try {
    const cover = await bookCoverRefresh(bookUrl);
    if (cover) {
      coverUrl.value = cover;
      message.value = "封面已刷新";
    } else {
      message.value = "没有找到可用的封面";
    }
  } catch (error) {
    message.value = String(error);
  }
}

function changeCoverSource() {
  message.value = "本地书籍暂不支持在线换封面";
}

async function save() {
  if (!book.value || !name.value.trim()) return;
  const local = (book.value.type ?? 0) & 16;
  const base = typeIndex.value === 2 ? 4 : typeIndex.value === 1 ? 2 : 1;
  const nextCover = coverUrl.value.trim();
  const nextIntro = intro.value;
  await bookUpsert({
    ...book.value,
    name: name.value.trim(),
    author: author.value.trim(),
    type: base | local,
    customCoverUrl: nextCover && nextCover !== (book.value.coverUrl ?? "") ? nextCover : null,
    customIntro: nextIntro !== (book.value.intro ?? "") ? nextIntro : null,
  });
  router.back();
}

onMounted(load);
</script>

<template>
  <div class="edit-page">
    <TitleBar title="书籍信息编辑" @back="router.back()">
      <template #actions>
        <button class="save-button" @click="save">保存</button>
      </template>
    </TitleBar>
    <main v-if="book" class="content">
      <div class="top">
        <CoverImage :name="name || book.name" :src="coverUrl" :width="90" :height="130" :radius="2" />
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

      <div class="type-row">
        <span>类型：</span>
        <select v-model.number="typeIndex">
          <option v-for="(label, index) in typeOptions" :key="label" :value="index">
            {{ label }}
          </option>
        </select>
      </div>

      <label class="field">
        封面地址
        <input v-model="coverUrl" placeholder="本地路径或网络地址" />
      </label>

      <div class="cover-actions">
        <button @click="selectCover">本地图片</button>
        <button @click="changeCoverSource">换封面源</button>
        <button @click="refreshCover">刷新封面</button>
      </div>

      <label class="field intro-field">
        简介
        <textarea v-model="intro" />
      </label>

      <p v-if="message" class="message">{{ message }}</p>
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
.save-button {
  border: none;
  background: transparent;
  color: inherit;
  font-size: 14px;
  padding: 8px 10px;
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 5px;
}
.top {
  display: flex;
  gap: 8px;
}
.fields {
  flex: 1;
  min-width: 0;
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
textarea,
select {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 10px;
  font-size: 14px;
}
.type-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 8px 5px 0;
  font-size: 14px;
}
.type-row select {
  padding: 4px 8px;
}
.field {
  margin: 10px 5px 0;
}
.cover-actions {
  display: flex;
  gap: 10px;
  padding: 8px 5px 0;
}
.cover-actions button {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  padding: 6px 10px;
  font-size: 13px;
}
.intro-field {
  margin-top: 14px;
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
