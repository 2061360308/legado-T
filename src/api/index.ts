import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { COMMANDS, type AppVersionResult } from "./bindings";

export async function appVersion(): Promise<AppVersionResult> {
  return invoke<AppVersionResult>(COMMANDS.appVersion);
}

export function listenEvent<T>(event: string, cb: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => cb(e.payload));
}

export * from "./bindings";