<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";

const router = useRouter();
const keyword = ref("");
const message = ref("");
</script>

<template>
  <div class="source-page">
    <TitleBar title="书源" @back="router.back()">
      <template #content>
        <input v-model="keyword" class="search" placeholder="搜索书源" />
      </template>
      <template #actions>
        <button aria-label="新增" @click="router.push('/book-source-edit')">
          <Icon name="ic_add" :size="24" />
        </button>
      </template>
    </TitleBar>
    <p v-if="message" class="message">{{ message }}</p>
    <main class="list">
      <EmptyState icon="ic_cfg_source" text="暂无书源" action="新增书源" @action="router.push('/book-source-edit')" />
    </main>
    <SelectActionBar
      :count="0"
      @enable="message = '批量启用暂未接入'"
      @disable="message = '批量禁用暂未接入'"
      @remove="message = '批量删除暂未接入'"
      @cancel="message = ''"
    />
  </div>
</template>

<style scoped>
.source-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.search {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
  padding: 6px 10px;
  border: none;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
  outline: none;
}
.search::placeholder {
  color: rgba(255, 255, 255, 0.6);
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
</style>
