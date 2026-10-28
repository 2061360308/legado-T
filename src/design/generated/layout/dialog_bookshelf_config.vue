<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:auto;padding:16px" data-tag="androidx.constraintlayout.widget.ConstraintLayout" data-path="$"><div class="box " style="width:100%;height:auto;flex-direction:row" data-tag="LinearLayout" data-path="$/0"><span class="text " style="width:0px;height:auto;padding:6px;flex-grow:1;flex-shrink:1;flex-basis:0%;color:#000000de" data-tag="TextView" data-path="$/0/0">{{ "@string/group_style" }}</span><div class="custom " style="width:auto;height:auto" data-tag="AppCompatSpinner" data-path="$/0/1" data-custom="true">AppCompatSpinner</div></div><div class="box " style="width:0px;height:auto;flex-direction:column" data-tag="LinearLayout" data-path="$/1"><span class="text " style="width:100%;height:100%;padding:6px;font-size:16px" data-tag="AccentTextView" data-path="$/1/0">{{ "@string/view" }}</span><div class="custom " style="width:100%;height:auto;flex-direction:column" data-tag="RadioGroup" data-path="$/1/1" data-custom="true">RadioGroup<label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/1/1/0"><input type="checkbox" /><span>{{ "@string/layout_list" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/1/1/1"><input type="checkbox" /><span>{{ "@string/layout_grid3" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/1/1/2"><input type="checkbox" /><span>{{ "@string/layout_grid4" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/1/1/3"><input type="checkbox" /><span>{{ "@string/layout_grid5" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/1/1/4"><input type="checkbox" /><span>{{ "@string/layout_grid6" }}</span></label></div></div><div class="box " style="width:0px;height:auto;flex-direction:column" data-tag="LinearLayout" data-path="$/2"><span class="text " style="width:100%;height:100%;padding:6px;font-size:16px" data-tag="AccentTextView" data-path="$/2/0">{{ "@string/sort" }}</span><div class="custom " style="width:100%;height:auto;flex-direction:column" data-tag="RadioGroup" data-path="$/2/1" data-custom="true">RadioGroup<label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/2/1/0"><input type="checkbox" /><span>{{ "@string/bookshelf_px_0" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/2/1/1"><input type="checkbox" /><span>{{ "@string/bookshelf_px_1" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/2/1/2"><input type="checkbox" /><span>{{ "@string/bookshelf_px_2" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/2/1/3"><input type="checkbox" /><span>{{ "@string/bookshelf_px_3" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/2/1/4"><input type="checkbox" /><span>{{ "@string/bookshelf_px_4" }}</span></label><label class="switch-row " style="width:100%;height:auto" data-tag="RadioButton" data-path="$/2/1/5"><input type="checkbox" /><span>{{ "@string/bookshelf_px_5" }}</span></label></div></div><label class="switch-row " style="width:100%;height:auto;padding:6px" data-tag="ThemeSwitch" data-path="$/3"><input type="checkbox" /><span>{{ "@string/show_unread" }}</span></label><label class="switch-row " style="width:100%;height:auto;padding:6px" data-tag="ThemeSwitch" data-path="$/4"><input type="checkbox" /><span>{{ "@string/show_last_update_time" }}</span></label><label class="switch-row " style="width:100%;height:auto;padding:6px" data-tag="ThemeSwitch" data-path="$/5"><input type="checkbox" /><span>{{ "@string/show_wait_up_count" }}</span></label><label class="switch-row " style="width:100%;height:auto;padding:6px" data-tag="ThemeSwitch" data-path="$/6"><input type="checkbox" /><span>{{ "@string/show_bookshelf_fast_scroller" }}</span></label></div>
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
