<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:100%;flex-direction:column" data-tag="androidx.constraintlayout.widget.ConstraintLayout" data-path="$"><div class="box " style="width:100%;height:100%" aria-label="@string/content" data-tag="View" data-path="$/0"></div><button class="fab " style="width:auto;height:auto;margin:16px" aria-label="上个结果" data-tag="FloatingActionButton" data-path="$/1"><AndroidIcon name="@drawable/ic_arrow_right" :size="22" /></button><button class="fab " style="width:auto;height:auto;margin:16px" aria-label="下个结果" data-tag="FloatingActionButton" data-path="$/2"><AndroidIcon name="@drawable/ic_arrow_right" :size="22" /></button><div class="box " style="width:100%;height:auto;flex-direction:column" data-tag="LinearLayout" data-path="$/3"><div class="box " style="width:100%;height:36px;padding-left:10px;padding-right:10px;flex-direction:row;justify-content:center;align-items:center;background:#EEEEEE" data-tag="LinearLayout" data-path="$/3/0"><span class="image " style="width:36px;height:100%;background:transparent" aria-label="@string/go_to_top" data-tag="AppCompatImageView" data-path="$/3/0/0"><AndroidIcon name="ic_arrow_drop_up" :size="24" /></span><span class="image " style="width:36px;height:100%;background:transparent" aria-label="@string/go_to_bottom" data-tag="AppCompatImageView" data-path="$/3/0/1"><AndroidIcon name="ic_arrow_drop_down" :size="24" /></span><span class="text " style="width:0px;height:100%;padding-left:10px;padding-right:10px;flex-grow:1;flex-shrink:1;flex-basis:0%;justify-content:center;align-items:center;background:transparent;color:#000000de;font-size:12px;text-overflow:ellipsis;white-space:nowrap;overflow:hidden" data-tag="TextView" data-path="$/3/0/2">{{ "" }}</span></div><div class="box " style="width:100%;height:auto;flex-direction:row" data-tag="LinearLayout" data-path="$/3/1"><div class="box " style="width:0px;height:100%;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/3/1/0"></div><div class="box " style="width:0px;height:100%;flex-grow:2;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/3/1/1"></div><div class="box " style="width:0px;height:100%;flex-grow:2;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/3/1/2"></div><div class="box " style="width:0px;height:100%;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/3/1/3"></div><div class="box " style="width:60px;height:auto;padding-bottom:7px;flex-direction:column;background:transparent" aria-label="结果" data-tag="LinearLayout" data-path="$/3/1/4"><span class="image " style="width:100%;height:0px;flex-grow:1;flex-shrink:1;flex-basis:0%;max-height:20px" aria-label="结果" data-tag="AppCompatImageView" data-path="$/3/1/4/0"><AndroidIcon name="ic_toc" :size="24" /></span><span class="text " style="width:auto;height:auto;margin-top:3px;justify-content:center;align-items:center;align-items:center;color:#000000de;font-size:12px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden" data-tag="TextView" data-path="$/3/1/4/1">{{ "结果" }}</span></div><div class="box " style="width:60px;height:auto;padding-bottom:7px;flex-direction:column;background:transparent" aria-label="@string/main_menu" data-tag="LinearLayout" data-path="$/3/1/5"><span class="image " style="width:100%;height:0px;flex-grow:1;flex-shrink:1;flex-basis:0%;max-height:20px" aria-label="@string/main_menu" data-tag="AppCompatImageView" data-path="$/3/1/5/0"><AndroidIcon name="ic_menu" :size="24" /></span><span class="text " style="width:auto;height:auto;margin-top:3px;justify-content:center;align-items:center;align-items:center;color:#000000de;font-size:12px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden" data-tag="TextView" data-path="$/3/1/5/1">{{ "@string/main_menu" }}</span></div><div class="box " style="width:60px;height:auto;padding-bottom:7px;flex-direction:column;background:transparent" aria-label="退出" data-tag="LinearLayout" data-path="$/3/1/6"><span class="image " style="width:100%;height:0px;flex-grow:1;flex-shrink:1;flex-basis:0%;max-height:20px" aria-label="退出" data-tag="AppCompatImageView" data-path="$/3/1/6/0"><AndroidIcon name="ic_auto_page_stop" :size="24" /></span><span class="text " style="width:auto;height:auto;margin-top:3px;justify-content:center;align-items:center;align-items:center;color:#000000de;font-size:12px;-webkit-line-clamp:1;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden" data-tag="TextView" data-path="$/3/1/6/1">{{ "退出" }}</span></div></div></div></div>
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

</style>
