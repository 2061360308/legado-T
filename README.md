# LegadoT

LegadoT is a multi-platform reader built with Tauri 2, Vue 3, TypeScript, and Rust.

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
