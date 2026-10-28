<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:100%;padding:16px" data-tag="FrameLayout" data-path="$"><div class="box bg-0" style="width:100%;height:auto;padding:16px;flex-direction:column;justify-content:center;align-items:center" data-tag="LinearLayout" data-path="$/0"><div class="box " style="width:100%;height:auto;padding-bottom:10px;flex-direction:row;justify-content:center;align-items:center" data-tag="LinearLayout" data-path="$/0/0"><span class="text " style="width:auto;height:auto;font-size:18px;font-weight:600" data-tag="TextView" data-path="$/0/0/0">{{ "@string/url_option" }}</span><span class="text " style="width:auto;height:auto;padding:12px;background:transparent" data-tag="TextView" data-path="$/0/0/1">{{ "@string/ok" }}</span><span class="space " style="width:0px;height:1px;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="Space" data-path="$/0/0/2"></span></div><div class="box " style="width:100%;height:auto" data-tag="NestedScrollView" data-path="$/0/1"><div class="box " style="width:100%;height:auto;flex-direction:column" data-tag="LinearLayout" data-path="$/0/1/0"><label class="switch-row " style="width:auto;height:auto" data-tag="ThemeCheckBox" data-path="$/0/1/0/0"><input type="checkbox" /><span>{{ "useWebView" }}</span></label><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/1"><input class="edit " placeholder="method" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/1/0" /></div><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/2"><input class="edit " placeholder="charset" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/2/0" /></div><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/3"><input class="edit " placeholder="headers" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/3/0" /></div><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/4"><input class="edit " placeholder="body" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/4/0" /></div><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/5"><input class="edit " placeholder="type" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/5/0" /></div><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/6"><input class="edit " placeholder="retry" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/6/0" /></div><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/7"><input class="edit " placeholder="webJs" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/7/0" /></div><div class="input-wrap " style="width:100%;height:auto" data-tag="TextInputLayout" data-path="$/0/1/0/8"><input class="edit " placeholder="js" style="width:100%;height:auto" data-tag="AutoCompleteTextView" data-path="$/0/1/0/8/0" /></div></div></div></div></div>
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
.bg-0{background-image:url("/src/assets/android/drawable/shape_card_view.svg");background-size:cover;background-position:center;}
</style>
