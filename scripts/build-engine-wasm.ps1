param(
    [string]$WasiSdk = $env:WASI_SDK,
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$srcTauri = Join-Path $root "src-tauri"

if (-not $WasiSdk) {
    throw "WASI_SDK is not set. Pass -WasiSdk <path-to-wasi-sdk> or set the WASI_SDK environment variable."
}

$env:WASI_SDK = $WasiSdk
$env:LIBCLANG_PATH = Join-Path $WasiSdk "bin"
$env:PATH = "$env:LIBCLANG_PATH;$env:PATH"

Push-Location $srcTauri
try {
    $profile = if ($Release) { "release" } else { "debug" }
    if ($Release) {
        cargo build -p engine-js-wasm --target wasm32-wasip1 --release
    } else {
        cargo build -p engine-js-wasm --target wasm32-wasip1
    }
    if ($LASTEXITCODE -ne 0) {
        throw "engine-js-wasm wasm build failed"
    }
    $artifact = Join-Path $srcTauri "target/wasm32-wasip1/$profile/engine_js_wasm.wasm"
    $target = Join-Path $root "src/adapters/web/wasm/engine/engine_js_wasm.wasm"
    Copy-Item -LiteralPath $artifact -Destination $target -Force
    Write-Host "Wrote $target"
} finally {
    Pop-Location
}
