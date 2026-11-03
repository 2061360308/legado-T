# LegadoT

LegadoT is a multi-platform reader built with Tauri 2, Vue 3, TypeScript, and Rust.

## 下载

| 平台 | 下载方式 |
|---|---|
| Web | [一键部署到 Cloudflare](https://deploy.workers.cloudflare.com/?url=https://github.com/2061360308/legado-T) |
| Windows | [GitHub Release](https://github.com/2061360308/legado-T/releases/latest) |
| Android | [GitHub Release](https://github.com/2061360308/legado-T/releases/latest) |
| Linux | [GitHub Release](https://github.com/2061360308/legado-T/releases/latest) |

[![Deploy to Cloudflare](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/2061360308/legado-T)

## Web 版使用

点击上方“一键部署到 Cloudflare”，登录 Cloudflare 账号并按提示授权 GitHub。Cloudflare 会自动克隆项目、构建并部署到你自己的 Cloudflare 账号。

部署完成后，Cloudflare 会提供一个 `*.workers.dev` 地址，打开即可使用。也可以后续绑定自己的域名。

注意：

- 这是部署到你自己的 Cloudflare 账号，不是公共在线站点。
- 部署需要 Cloudflare 账号和 GitHub 账号。
- Web 数据保存在浏览器本地，清除浏览器数据前请先备份。

## Development

```powershell
npm ci
npm run dev:web
npm run dev:pc
```

## Build

```powershell
npm run build:web
npm run build:pc
npm run android:debug
npm run android:release
```

- Web build requires `WASI_SDK` for the book-source engine wasm.
- Windows PC build produces a portable executable with `defaultData` and `pdfium`; it does not create an installer or zip.
- Linux PC build produces AppImage and deb packages.
- Android builds target `arm64-v8a` and `x86_64`.

## Release

1. Update `VERSION`.
2. Push to `main`.
3. The `LegadoT Release` workflow builds Android, Windows, Linux, and wasm artifacts.
4. The workflow creates or updates `release-<VERSION>`.

See `docs/commands.md` for the full command list.

## Layout

```text
src/             Vue frontend
src-tauri/       Tauri and Rust backend
server/          Hono server for Web and Cloudflare
scripts/         Build, release, and environment scripts
docs/            Design and build documentation
```

## License

See `LICENSE` and `NOTICE.md`.
