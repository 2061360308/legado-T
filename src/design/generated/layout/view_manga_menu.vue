<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:100%;flex-direction:column" data-tag="androidx.constraintlayout.widget.ConstraintLayout" data-path="$"><div class="box " style="width:100%;height:100%" aria-label="@string/content" data-tag="View" data-path="$/0"></div><div class="titlebar " style="width:100%;height:auto" data-tag="TitleBar" data-path="$/1"><span class="titlebar-title">{{ "" }}</span><div class="box " style="width:100%;height:auto" data-tag="ConstraintLayout" data-path="$/1/0"><span class="text " style="width:0px;height:auto;padding-left:10px;padding-right:10px;display:none" data-tag="TextView" data-path="$/1/0/0">{{ "" }}</span><span class="text " style="width:0px;height:auto;padding-left:10px;padding-right:10px;display:none" data-tag="TextView" data-path="$/1/0/1">{{ "" }}</span><span class="text " style="width:auto;height:0px;margin:1px;padding-left:6px;padding-right:6px;justify-content:center;align-items:center;max-width:120px" data-tag="AccentBgTextView" data-path="$/1/0/2">{{ "@string/book_source" }}</span></div></div><div class="box " style="width:100%;height:auto;flex-direction:column;background:#EEEEEE" data-tag="LinearLayout" data-path="$/2"><div class="box " style="width:100%;height:auto;margin-top:10px;margin-bottom:10px;margin-left:20px;margin-right:20px;flex-direction:row" data-tag="LinearLayout" data-path="$/2/0"><span class="text " style="width:auto;height:auto;margin-left:10px;margin-right:10px;padding-top:10px;padding-bottom:10px;background:transparent;color:#000000de;font-size:14px" data-tag="TextView" data-path="$/2/0/0">{{ "@string/previous_chapter" }}</span><span class="text " style="width:auto;height:auto;margin-left:10px;margin-right:10px;padding-top:10px;padding-bottom:10px;background:transparent;color:#000000de;font-size:14px" data-tag="TextView" data-path="$/2/0/1">{{ "@string/next_chapter" }}</span><input class="seek " type="range" style="width:0px;height:25px;flex-grow:1;flex-shrink:1;flex-basis:0%;justify-content:center;align-items:center" data-tag="ThemeSeekBar" data-path="$/2/0/2" /></div></div></div>
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
