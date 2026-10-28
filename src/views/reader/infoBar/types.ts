export type TipKey =
  | "none"
  | "bookName"
  | "chapterTitle"
  | "time"
  | "battery"
  | "batteryPercentage"
  | "page"
  | "totalProgress"
  | "totalProgress1"
  | "pageAndTotal"
  | "timeBattery"
  | "timeBatteryPercentage";

export interface InfoBarContext {
  bookName: string;
  chapterTitle: string;
  pageIndex: number;
  pageSize: number;
  chapterIndex: number;
  chapterSize: number;
  time: string;
  battery: number | null;
}

export interface InfoBarPadding {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface InfoBarStyle {
  headerVisible: boolean;
  footerVisible: boolean;
  headerSlots: [TipKey, TipKey, TipKey];
  footerSlots: [TipKey, TipKey, TipKey];
  headerPadding: InfoBarPadding;
  footerPadding: InfoBarPadding;
  showHeaderLine: boolean;
  showFooterLine: boolean;
  textColor: string;
  dividerColor: string;
  fontFamily: string;
}

export interface InfoBarPayload {
  style: InfoBarStyle;
  context: InfoBarContext;
}

export interface SlotRender {
  key: TipKey;
  text: string;
  battery: number | null;
  showBatteryIcon: boolean;
}
