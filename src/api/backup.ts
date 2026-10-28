import { invoke } from "@tauri-apps/api/core";
import { COMMANDS } from "./bindings";

export interface BackupReport {
  path: string;
  files: number;
  items: number;
}

export interface RestoreReport {
  files: number;
  items: number;
  skipped: string[];
}

export function backupExport(path: string, password: string | null): Promise<BackupReport> {
  return invoke<BackupReport>(COMMANDS.backupExport, { path, password });
}

export function backupImport(
  path: string,
  password: string | null,
  ignore: string[],
): Promise<RestoreReport> {
  return invoke<RestoreReport>(COMMANDS.backupImport, { path, password, ignore });
}
