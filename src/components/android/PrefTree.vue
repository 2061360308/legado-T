<script setup lang="ts">
import PreferenceCategory from "./PreferenceCategory.vue";
import PreferenceRow from "./PreferenceRow.vue";
import type { PrefValue } from "@/stores/readerSettings";

export interface PrefNode {
  tag: string;
  title: string;
  summary: string;
  icon: string;
  key: string;
  defaultValue: string;
  dependency?: string;
  entries?: string[];
  entryValues?: string[];
  children: PrefNode[];
}

const props = defineProps<{
  nodes: PrefNode[];
  model: Record<string, PrefValue>;
  summaries?: Record<string, string>;
}>();

const emit = defineEmits<{
  change: [key: string, value: PrefValue];
  action: [node: PrefNode];
}>();

function rawValue(node: PrefNode): PrefValue {
  const value = props.model[node.key];
  if (value !== undefined) return value;
  return node.tag === "SwitchPreference" ? node.defaultValue === "true" : node.defaultValue ?? "";
}

function checked(node: PrefNode): boolean {
  const value = rawValue(node);
  return value === true || value === "true";
}

function valueLabel(node: PrefNode): string {
  const value = String(rawValue(node) ?? "");
  const index = node.entryValues?.indexOf(value) ?? -1;
  if (index >= 0) return node.entries?.[index] ?? value;
  return value;
}

function disabled(node: PrefNode): boolean {
  if (!node.dependency) return false;
  const value = props.model[node.dependency];
  return value === false || value === "false";
}

function onRow(node: PrefNode) {
  if (node.tag === "SwitchPreference") {
    emit("change", node.key, !checked(node));
    return;
  }
  emit("action", node);
}

function onSwitch(node: PrefNode, event: Event) {
  emit("change", node.key, (event.target as HTMLInputElement).checked);
}

function forwardChange(key: string, value: PrefValue) {
  emit("change", key, value);
}

function forwardAction(node: PrefNode) {
  emit("action", node);
}
</script>

<template>
  <template v-for="(node, index) in nodes" :key="`${node.key}-${index}`">
    <PreferenceCategory v-if="node.children.length > 0" :title="node.title">
      <PrefTree
        :nodes="node.children"
        :model="model"
        :summaries="summaries"
        @change="forwardChange"
        @action="forwardAction"
      />
    </PreferenceCategory>
    <PreferenceRow
      v-else
      :label="node.title || node.key"
      :summary="summaries?.[node.key] ?? node.summary"
      :icon="node.icon"
      :disabled="disabled(node)"
      :chevron="node.tag !== 'SwitchPreference'"
      @click="onRow(node)"
    >
      <template #right>
        <input
          v-if="node.tag === 'SwitchPreference'"
          class="pref-switch"
          type="checkbox"
          :checked="checked(node)"
          @click.stop
          @change="onSwitch(node, $event)"
        />
        <span v-else-if="node.entries?.length" class="pref-value">{{ valueLabel(node) }}</span>
      </template>
    </PreferenceRow>
  </template>
</template>

<style scoped>
.pref-switch {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.pref-value {
  font-size: 13px;
  color: var(--ac-secondary-text);
  max-width: 45%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
