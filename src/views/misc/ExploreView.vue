<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const keyword = ref("");
const hasSources = false;
const groupMenuVisible = ref(false);
const groupItems = computed<PopupMenuItem[]>(() => [
  { key: "empty", label: "暂无分组", muted: true },
]);
function onGroupSelect() {
  groupMenuVisible.value = false;
}

function onSearch() {
  /* 无发现源时搜索无目标 */
}
</script>

<template>
  <div class="explore-page">
    <TitleBar title="发现" :back="false">
      <template #content>
        <label class="search">
          <input
            v-model="keyword"
            type="search"
            placeholder="搜索"
            @keyup.enter="onSearch"
          />
        </label>
      </template>
      <template #actions>
        <button aria-label="分组" @click="groupMenuVisible = !groupMenuVisible">
          <Icon name="ic_groups" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="groupMenuVisible" :items="groupItems" @select="onGroupSelect" />

    <main class="explore-list">
      <EmptyState
        v-if="!hasSources"
        icon="ic_bottom_explore_e"
        text="当前没有发现源哦～"
        action="去书源管理添加"
        @action="router.push('/book-source')"
      />
    </main>
  </div>
</template>

<style scoped>
.explore-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.search {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
}
.search input {
  width: 100%;
  height: 30px;
  border: none;
  border-radius: 17px;
  background: rgba(255, 255, 255, 0.22);
  color: #fff;
  font-size: 13px;
  padding: 0 16px;
  outline: none;
}
.search input::placeholder {
  color: rgba(255, 255, 255, 0.65);
}
.explore-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
</style>
