<script setup lang="ts">
import type { ReplaceRule } from "@/api/bindings";
import AndroidDialog from "./AndroidDialog.vue";

defineProps<{ modelValue: boolean; rules: ReplaceRule[] }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="本章生效替换"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <ul class="rules">
      <li v-for="rule in rules" :key="rule.id">
        <p class="name">
          {{ rule.name }}
          <span v-if="rule.group" class="tag">{{ rule.group }}</span>
        </p>
        <code>{{ rule.pattern }} → {{ rule.replacement || "（空）" }}</code>
        <p v-if="rule.scope || rule.excludeScope" class="scope">
          范围：{{ rule.scope || "全部" }}
          <span v-if="rule.excludeScope"> · 排除：{{ rule.excludeScope }}</span>
        </p>
      </li>
      <li v-if="rules.length === 0" class="empty">本章无生效替换</li>
    </ul>
  </AndroidDialog>
</template>

<style scoped>
.rules {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.rules li {
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  padding: 10px;
}
.name {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
}
.tag {
  margin-left: 6px;
  font-size: 11px;
  color: var(--ac-secondary-text);
  background: var(--ac-background-card);
  padding: 1px 5px;
  border-radius: 2px;
}
code {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
  word-break: break-all;
}
.scope {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.empty {
  color: var(--ac-secondary-text);
  font-size: 13px;
  text-align: center;
  border: none;
}
</style>
