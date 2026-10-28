<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:100%;padding:10px;flex-direction:column;background:#FAFAFA" data-tag="LinearLayout" data-path="$"><div class="box " style="width:100%;height:auto;padding:6px;justify-content:center;align-items:center" data-tag="LinearLayout" data-path="$/0"><span class="text " style="width:auto;height:auto;color:#000000de;font-size:16px" data-tag="TextView" data-path="$/0/0">{{ "@string/style_name" }}</span><span class="text " style="width:auto;height:auto;padding-left:6px;padding-right:6px;color:#0000008a" data-tag="TextView" data-path="$/0/1">{{ "" }}</span><span class="text " style="width:auto;height:auto;color:#000000de" data-tag="TextView" data-path="$/0/2">{{ "@string/restore" }}</span><span class="image " style="width:20px;height:20px" aria-label="@string/edit" data-tag="ImageView" data-path="$/0/3"><AndroidIcon name="ic_edit" :size="24" /></span><span class="space " style="width:0px;height:auto;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="Space" data-path="$/0/4"></span></div><div class="box " style="width:100%;height:auto;padding:6px;flex-direction:row;justify-content:center;align-items:center" data-tag="LinearLayout" data-path="$/1"><span class="text " style="width:0px;height:auto;margin-left:6px;margin-right:6px;padding:6px;flex-grow:5;flex-shrink:1;flex-basis:0%;justify-content:center;align-items:center;background:transparent;font-size:14px" data-tag="StrokeTextView" data-path="$/1/0">{{ "@string/text_color" }}</span><span class="text " style="width:0px;height:auto;margin-left:6px;margin-right:6px;padding:6px;flex-grow:5;flex-shrink:1;flex-basis:0%;justify-content:center;align-items:center;background:transparent;font-size:14px" data-tag="StrokeTextView" data-path="$/1/1">{{ "@string/bg_color" }}</span><span class="image " style="width:36px;height:36px;margin-left:6px;margin-right:6px;background:transparent" aria-label="@string/import_str" data-tag="ImageView" data-path="$/1/2"><AndroidIcon name="ic_import" :size="24" /></span><span class="image " style="width:36px;height:36px;margin-left:6px;margin-right:6px;background:transparent" aria-label="@string/import_str" data-tag="ImageView" data-path="$/1/3"><AndroidIcon name="ic_export" :size="24" /></span><span class="image " style="width:36px;height:36px;margin-left:6px;margin-right:6px;background:transparent" aria-label="@string/delete" data-tag="ImageView" data-path="$/1/4"><AndroidIcon name="ic_clear_all" :size="24" /></span></div><label class="switch-row " style="width:100%;height:auto;padding:6px" data-tag="ThemeSwitch" data-path="$/2"><input type="checkbox" /><span>{{ "@string/dark_status_icon" }}</span></label><label class="switch-row " style="width:100%;height:auto;padding:6px" data-tag="ThemeSwitch" data-path="$/3"><input type="checkbox" /><span>{{ "@string/text_underline" }}</span></label><span class="text " style="width:100%;height:auto;padding-left:6px;padding-right:6px" data-tag="TextView" data-path="$/4">{{ "@string/bg_alpha" }}</span><span class="text " style="width:100%;height:auto;padding-left:6px;padding-right:6px" data-tag="TextView" data-path="$/5">{{ "@string/bg_image" }}</span><input class="seek " type="range" style="width:100%;height:auto;padding:6px" data-tag="ThemeSeekBar" data-path="$/6" /><div class="list " style="width:100%;height:100px;padding:6px;flex-direction:row" data-tag="RecyclerView" data-path="$/7"><div class="list-row">列表项</div><div class="list-row">列表项</div><div class="list-row">列表项</div></div></div>
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
