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
  <AndroidIcon name="ic_toc" :size="20" />
  <span class="menu-title">{{ "目录" }}</span>
  <span class="menu-meta">never</span>
  
</div></div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "预下载" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "禁用漫画缩放" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "禁用点击翻页" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "开启自动翻页" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "开启滚动" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "翻页速度 %s" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "水平滚动" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "禁用水平翻页效果" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "禁用翻页动画" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "页脚配置" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "滤镜" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "隐藏漫画列表标题" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "墨水屏" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "墨水屏设置" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "开启图片灰色" }}</span>
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
