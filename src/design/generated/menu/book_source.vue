<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-menu">
<div class="menu-group"><div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_sort" :size="20" />
  <span class="menu-title">{{ "排序" }}</span>
  <span class="menu-meta">never</span>
  <div class="menu-children"><div class="menu-group"><div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "反序" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-group"><div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "手动排序" }}</span>
  <span class="menu-meta">never · checkable · checked</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "智能排序" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "名称排序" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "地址排序" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "更新时间排序" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "响应时间排序" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="4">
  
  <span class="menu-title">{{ "是否启用" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div></div></div></div>
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_groups" :size="20" />
  <span class="menu-title">{{ "分组" }}</span>
  <span class="menu-meta">never</span>
  <div class="menu-children"><div class="menu-group"><div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "分组管理" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "已启用" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "已禁用" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "需要登录" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "未分组" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "已启用发现" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="3">
  
  <span class="menu-title">{{ "已禁用发现" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-group"></div></div></div>
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_add" :size="20" />
  <span class="menu-title">{{ "新建书源" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_import" :size="20" />
  <span class="menu-title">{{ "本地导入" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_import" :size="20" />
  <span class="menu-title">{{ "网络导入" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_import" :size="20" />
  <span class="menu-title">{{ "二维码导入" }}</span>
  <span class="menu-meta">never</span>
  
</div>
<div class="menu-item" data-depth="1">
  
  <span class="menu-title">{{ "按域名分组显示" }}</span>
  <span class="menu-meta">never · checkable</span>
  
</div>
<div class="menu-item" data-depth="1">
  <AndroidIcon name="ic_help" :size="20" />
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
