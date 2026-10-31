export const SETTINGS_SNAPSHOT_VERSION = 1;

const SETTINGS_KEYS = [
  "local-reader:theme",
  "local-reader:appearance",
  "local-reader:reader-prefs",
  "local-reader:reader-settings",
  "local-reader:search-history",
  "bookshelf:lastGroup",
] as const;

interface SettingsSnapshot {
  version: number;
  values: Record<string, string>;
}

export function snapshotSettings(): string {
  const values: Record<string, string> = {};
  for (const key of SETTINGS_KEYS) {
    const value = localStorage.getItem(key);
    if (value !== null) values[key] = value;
  }
  const snapshot: SettingsSnapshot = {
    version: SETTINGS_SNAPSHOT_VERSION,
    values,
  };
  return JSON.stringify(snapshot);
}

export function applySettingsSnapshot(raw: string | null | undefined): boolean {
  if (!raw) return false;
  try {
    const parsed = JSON.parse(raw) as Partial<SettingsSnapshot>;
    if (!parsed.values || typeof parsed.values !== "object") return false;
    for (const key of SETTINGS_KEYS) {
      const value = parsed.values[key];
      if (typeof value === "string") localStorage.setItem(key, value);
    }
    return true;
  } catch {
    return false;
  }
}
