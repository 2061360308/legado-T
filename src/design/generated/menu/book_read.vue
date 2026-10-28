<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-menu">
<div class="menu-group"><div class="menu-group"><div class="menu-item" data-depth="2">
  <AndroidIcon name="ic_exchange" :size="20" />
  <span class="menu-title">{{ "换源" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="2">
  <AndroidIcon name="ic_refresh_black_24dp" :size="20" />
  <span class="menu-title">{{ "刷新" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="2">
  <AndroidIcon name="ic_download_line" :size="20" />
  <span class="menu-title">{{ "离线缓存" }}</span>
  <span class="menu-meta">never</span>
  
</div></div>
<div class="menu-group"><div class="menu-item" data-depth="2">
  <AndroidIcon name="ic_exchange" :size="20" />
  <span class="menu-title">{{ "TXT 目录规则" }}</span>
  <span class="menu-meta">never</span>
  
</div></div>
<div class="menu-group"><div class="menu-item" data-depth="2">
  <AndroidIcon name="ic_translate" :size="20" />
  <span class="menu-title">{{ "设置编码" }}</span>
  <span class="menu-meta">never</span>
  
</div></div>
<div class="menu-group"><div class="menu-item" data-depth="2">
  
  <span class="menu-title">{{ "删除ruby标签" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="2">
  
  <span class="menu-title">{{ "删除h标签" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div></div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "添加书签" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "编辑内容" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "翻页动画（本书）" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "拉取云端进度" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "覆盖云端进度" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "反转内容" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "模拟追读" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "替换净化" }}</span>
  <span class="menu-meta">never · checkable · checked</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "移除重复标题" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "重新分段" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "段评" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "图片样式" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "更新目录" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "起效的替换" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "日志" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "帮助" }}</span>
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
