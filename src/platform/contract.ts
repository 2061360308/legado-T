export interface SafeAreaInsets {
  left: number;
  top: number;
  right: number;
  bottom: number;
  cutoutLeft: number;
  cutoutTop: number;
  cutoutRight: number;
  cutoutBottom: number;
}

export interface PlatformInfo {
  os: string;
  apiLevel: number;
  abi: string;
  screenWidth: number;
  screenHeight: number;
  density: number;
  safeArea: SafeAreaInsets;
}

export interface SafDirectoryLocation {
  cancelled: boolean;
  treeUri: string;
  documentId: string;
  rootName: string;
}

export interface SafEntry {
  documentId: string;
  name: string;
  isDir: boolean;
  size: number;
  modified: number;
  ext: string;
  supported: boolean;
}

export interface SafPrepareFile {
  name: string;
  path: string;
  size: number;
  relativePath: string;
}

export interface SafPrepareResult {
  sessionId: string;
  files: SafPrepareFile[];
  skipped: number;
}

export interface PickedImage {
  cancelled: boolean;
  path: string;
  name: string;
}

export interface PickedFont {
  cancelled: boolean;
  path: string;
  name: string;
}

export type Unlisten = () => void;

export interface PlatformCapabilities {
  android: boolean;
  safDirectory: boolean;
  systemBars: boolean;
  volumeKeys: boolean;
  backgroundAudio: boolean;
  webview: boolean;
  opfs: boolean;
  sqliteWasm: boolean;
  webdavProxy: boolean;
}

export interface BackendPort {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

export interface EventPort {
  listen<T>(event: string, cb: (payload: T) => void): Promise<Unlisten>;
}

export interface AssetPort {
  fileSrc(path: string): string;
}

export interface HttpPort {
  fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

export interface FileSystemPort {
  readFile(path: string): Promise<Uint8Array<ArrayBuffer>>;
  writeFile(path: string, data: Uint8Array<ArrayBuffer>): Promise<void>;
  writeStream?(path: string, stream: ReadableStream<Uint8Array>): Promise<void>;
  readDir(path: string): Promise<{ name: string; isFile: boolean; isDirectory: boolean }[]>;
  exists(path: string): Promise<boolean>;
  mkdir(path: string, options?: { recursive?: boolean }): Promise<void>;
  join(...parts: string[]): Promise<string>;
  appDataDir(): Promise<string>;
}

export interface DialogOpenOptions {
  multiple?: boolean;
  directory?: boolean;
  title?: string;
  filters?: { name: string; extensions: string[] }[];
}

export interface DialogSaveOptions {
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}

export interface DialogPort {
  open(options?: DialogOpenOptions): Promise<string | string[] | null>;
  save(options?: DialogSaveOptions): Promise<string | null>;
}

export interface ImportLocation {
  kind: "desktop" | "saf" | "fsa" | "web";
  token: string;
  rootToken: string;
  name: string;
}

export interface ImportEntry {
  key: string;
  name: string;
  isDir: boolean;
  size: number;
  modified: number;
  ext: string;
  supported: boolean;
  onShelf: boolean;
  bookUrl?: string | null;
}

export interface PreparedImport {
  paths: string[];
  sessionId?: string;
  skipped: number;
}

export interface ImportPickerCapabilities {
  scan: boolean;
  delete: boolean;
  directoryNavigation: boolean;
  autoPick: boolean;
}

export interface ImportPickerPort {
  pickLocation(): Promise<ImportLocation | null>;
  loadSavedLocation(): Promise<ImportLocation | null>;
  saveLocation(location: ImportLocation): Promise<void>;
  listDirectory(location: ImportLocation, token: string): Promise<ImportEntry[]>;
  prepareImport(location: ImportLocation, entries: ImportEntry[]): Promise<PreparedImport>;
  cleanupPreparedImport(prepared: PreparedImport): Promise<void>;
  capabilities(location: ImportLocation | null): ImportPickerCapabilities;
  scanDirectory(location: ImportLocation, token: string): Promise<ImportEntry[]>;
  deleteEntries(location: ImportLocation, entries: ImportEntry[]): Promise<number>;
}

export interface Platform {
  capabilities: PlatformCapabilities;
  backend: BackendPort;
  events: EventPort;
  assets: AssetPort;
  http: HttpPort;
  files: FileSystemPort;
  dialog: DialogPort;
  importPicker: ImportPickerPort;
  isAndroid(): boolean;
  getPlatformInfo(): PlatformInfo | null;
  refreshPlatform(): Promise<PlatformInfo>;
  refreshSafeArea(): Promise<SafeAreaInsets>;
  setKeepScreenOn(enabled: boolean, timeoutSeconds?: number | null): Promise<void>;
  setScreenOrientation(
    mode: "system" | "portrait" | "landscape" | "sensor" | "reversePortrait",
  ): Promise<void>;
  setCutoutMode(mode: "never" | "shortEdges"): Promise<void>;
  setSystemBars(options: {
    hideStatusBar: boolean;
    hideNavigationBar: boolean;
    lightStatusBar?: boolean;
    lightNavigationBar?: boolean;
  }): Promise<void>;
  setSystemBarStyle(options: {
    transparentStatusBar: boolean;
    transparentNavigationBar: boolean;
    statusBarColor?: string;
    navigationBarColor?: string;
  }): Promise<void>;
  setTextZoom(percent: number): Promise<void>;
  setLauncherIcon(icon: string): Promise<void>;
  pickThemeImage(): Promise<PickedImage>;
  pickFontFile(): Promise<PickedFont>;
  setReaderKeyMode(options: {
    active: boolean;
    volumeKeyEnabled: boolean;
    keyPageOnLongPress: boolean;
    disableReturnKey: boolean;
  }): Promise<void>;
  startBackgroundAudio(title: string, playing?: boolean): Promise<void>;
  stopBackgroundAudio(): Promise<void>;
  pickSafDirectory(): Promise<SafDirectoryLocation>;
  listSafDirectory(treeUri: string, documentId: string): Promise<SafEntry[]>;
  scanSafDirectory(treeUri: string, documentId: string): Promise<SafEntry[]>;
  deleteSafDocuments(treeUri: string, documentIds: string[]): Promise<number>;
  prepareSafImport(treeUri: string, documentIds: string[]): Promise<SafPrepareResult>;
  cleanupSafImport(sessionId: string): Promise<void>;
  closeWindow(): void;
}
