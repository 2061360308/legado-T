// 由 PNG 生成单尺寸 ICO（内嵌 PNG，Vista+ 支持）。用于 tauri-build 的 Windows 资源。
const fs = require("fs");
const path = require("path");

const iconsDir = path.join(__dirname, "..", "src-tauri", "icons");
const png = fs.readFileSync(path.join(iconsDir, "icon.png"));

const header = Buffer.alloc(22);
header.writeUInt16LE(0, 0); // reserved
header.writeUInt16LE(1, 2); // type = icon
header.writeUInt16LE(1, 4); // count
header.writeUInt8(0, 6); // width (0 => 256+)
header.writeUInt8(0, 7); // height
header.writeUInt8(0, 8); // colors
header.writeUInt8(0, 9); // reserved
header.writeUInt16LE(1, 10); // planes
header.writeUInt16LE(32, 12); // bit count
header.writeUInt32LE(png.length, 14); // bytes in resource
header.writeUInt32LE(22, 18); // image offset

const ico = Buffer.concat([header, png]);
fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);
console.log("icon.ico bytes:", ico.length);