<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:100%;flex-direction:column" data-tag="androidx.constraintlayout.widget.ConstraintLayout" data-path="$"><div class="titlebar " style="width:100%;height:auto" data-tag="TitleBar" data-path="$/0"><span class="titlebar-title">{{ "@string/debug_source" }}</span><div class="titlebar-content"><label class="search bg-0" style="width:100%;height:30px;margin-top:10px;margin-bottom:10px" data-tag="io.legado.app.ui.widget.SearchView" data-path="$/0@view_search"><input class="search-input" type="search" placeholder="搜索" /></label></div></div><div class="list " style="width:100%;height:0px" data-tag="RecyclerView" data-path="$/1"><div class="list-row">列表项</div><div class="list-row">列表项</div><div class="list-row">列表项</div></div><div class="progress " style="width:36px;height:36px;margin:6px;display:none" data-tag="RotateLoading" data-path="$/2">加载中</div><div class="box " style="width:100%;height:auto;display:none" data-tag="NoChildScrollNestedScrollView" data-path="$/3"><div class="box " style="width:100%;height:auto;padding:16px;flex-direction:column;background:#FAFAFA" data-tag="LinearLayout" data-path="$/3/0"><span class="text " style="width:100%;height:auto;color:#0000008a" data-tag="TextView" data-path="$/3/0/0">{{ "调试搜索>>输入关键字，如：" }}</span><span class="text " style="width:100%;height:auto;color:#0000008a" data-tag="TextView" data-path="$/3/0/1">{{ "调试发现>>输入发现URL，如：" }}</span><span class="text bg-1" style="width:auto;height:auto;margin:3px;padding-top:4px;padding-bottom:4px;padding-left:12px;padding-right:12px;justify-content:center;align-items:center;color:#000000de;font-size:14px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;overflow:hidden" data-tag="TextView" data-path="$/3/0/2">{{ "系统::http://xxx" }}</span><span class="text " style="width:100%;height:auto;color:#0000008a" data-tag="TextView" data-path="$/3/0/3">{{ "调试详情页>>输入详情页URL，如：" }}</span><span class="text bg-2" style="width:auto;height:auto;margin:3px;padding-top:4px;padding-bottom:4px;padding-left:12px;padding-right:12px;justify-content:center;align-items:center;color:#000000de;font-size:14px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;overflow:hidden" data-tag="TextView" data-path="$/3/0/4">{{ "https://m.qidian.com/book/1015609210" }}</span><span class="text " style="width:100%;height:auto;color:#0000008a" data-tag="TextView" data-path="$/3/0/5">{{ "调试目录页>>输入目录页URL，如：" }}</span><span class="text bg-3" style="width:auto;height:auto;margin:3px;padding-top:4px;padding-bottom:4px;padding-left:12px;padding-right:12px;justify-content:center;align-items:center;color:#000000de;font-size:14px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;overflow:hidden" data-tag="TextView" data-path="$/3/0/6">{{ "++https://www.zhaishuyuan.com/read/30394" }}</span><span class="text " style="width:100%;height:auto;color:#0000008a" data-tag="TextView" data-path="$/3/0/7">{{ "调试正文页>>输入正文页URL，如：" }}</span><span class="text bg-4" style="width:auto;height:auto;margin:3px;padding-top:4px;padding-bottom:4px;padding-left:12px;padding-right:12px;justify-content:center;align-items:center;color:#000000de;font-size:14px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;overflow:hidden" data-tag="TextView" data-path="$/3/0/8">{{ "--https://www.zhaishuyuan.com/chapter/30394/20940996" }}</span><div class="box " style="width:100%;height:auto;flex-direction:row" data-tag="LinearLayout" data-path="$/3/0/9"><span class="text bg-5" style="width:auto;height:auto;margin:3px;padding-top:4px;padding-bottom:4px;padding-left:12px;padding-right:12px;justify-content:center;align-items:center;color:#000000de;font-size:14px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;overflow:hidden" data-tag="TextView" data-path="$/3/0/9/0">{{ "我的" }}</span><span class="text bg-6" style="width:auto;height:auto;margin:3px;padding-top:4px;padding-bottom:4px;padding-left:12px;padding-right:12px;justify-content:center;align-items:center;color:#000000de;font-size:14px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;overflow:hidden" data-tag="TextView" data-path="$/3/0/9/1">{{ "系统" }}</span></div></div></div></div>
  </div>
</template>

<style scoped>
.generated-root { width:100%; min-height:100%; background:var(--ac-background); color:var(--ac-primary-text); }
.box { display:flex; flex-direction:column; box-sizing:border-box; min-width:0; min-height:0; }
.text { display:inline-flex; align-items:center; box-sizing:border-box; min-width:0; }
.switch-row { display:flex; align-items:center; gap:6px; box-sizing:border-box; }
.edit { border:1px solid var(--ac-divider); background:var(--ac-background); color:var(--ac-primary-text); padding:6px 8px; }
.image { display:inline-flex; align-items:center; justify-content:center; }
.image-placeholder { display:inline-flex; align-items:center; justify-content:center; border:1px dashed var(--ac-divider); color:var(--ac-secondary-text); font-size:10px; min-width:24px; min-height:24px; }
.list { display:flex; flex-direction:column; border-top:1px solid var(--ac-divider); border-bottom:1px solid var(--ac-divider); }
.list-row { padding:10px 16px; border-bottom:1px solid var(--ac-divider); color:var(--ac-secondary-text); font-size:13px; }
.titlebar { display:flex; align-items:center; gap:8px; min-height:56px; padding:0 16px; background:var(--ac-primary); color:#fff; font-size:20px; }
.titlebar-subtitle { font-size:12px; opacity:.8; }
.seek { width:100%; }
.seek-wrap { display:flex; align-items:center; justify-content:center; width:100%; height:100%; }
.progress { padding:12px; color:var(--ac-secondary-text); text-align:center; font-size:12px; }
.fab { width:56px; height:56px; border-radius:50%; border:none; background:var(--ac-primary); color:#fff; display:inline-flex; align-items:center; justify-content:center; }
.select-bar { padding:8px 16px; background:var(--ac-background-menu); color:var(--ac-secondary-text); font-size:12px; border-top:1px solid var(--ac-divider); }
.tabs { display:flex; gap:8px; padding:0 12px; border-bottom:1px solid var(--ac-divider); }
.tabs span { padding:10px 8px; font-size:13px; color:var(--ac-secondary-text); }
.bottom-nav { display:flex; justify-content:center; align-items:center; height:50px; background:var(--ac-background); border-top:1px solid var(--ac-divider); color:var(--ac-secondary-text); }
.space { flex:1; }
.custom { display:flex; box-sizing:border-box; }
.titlebar-content { flex:1; min-width:0; display:flex; align-items:center; }
.search { display:flex; align-items:center; width:100%; height:100%; box-sizing:border-box; }
.search-input { flex:1; min-width:0; border:none; outline:none; background:transparent; color:inherit; font-size:13px; padding:0 12px; }
.bg-0{background-image:url("/src/assets/android/drawable/bg_searchview.svg");background-size:cover;background-position:center;}
.bg-1{background-image:url("/src/assets/android/drawable/selector_fillet_btn_bg.svg");background-size:cover;background-position:center;}
.bg-2{background-image:url("/src/assets/android/drawable/selector_fillet_btn_bg.svg");background-size:cover;background-position:center;}
.bg-3{background-image:url("/src/assets/android/drawable/selector_fillet_btn_bg.svg");background-size:cover;background-position:center;}
.bg-4{background-image:url("/src/assets/android/drawable/selector_fillet_btn_bg.svg");background-size:cover;background-position:center;}
.bg-5{background-image:url("/src/assets/android/drawable/selector_fillet_btn_bg.svg");background-size:cover;background-position:center;}
.bg-6{background-image:url("/src/assets/android/drawable/selector_fillet_btn_bg.svg");background-size:cover;background-position:center;}
</style>
