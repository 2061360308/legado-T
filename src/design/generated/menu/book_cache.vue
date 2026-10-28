<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-menu">
<div class="menu-group"><div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_play_24dp" :size="20" />
  <span class="menu-title">{{ "下载" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_groups" :size="20" />
  <span class="menu-title">{{ "分组" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "导出所有" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "替换净化" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "自定义Epub导出章节" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "导出到 WebDav" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "TXT 不导出章节名" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "TXT 导出图片" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "多线程导出" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "导出文件夹" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "导出文件名" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "导出格式" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "导出编码" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
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
