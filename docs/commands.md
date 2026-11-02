# 常用命令

所有命令在仓库根目录执行。

## 主命令

| 场景 | 命令 | 说明 |
|---|---|---|
| 前端开发 | `npm run dev:web` | 同时启动 Hono 和 Vite Web |
| 前端构建 | `npm run build:web` | 同步版本、构建 wasm、构建 Web 前端 |
| PC 开发 | `npm run dev:pc` | 启动 Tauri 桌面开发 |
| PC 构建 | `npm run build:pc` | Windows 生成免安装 exe；Linux 生成 AppImage + deb |
| Android 调试 | `npm run android:debug` | arm64-v8a + x86_64 Debug APK |
| Android 发布 | `npm run android:release` | arm64-v8a + x86_64 Release APK |

## Web

| 命令 | 说明 |
|---|---|
| `npm run dev:web:ui` | 只启动 Vite Web，后端需自行启动 |
| `npm run preview:web` | 同时启动 Hono 和 Vite Preview |
| `npm run preview:web:ui` | 只预览 Web 静态包 |
| `npm run build:web:prebuilt` | 使用现有 wasm 构建 Web |
| `npm run build:wasm` | 构建全部 wasm |
| `npm run check:wasm` | 校验 engine wasm 导出 |
| `npm run deploy:web` | 构建 Web 并部署到 Cloudflare |

## PC

| 命令 | 说明 |
|---|---|
| `npm run build:pc` | Windows 输出 `src-tauri/target/release/local-reader.exe`，并带 `defaultData`、`pdfium`；不生成安装器或 zip |
| `npm run tauri:build` | 兼容旧命令，直接调用 `tauri build` |

说明：本地 `build:pc` 不生成 zip；GitHub Release 的 Windows 资产仍使用 zip，因为需要同时携带 `defaultData` 和 `pdfium`。

## Android

| 命令 | 说明 |
|---|---|
| `npm run android:dev` | Tauri Android 热重载开发 |
| `npm run android:debug` | Debug APK，`-Target split` |
| `npm run android:release` | Release APK，`-Target split -Release -SizeOptimized` |
| `npm run android:ui` | 重新生成 Android UI 资源 |

需要 universal APK 时手动执行：

```powershell
.\scripts\build-android.ps1 -Target all -Release -SizeOptimized
```

## 服务端

| 命令 | 说明 |
|---|---|
| `npm run server:dev` | Hono watch 开发 |
| `npm run server:start` | Hono 非 watch 启动 |
| `npm run server:check` | Hono 类型检查 |
| `npm run server:deploy` | 部署 Hono Worker |

## 检查与维护

| 命令 | 说明 |
|---|---|
| `npm run typecheck` | 前端类型检查 |
| `npm run check` | 前端 + 服务端 + Rust 检查 |
| `npm run test:rust` | 运行 `local-reader` Rust 测试 |
| `npm run doctor` | 检查 Node、Rust、wasm、Android、WASI 环境 |
| `npm run version:apply` | 将根目录 `VERSION` 同步到各构建配置 |
| `npm run clean:dist` | 删除 `dist` 和 `dist-ssr` |

## 版本发布

1. 修改根目录 `VERSION`
2. 推送到 `main`
3. GitHub Actions `LegadoT Release` 自动构建并发布 `release-<VERSION>`

发布资产：

```text
LegadoT-<VERSION>-android-arm64-v8a.apk
LegadoT-<VERSION>-android-x86_64.apk
LegadoT-<VERSION>-windows-x64.zip
LegadoT-<VERSION>-linux-x64.AppImage
LegadoT-<VERSION>-linux-x64.deb
LegadoT-wasm.zip
LegadoT-wasm.zip.sha256
SHA256SUMS.txt
```
