# Execution instructions

## Scope

- Repository root is the LegadoT project.
- Do not assume files outside this repository are available.
- Do not commit generated build outputs, wasm outputs, APKs, installers, or signing material.

## Common commands

```powershell
npm run dev:web
npm run build:web
npm run dev:pc
npm run build:pc
npm run android:debug
npm run android:release
```

Use `npm run doctor` to inspect Node, Rust, wasm, Android, and WASI tooling.

## Android

- Prefer `npm run android:debug` and `npm run android:release`.
- Local Windows builds use `scripts/build-android.ps1 -Target split`.
- CI builds Android on Ubuntu with Tauri CLI.
- `key.properties` and keystores must never be committed.

## Release

- `VERSION` is the release version source.
- The release tag is `release-<VERSION>`.
- Keep only `.github/workflows/legadot-release.yml` for release automation.
