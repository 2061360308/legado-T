<script setup lang="ts">
import TitleBar from "@/components/android/TitleBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";
</script>

<template>
  <div class="explore-page">
    <TitleBar title="鍙戠幇" :back="false" />
    <main class="content">
      <EmptyState icon="ic_bottom_explore_e" text="鍙戠幇椤垫殏鏈帴鍏ヤ功婧愬唴瀹? />
    </main>
  </div>
</template>

<style scoped>
.explore-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
}
</style>

