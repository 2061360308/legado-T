import { invoke } from "./backend";
import { COMMANDS } from "./bindings";

export function kvGet(key: string): Promise<string | null> {
  return invoke<string | null>(COMMANDS.kvGet, { key });
}

export function kvSet(key: string, value: string): Promise<void> {
  return invoke<void>(COMMANDS.kvSet, { key, value });
}