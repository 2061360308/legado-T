import { platform } from "@/runtime";
import { COMMANDS, type AppVersionResult, type Unlisten } from "./bindings";
import { invoke } from "./backend";

export async function appVersion(): Promise<AppVersionResult> {
  return invoke<AppVersionResult>(COMMANDS.appVersion);
}

export function listenEvent<T>(event: string, cb: (payload: T) => void): Promise<Unlisten> {
  return platform.events.listen<T>(event, cb);
}

export * from "./bindings";
