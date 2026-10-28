<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-menu">
<div class="menu-group"><div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_search" :size="20" />
  <span class="menu-title">{{ "搜索" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_refresh_black_24dp" :size="20" />
  <span class="menu-title">{{ "更新目录" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_add" :size="20" />
  <span class="menu-title">{{ "添加本地" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_add" :size="20" />
  <span class="menu-title">{{ "远程书籍" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_add_online" :size="20" />
  <span class="menu-title">{{ "添加网址" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_arrange" :size="20" />
  <span class="menu-title">{{ "书架管理" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_download_line" :size="20" />
  <span class="menu-title">{{ "缓存/导出" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_groups" :size="20" />
  <span class="menu-title">{{ "分组管理" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_view_quilt" :size="20" />
  <span class="menu-title">{{ "书架布局" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_export" :size="20" />
  <span class="menu-title">{{ "导出书单" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_import" :size="20" />
  <span class="menu-title">{{ "导入书单" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_cfg_about" :size="20" />
  <span class="menu-title">{{ "日志" }}</span>
  <span class="menu-meta">never</span>
  
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
