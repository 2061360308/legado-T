import { defineStore } from "pinia";

export type PrefValue = string | boolean;

const STORAGE_KEY = "local-reader:reader-settings";

interface Persisted {
  values: Record<string, PrefValue>;
}

function load(): Record<string, PrefValue> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    if (parsed.values && typeof parsed.values === "object") return parsed.values;
  } catch {
    // ignore corrupt storage
  }
  return {};
}

export const useReaderSettingsStore = defineStore("readerSettings", {
  state: () => ({
    values: load() as Record<string, PrefValue>,
  }),
  actions: {
    persist() {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ values: this.values }));
    },
    set(key: string, value: PrefValue) {
      this.values[key] = value;
      this.persist();
    },
    get(key: string): PrefValue | undefined {
      return this.values[key];
    },
  },
});
