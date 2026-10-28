<script setup lang="ts">
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";

const router = useRouter();
</script>

<template>
  <div class="donate-page">
    <TitleBar title="捐赠" subtitle="您的支持是我更新的动力" @back="router.back()" />
    <main class="content">
      <div class="card">
        <p class="title">感谢支持</p>
        <p class="desc">本页面为原版捐赠页面的视觉壳子，捐赠入口暂未接入。</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.donate-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  padding: 8px;
}
.card {
  padding: 12px;
  background: var(--ac-background-card);
  border-radius: 2px;
}
.title {
  margin: 0 0 8px;
  font-size: 16px;
  font-weight: 600;
}
.desc {
  margin: 0;
  font-size: 13px;
  line-height: 1.7;
  color: var(--ac-secondary-text);
}
</style>
