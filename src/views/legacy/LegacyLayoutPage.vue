<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import AndroidLayoutView from "@/components/android/AndroidLayoutView.vue";

const route = useRoute();
const router = useRouter();
const name = computed(() => String(route.params.name || ""));
const group = computed(() => String(route.query.group || "layout"));
const title = computed(() => name.value.replace(/_/g, " "));
</script>

<template>
  <div class="legacy-page">
    <TitleBar :title="title" @back="router.back()" />
    <div class="banner">原版界面壳子 · 功能暂未接入</div>
    <div class="body">
      <AndroidLayoutView :name="name" :group="group" />
    </div>
  </div>
</template>

<style scoped>
.legacy-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.banner {
  flex: none;
  padding: 6px 16px;
  font-size: 12px;
  color: var(--ac-secondary-text);
  background: var(--ac-background-card);
  border-bottom: 1px solid var(--ac-divider);
}
.body {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
</style>
