import type { RemoteEntry, RemoteServerType } from "@/core/domain";

export interface RemoteStorage {
  readonly type: RemoteServerType;
  readonly rootPath: string;
  test(): Promise<void>;
  list(path: string): Promise<RemoteEntry[]>;
  get(path: string): Promise<Response>;
  put(path: string, data: Blob | Uint8Array, contentType?: string): Promise<void>;
  delete(path: string): Promise<void>;
  childPath(path: string, name: string): string;
}

export type { RemoteHttpTransport, RemoteTransportMode } from "./transport";
