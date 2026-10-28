import type { InfoBarContext, SlotRender, TipKey } from "./types";

export function readProgress(context: InfoBarContext): string {
  const { chapterIndex, chapterSize, pageIndex, pageSize } = context;
  if (chapterSize <= 0 || (pageSize <= 0 && chapterIndex === 0)) return "0.0%";
  if (pageSize <= 0) {
    return `${(((chapterIndex + 1) / chapterSize) * 100).toFixed(1)}%`;
  }
  const ratio =
    chapterIndex / chapterSize + (1 / chapterSize) * ((pageIndex + 1) / pageSize);
  let percent = `${(ratio * 100).toFixed(1)}%`;
  if (percent === "100.0%" && (chapterIndex + 1 !== chapterSize || pageIndex + 1 !== pageSize)) {
    percent = "99.9%";
  }
  return percent;
}

function batteryText(battery: number | null): string {
  return battery === null ? "--" : String(battery);
}

export function renderSlot(key: TipKey, context: InfoBarContext): SlotRender {
  const progress = readProgress(context);
  const text = (value: string, showBatteryIcon = false, battery: number | null = null): SlotRender => ({
    key,
    text: value,
    battery,
    showBatteryIcon,
  });
  switch (key) {
    case "none":
      return text("");
    case "bookName":
      return text(context.bookName);
    case "chapterTitle":
      return text(context.chapterTitle);
    case "time":
      return text(context.time);
    case "battery":
      return text(batteryText(context.battery), true, context.battery);
    case "batteryPercentage":
      return text(`${batteryText(context.battery)}%`);
    case "page":
      return text(`${context.pageIndex + 1}/${context.pageSize}`);
    case "totalProgress":
      return text(progress);
    case "totalProgress1":
      return text(`${context.chapterIndex + 1}/${context.chapterSize}`);
    case "pageAndTotal":
      return text(`${context.pageIndex + 1}/${context.pageSize}  ${progress}`);
    case "timeBattery":
      return text(`${context.time}  ${batteryText(context.battery)}`, true, context.battery);
    case "timeBatteryPercentage":
      return text(`${context.time} ${batteryText(context.battery)}%`);
    default:
      return text("");
  }
}
