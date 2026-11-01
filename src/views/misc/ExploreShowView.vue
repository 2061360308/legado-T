<script setup lang="ts">
import TitleBar from "@/components/android/TitleBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";
</script>

<template>
  <div class="explore-page">
    <TitleBar title="发现" :back="false" />
    <main class="content">
<EmptyState icon="ic_bottom_explore_e" text="发现页暂未接入书源内容" />

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

