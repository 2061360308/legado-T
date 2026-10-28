<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:auto;flex-direction:column" data-tag="LinearLayout" data-path="$"><div class="titlebar " style="width:100%;height:auto;background:#EEEEEE;box-shadow:0 5px 5px rgba(0,0,0,0.18)" data-tag="Toolbar" data-path="$/0"><span class="titlebar-title">{{ "@string/group_edit" }}</span></div><div class="box " style="width:100%;height:auto;background:#EEEEEE" data-tag="ConstraintLayout" data-path="$/1"><span class="image " style="width:90px;height:126px;margin:6px;object-fit:cover" aria-label="@string/img_cover" data-tag="CoverImageView" data-path="$/1/0"><AndroidIcon name="image_cover_default" :size="24" /></span><div class="input-wrap " style="width:0px;height:auto;margin:6px" data-tag="TextInputLayout" data-path="$/1/1"><input class="edit " placeholder="" style="width:100%;height:auto;-webkit-line-clamp:2;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden" data-tag="ThemeEditText" data-path="$/1/1/0" /></div><span class="text " style="width:auto;height:auto;padding:3px;color:#000000de" data-tag="AccentTextView" data-path="$/1/2">{{ "@string/sort" }}</span><div class="custom " style="width:auto;height:auto;margin-left:6px" data-tag="AppCompatSpinner" data-path="$/1/3" data-custom="true">AppCompatSpinner</div><label class="switch-row " style="width:auto;height:auto" data-tag="ThemeCheckBox" data-path="$/1/4"><input type="checkbox" checked /><span>{{ "@string/allow_drop_down_refresh" }}</span></label></div><div class="box " style="width:100%;height:auto;flex-direction:row;justify-content:center;align-items:center" data-tag="LinearLayout" data-path="$/2"><span class="text " style="width:auto;height:auto;margin:12px;padding:6px" data-tag="AccentTextView" data-path="$/2/0">{{ "@string/delete" }}</span><span class="text " style="width:auto;height:auto;margin:12px;padding:6px" data-tag="AccentTextView" data-path="$/2/1">{{ "@string/cancel" }}</span><span class="text " style="width:auto;height:auto;margin:12px;padding:6px" data-tag="AccentTextView" data-path="$/2/2">{{ "@string/ok" }}</span><span class="space " style="width:0px;height:1px;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="Space" data-path="$/2/3"></span></div></div>
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
