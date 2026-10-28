<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-pref">
<div class="pref-row" data-depth="0">
  
  <div class="pref-text">
    <span class="pref-title">{{ "" }}</span>
    
    
    
    
  </div>
  
</div>
  </div>
</template>

<style scoped>
.generated-pref { width:100%; min-height:100%; background:var(--ac-background); color:var(--ac-primary-text); }
.pref-group { display:flex; flex-direction:column; }
.pref-group h2 { margin:0; padding:10px 20px 6px; font-size:13px; color:var(--ac-primary); }
.pref-row { display:flex; align-items:center; gap:12px; min-height:52px; padding:10px 20px; border-bottom:1px solid var(--ac-divider); }
.pref-text { flex:1; min-width:0; display:flex; flex-direction:column; gap:3px; }
.pref-title { font-size:15px; }
.pref-summary,.pref-key,.pref-default,.pref-entries { font-size:12px; color:var(--ac-secondary-text); }
</style>
