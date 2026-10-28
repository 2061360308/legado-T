<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import index from "@/design/layouts/index.json";
import menuIndex from "@/design/menus/index.json";
import prefIndex from "@/design/prefs/index.json";

interface Entry {
  group: string;
  name: string;
  file: string;
}

const router = useRouter();
const entries = index as Entry[];
const groups = computed(() => {
  const map = new Map<string, Entry[]>();
  for (const entry of entries) {
    const list = map.get(entry.group) ?? [];
    list.push(entry);
    map.set(entry.group, list);
  }
  return [...map.entries()];
});

const menus = menuIndex as Entry[];
const prefs = prefIndex as Entry[];

function open(entry: Entry) {
  router.push({ path: `/legacy/${entry.name}`, query: { group: entry.group } });
}

function openMenu(entry: Entry) {
  router.push(`/legacy/menu/${entry.name}`);
}

function openPref(entry: Entry) {
  router.push(`/legacy/pref/${entry.name}`);
}
</script>

<template>
  <div class="gallery">
    <TitleBar title="原版界面清单" @back="router.back()" />
    <main class="list">
      <section v-for="[group, items] in groups" :key="group">
        <h2>{{ group }} · {{ items.length }}</h2>
        <button v-for="entry in items" :key="entry.file" @click="open(entry)">
          {{ entry.name }}
        </button>
      </section>
      <section>
        <h2>menu · {{ menus.length }}</h2>
        <button v-for="entry in menus" :key="entry.file" @click="openMenu(entry)">
          {{ entry.name }}
        </button>
      </section>
      <section>
        <h2>xml · {{ prefs.length }}</h2>
        <button v-for="entry in prefs" :key="entry.file" @click="openPref(entry)">
          {{ entry.name }}
        </button>
      </section>
    </main>
  </div>
</template>

<style scoped>
.gallery {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.list {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 24px;
}
h2 {
  margin: 0;
  padding: 10px 20px 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--ac-primary);
}
button {
  width: 100%;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
  padding: 11px 20px;
  font-size: 13px;
  cursor: pointer;
}
button:hover {
  background: var(--ac-transparent-10);
}
</style>
