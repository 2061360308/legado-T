import type { RemoteEntry, RemoteServerType } from "@/core/domain";

export interface RemoteStorage {
  readonly type: RemoteServerType;
  readonly rootPath: string;
  test(signal?: AbortSignal): Promise<void>;
  list(path: string): Promise<RemoteEntry[]>;
  get(path: string, signal?: AbortSignal): Promise<Response>;
  put(
    path: string,
    data: Blob | Uint8Array,
    contentType?: string,
    signal?: AbortSignal,
  ): Promise<void>;
  delete(path: string): Promise<void>;
  mkdir(path: string): Promise<void>;
  move(from: string, to: string): Promise<void>;
  childPath(path: string, name: string, isDir?: boolean): string;
}

export type { RemoteHttpTransport, RemoteTransportMode } from "./transport";
