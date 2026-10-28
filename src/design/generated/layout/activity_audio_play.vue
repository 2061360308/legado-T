<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
<div class="box " style="width:100%;height:100%;flex-direction:column;background:#FAFAFA" data-tag="androidx.constraintlayout.widget.ConstraintLayout" data-path="$"><span class="image-placeholder " style="width:100%;height:100%;object-fit:cover" data-tag="ImageView" data-path="$/0">ImageView</span><div class="box " style="width:100%;height:100%;background:#00000050" data-tag="View" data-path="$/1"></div><div class="titlebar " style="width:100%;height:auto" data-tag="TitleBar" data-path="$/2"><span class="titlebar-title">{{ "" }}</span></div><span class="text bg-0" style="width:auto;height:auto;margin:16px;padding-left:6px;padding-right:6px;color:#FFFFFF;visibility:hidden" data-tag="TextView" data-path="$/3">{{ "" }}</span><span class="text bg-1" style="width:auto;height:auto;margin:16px;padding-left:6px;padding-right:6px;color:#FFFFFF;visibility:hidden" data-tag="TextView" data-path="$/4">{{ "" }}</span><span class="text " style="width:100%;height:auto;padding-top:6px;padding-bottom:6px;padding-left:16px;padding-right:16px;justify-content:center;align-items:center;color:#FFFFFF" data-tag="TextView" data-path="$/5">{{ "" }}</span><span class="image-placeholder " style="width:260px;height:260px;object-fit:cover" aria-label="@string/img_cover" data-tag="CircleImageView" data-path="$/6">CircleImageView</span><span class="image bg-2" style="width:46px;height:46px;margin-left:16px;padding:5px" aria-label="@string/play_mode" data-tag="ImageButton" data-path="$/7"><AndroidIcon name="ic_play_mode_list_end_stop" :size="24" /></span><div class="box " style="width:100%;height:auto;padding-left:16px;padding-right:16px;flex-direction:row;justify-content:center;align-items:center" data-tag="LinearLayout" data-path="$/8"><span class="text " style="width:auto;height:auto;color:#FFFFFF" data-tag="TextView" data-path="$/8/0">{{ "00:00" }}</span><span class="text " style="width:auto;height:auto;color:#FFFFFF" data-tag="TextView" data-path="$/8/1">{{ "00:00" }}</span><input class="seek " type="range" style="width:0px;height:25px;flex-grow:1;flex-shrink:1;flex-basis:0%;justify-content:center;align-items:center" data-tag="ThemeSeekBar" data-path="$/8/2" /></div><div class="box " style="width:100%;height:auto;padding-inline:6px;flex-direction:row;justify-content:center;align-items:center" data-tag="LinearLayout" data-path="$/9"><span class="image bg-3" style="width:46px;height:46px;padding:5px" aria-label="@string/set_timer" data-tag="ImageButton" data-path="$/9/0"><AndroidIcon name="ic_timer_black_24dp" :size="24" /></span><span class="image bg-4" style="width:46px;height:46px;padding:5px" aria-label="@string/skip_previous" data-tag="ImageButton" data-path="$/9/1"><AndroidIcon name="ic_fast_rewind" :size="24" /></span><span class="image bg-5" style="width:46px;height:46px;padding:5px" aria-label="@string/skip_previous" data-tag="ImageButton" data-path="$/9/2"><AndroidIcon name="ic_skip_previous" :size="24" /></span><span class="image bg-6" style="width:46px;height:46px;padding:5px" aria-label="@string/skip_next" data-tag="ImageButton" data-path="$/9/3"><AndroidIcon name="ic_skip_next" :size="24" /></span><span class="image bg-7" style="width:46px;height:46px;padding:5px" aria-label="@string/skip_next" data-tag="ImageButton" data-path="$/9/4"><AndroidIcon name="ic_fast_forward" :size="24" /></span><span class="image bg-8" style="width:46px;height:46px;padding:5px" aria-label="@string/chapter_list" data-tag="ImageButton" data-path="$/9/5"><AndroidIcon name="ic_chapter_list" :size="24" /></span><div class="box " style="width:0px;height:1px;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/9/6"></div><div class="box " style="width:0px;height:1px;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/9/7"></div><div class="box " style="width:0px;height:1px;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/9/8"></div><div class="box " style="width:0px;height:1px;flex-grow:1;flex-shrink:1;flex-basis:0%" data-tag="View" data-path="$/9/9"></div><div class="box " style="width:auto;height:auto" data-tag="ConstraintLayout" data-path="$/9/10"><button class="fab " style="width:auto;height:auto;margin:10px" aria-label="@string/audio_play" data-tag="FloatingActionButton" data-path="$/9/10/0"><AndroidIcon name="@drawable/ic_play_24dp" :size="22" /></button><div class="progress " style="width:0px;height:0px;visibility:hidden" data-tag="ThemeProgressBar" data-path="$/9/10/1">加载中</div></div></div></div>
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
.bg-0{background-image:url("/src/assets/android/drawable/shape_fillet_btn_press.svg");background-size:cover;background-position:center;}
.bg-1{background-image:url("/src/assets/android/drawable/shape_fillet_btn_press.svg");background-size:cover;background-position:center;}
.bg-2{background-image:url("/src/assets/android/drawable/selector_circle_btn_bg.svg");background-size:cover;background-position:center;}
.bg-3{background-image:url("/src/assets/android/drawable/selector_circle_btn_bg.svg");background-size:cover;background-position:center;}
.bg-4{background-image:url("/src/assets/android/drawable/selector_circle_btn_bg.svg");background-size:cover;background-position:center;}
.bg-5{background-image:url("/src/assets/android/drawable/selector_circle_btn_bg.svg");background-size:cover;background-position:center;}
.bg-6{background-image:url("/src/assets/android/drawable/selector_circle_btn_bg.svg");background-size:cover;background-position:center;}
.bg-7{background-image:url("/src/assets/android/drawable/selector_circle_btn_bg.svg");background-size:cover;background-position:center;}
.bg-8{background-image:url("/src/assets/android/drawable/selector_circle_btn_bg.svg");background-size:cover;background-position:center;}
</style>
