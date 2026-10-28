<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-menu">
<div class="menu-group"><div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_baseline_sort_24" :size="20" />
  <span class="menu-title">{{ "排序" }}</span>
  <span class="menu-meta">never</span>
  <div class="menu-children"><div class="menu-group"><div class="menu-group"><div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "名称排序" }}</span>
  <span class="menu-meta">never · checked</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "阅读时长排序" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "阅读时间排序" }}</span>
  <span class="menu-meta">never</span>
  
</div></div></div></div>
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "开启记录" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div></div>
  </div>
</template>

<style scoped>
.generated-menu { width:100%; min-height:100%; background:var(--ac-background); color:var(--ac-primary-text); }
.menu-item { display:flex; align-items:center; gap:10px; padding:11px 16px; border-bottom:1px solid var(--ac-divider); }
.menu-title { flex:1; font-size:14px; }
.menu-meta { font-size:11px; color:var(--ac-secondary-text); }
.menu-group,.menu-children { display:flex; flex-direction:column; width:100%; }
.menu-children { padding-left:20px; }
</style>
