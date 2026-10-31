param(
    [string]$Target = "x86_64",
    [string]$ApiLevel = "24",
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

if (-not $env:JAVA_HOME) {
    $jbr = "C:\Program Files\Android\Android Studio\jbr"
    if (Test-Path $jbr) {
        $env:JAVA_HOME = $jbr
    }
}

if (-not $env:ANDROID_HOME) {
    $env:ANDROID_HOME = "C:\Users\$env:USERNAME\AppData\Local\Android\Sdk"
}
if (-not $env:ANDROID_NDK_HOME) {
    $ndk = Get-ChildItem "$env:ANDROID_HOME\ndk" -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending |
        Select-Object -First 1
    if ($ndk) {
        $env:ANDROID_NDK_HOME = $ndk.FullName
    }
}
if (-not $env:ANDROID_NDK_HOME -or -not (Test-Path $env:ANDROID_NDK_HOME)) {
    throw "Android NDK not found. Set ANDROID_NDK_HOME."
}
$env:ANDROID_NDK_ROOT = $env:ANDROID_NDK_HOME

$toolchain = Join-Path $env:ANDROID_NDK_HOME "toolchains\llvm\prebuilt\windows-x86_64"
$clang = Join-Path $toolchain "bin\x86_64-linux-android$ApiLevel-clang.cmd"
if (-not (Test-Path $clang)) {
    throw "Android clang wrapper not found: $clang"
}

$libclangCandidates = @()
if ($env:LIBCLANG_PATH) {
    $libclangCandidates += $env:LIBCLANG_PATH
}
$libclangCandidates += "C:\Program Files\LLVM\bin"

$python = Get-Command python -ErrorAction SilentlyContinue
if ($python) {
    $pythonLibclang = & python -c "import clang, os; print(os.path.join(os.path.dirname(clang.__file__), 'native'))" 2>$null
    if ($LASTEXITCODE -eq 0 -and $pythonLibclang) {
        $libclangCandidates += $pythonLibclang.Trim()
    }
}

$libclangPath = $libclangCandidates |
    Where-Object { Test-Path (Join-Path $_ "libclang.dll") } |
    Select-Object -First 1
if (-not $libclangPath) {
    throw "libclang.dll not found. Install LLVM or run: pip install libclang"
}
$env:LIBCLANG_PATH = $libclangPath

$toolchainPosix = $toolchain -replace "\\", "/"
$sysroot = "$toolchainPosix/sysroot"
$clangVersion = Get-ChildItem (Join-Path $toolchain "lib\clang") -Directory |
    Sort-Object Name -Descending |
    Select-Object -First 1
if (-not $clangVersion) {
    throw "Clang resource directory not found."
}
$resourceDir = "$toolchainPosix/lib/clang/$($clangVersion.Name)"
$env:BINDGEN_EXTRA_CLANG_ARGS = "--target=x86_64-linux-android$ApiLevel --sysroot=$sysroot -resource-dir=$resourceDir"
$env:CC_x86_64_linux_android = $clang
$env:AR_x86_64_linux_android = Join-Path $toolchain "bin\llvm-ar.exe"
$env:CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER = $clang

if ($Target -ne "x86_64") {
    throw "This script currently supports only the x86_64 Android target."
}

$profile = if ($Release) { "release" } else { "debug" }
$buildArgs = @("tauri", "android", "build", "--target", $Target, "--apk", "--$profile")
$log = Join-Path $env:TEMP "local-reader-android-build.log"

$previousErrorAction = $ErrorActionPreference
Push-Location $root
try {
    $ErrorActionPreference = "Continue"
    & npx @buildArgs 2>&1 | Tee-Object -FilePath $log
    $buildExit = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $previousErrorAction
    Pop-Location
}

if ($buildExit -ne 0) {
    $output = Get-Content -LiteralPath $log -Raw
    if ($output -notmatch "symbolic link|Creation symbolic link is not allowed") {
        throw "Tauri Android build failed with exit code $buildExit"
    }

    $so = Join-Path $root "src-tauri\target\x86_64-linux-android\$profile\liblocal_reader_lib.so"
    if (-not (Test-Path $so)) {
        throw "Rust library not found: $so"
    }
    $jniDir = Join-Path $root "src-tauri\gen\android\app\src\main\jniLibs\x86_64"
    New-Item -ItemType Directory -Force -Path $jniDir | Out-Null
    Copy-Item -Force $so (Join-Path $jniDir "liblocal_reader_lib.so")

    $gradle = Join-Path $env:TEMP "gradle-9.6.1\gradle-9.6.1\bin\gradle.bat"
    if (-not (Test-Path $gradle)) {
        $gradleCommand = Get-Command gradle.bat -ErrorAction SilentlyContinue
        if (-not $gradleCommand) {
            throw "Gradle not found. Install Gradle 9.6.1 or set up gradle in PATH."
        }
        $gradle = $gradleCommand.Source
    }

    $task = if ($Release) { "assembleX86_64Release" } else { "assembleX86_64Debug" }
    $rustTask = if ($Release) { "rustBuildX86_64Release" } else { "rustBuildX86_64Debug" }

    Push-Location (Join-Path $root "src-tauri\gen\android")
    try {
        & $gradle $task "-x" $rustTask "--no-daemon"
        if ($LASTEXITCODE -ne 0) {
            throw "Gradle Android build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
}
