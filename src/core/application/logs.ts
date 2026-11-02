import { invoke } from "./backend";
import { COMMANDS, type AppLogEntry } from "./bindings";

export function appLogAdd(
  level: string,
  source: string,
  message: string,
): Promise<void> {
  return invoke<void>(COMMANDS.appLogAdd, { level, source, message });
}

export function appLogList(
  limit = 500,
  source?: string | null,
): Promise<AppLogEntry[]> {
  return invoke<AppLogEntry[]>(COMMANDS.appLogList, {
    limit,
    source: source ?? null,
  });
}

export function appLogClear(): Promise<void> {
  return invoke<void>(COMMANDS.appLogClear);
}
