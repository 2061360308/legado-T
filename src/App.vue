<script setup lang="ts">
import { computed } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import BottomNav, { type MainTab } from "@/components/android/BottomNav.vue";

const route = useRoute();
const router = useRouter();

const hideNav = [
  "/reader",
  "/audio",
  "/book-search",
  "/book-source",
  "/book-source-edit",
  "/cache-books",
  "/toc",
  "/import",
  "/arrange-books",
  "/group-manage",
  "/book-info",
  "/book-info-edit",
  "/toc-rules",
  "/replace",
  "/replace-edit",
  "/dict-rules",
  "/dict",
  "/theme",
  "/cover-config",
  "/welcome-config",
  "/welcome",
  "/read-schemes",
  "/backup",
  "/webdav",
  "/aloud",
  "/other-settings",
  "/bookmarks",
  "/read-records",
  "/files",
  "/about",
  "/legacy",
  "/generated",
  "/shell",
];
const showNav = computed(
  () => !hideNav.some((path) => route.path === path || route.path.startsWith(`${path}/`)),
);
const activeTab = computed<MainTab>(() => {
  if (route.path === "/config") return "config";
  if (route.path === "/explore") return "discover";
  if (route.path === "/rss-sources") return "rss";
  return "bookshelf";
});

function navigate(tab: MainTab) {
  if (tab === "config") router.push("/config");
  else if (tab === "discover") router.push("/explore");
  else if (tab === "rss") router.push("/rss-sources");
  else router.push("/");
}
</script>

<template>
  <div class="app-bg" aria-hidden="true" />
  <div class="app-shell">
    <div class="app-content" :class="{ 'with-nav': showNav }">
      <RouterView />
    </div>
    <BottomNav v-if="showNav" :model-value="activeTab" @update:model-value="navigate" />
  </div>
</template>

<style scoped>
.app-bg {
  position: fixed;
  inset: 0;
  z-index: 0;
  pointer-events: none;
  background-image: var(--app-bg-image, none);
  background-position: center;
  background-size: cover;
  filter: blur(var(--app-bg-blur, 0px));
  transform: scale(1.06);
}
.app-shell {
  position: relative;
  z-index: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.app-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
}
.app-content.with-nav {
  padding-bottom: calc(50px + max(env(safe-area-inset-bottom), var(--safe-bottom, 0px)));
}
</style>
