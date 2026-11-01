param(
    [ValidateSet("all", "arm64", "x86_64")]
    [string]$Target = "all",
    [string]$ApiLevel = "24",
    [switch]$Release,
    [switch]$SizeOptimized,
    [switch]$SkipFrontend,
    [switch]$SkipVerify
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$profile = if ($Release) { "release" } else { "debug" }
$variant = if ($Release) { "Release" } else { "Debug" }
$cargoProfileArgs = if ($Release) { @("--release") } else { @() }

function Fail([string]$Message) {
    throw $Message
}

function Resolve-JavaHome {
    if ($env:JAVA_HOME -and (Test-Path (Join-Path $env:JAVA_HOME "bin\java.exe"))) {
        return $env:JAVA_HOME
    }
    $jbr = "C:\Program Files\Android\Android Studio\jbr"
    if (Test-Path (Join-Path $jbr "bin\java.exe")) {
        return $jbr
    }
    Fail "Java not found. Set JAVA_HOME to a JDK 17+."
}

function Resolve-AndroidHome {
    $candidates = @($env:ANDROID_HOME, $env:ANDROID_SDK_ROOT, "$env:LOCALAPPDATA\Android\Sdk")
    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path (Join-Path $candidate "platforms"))) {
            return $candidate
        }
    }
    Fail "Android SDK not found. Set ANDROID_HOME."
}

function Resolve-NdkHome([string]$SdkHome) {
    $candidates = @($env:ANDROID_NDK_HOME, $env:NDK_HOME)
    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path (Join-Path $candidate "toolchains"))) {
            return $candidate
        }
    }
    $latest = Get-ChildItem (Join-Path $SdkHome "ndk") -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending |
        Select-Object -First 1
    if ($latest) {
        return $latest.FullName
    }
    Fail "Android NDK not found. Set ANDROID_NDK_HOME or install an NDK."
}

function Resolve-Libclang {
    $candidates = @()
    if ($env:LIBCLANG_PATH) {
        $candidates += $env:LIBCLANG_PATH
    }
    $candidates += "C:\Program Files\LLVM\bin"

    $python = Get-Command python -ErrorAction SilentlyContinue
    if ($python) {
        $pythonLibclang = & python -c "import clang, os; print(os.path.join(os.path.dirname(clang.__file__), 'native'))" 2>$null
        if ($LASTEXITCODE -eq 0 -and $pythonLibclang) {
            $candidates += $pythonLibclang.Trim()
        }
    }

    $found = $candidates |
        Where-Object { $_ -and (Test-Path (Join-Path $_ "libclang.dll")) } |
        Select-Object -First 1
    if (-not $found) {
        Fail "libclang.dll not found. Set LIBCLANG_PATH, install LLVM, or run: pip install libclang"
    }
    return $found
}

function Get-GradleVersion {
    $properties = Join-Path $root "src-tauri\gen\android\gradle\wrapper\gradle-wrapper.properties"
    if (Test-Path $properties) {
        $match = [regex]::Match((Get-Content -LiteralPath $properties -Raw), "gradle-([0-9.]+)-bin\.zip")
        if ($match.Success) {
            return $match.Groups[1].Value
        }
    }
    return "9.6.1"
}

function Find-Gradle([string]$Version) {
    $known = @(
        (Join-Path $env:TEMP "opencode\gradle-$Version\bin\gradle.bat"),
        (Join-Path $env:TEMP "local-reader-gradle-$Version\gradle-$Version\bin\gradle.bat"),
        (Join-Path $env:TEMP "gradle-$Version\gradle-$Version\bin\gradle.bat"),
        (Join-Path $env:TEMP "gradle-$Version\bin\gradle.bat")
    )
    foreach ($candidate in $known) {
        if (Test-Path $candidate) {
            return $candidate
        }
    }

    $distRoot = Join-Path $env:USERPROFILE ".gradle\wrapper\dists\gradle-$Version-bin"
    if (Test-Path $distRoot) {
        $candidate = Get-ChildItem $distRoot -Directory -ErrorAction SilentlyContinue |
            ForEach-Object { Join-Path $_.FullName "gradle-$Version\bin\gradle.bat" } |
            Where-Object { Test-Path $_ } |
            Select-Object -First 1
        if ($candidate) {
            return $candidate
        }
    }

    $gradleCommand = Get-Command gradle.bat -ErrorAction SilentlyContinue
    if ($gradleCommand) {
        return $gradleCommand.Source
    }
    return $null
}

function Install-Gradle([string]$Version) {
    $cache = Join-Path $env:TEMP "local-reader-gradle-$Version"
    $zip = "$cache.zip"
    $url = "https://services.gradle.org/distributions/gradle-$Version-bin.zip"

    if (-not (Test-Path $zip)) {
        Write-Output "Downloading Gradle $Version"
        & curl.exe --ssl-no-revoke -L --fail --retry 2 -o $zip $url
        if ($LASTEXITCODE -ne 0) {
            Fail "Gradle download failed. Download manually: $url"
        }
    }

    if (-not (Test-Path (Join-Path $cache "gradle-$Version\bin\gradle.bat"))) {
        if (Test-Path $cache) {
            Remove-Item $cache -Recurse -Force
        }
        New-Item -ItemType Directory -Force $cache | Out-Null
        Expand-Archive -LiteralPath $zip -DestinationPath $cache -Force
    }

    $gradle = Join-Path $cache "gradle-$Version\bin\gradle.bat"
    if (-not (Test-Path $gradle)) {
        Fail "Gradle extraction failed: $gradle"
    }
    return $gradle
}

function Get-TargetInfo {
    return @{
        arm64 = @{
            Rust = "aarch64-linux-android"
            Env = "aarch64_linux_android"
            Jni = "arm64-v8a"
            ApkDir = "arm64"
        }
        x86_64 = @{
            Rust = "x86_64-linux-android"
            Env = "x86_64_linux_android"
            Jni = "x86_64"
            ApkDir = "x86_64"
        }
    }
}

$javaHome = Resolve-JavaHome
$androidHome = Resolve-AndroidHome
$ndkHome = Resolve-NdkHome $androidHome
$libclang = Resolve-Libclang

$env:JAVA_HOME = $javaHome
$env:ANDROID_HOME = $androidHome
$env:ANDROID_SDK_ROOT = $androidHome
$env:ANDROID_NDK_HOME = $ndkHome
$env:ANDROID_NDK_ROOT = $ndkHome
$env:NDK_HOME = $ndkHome
$env:LIBCLANG_PATH = $libclang

$toolchain = Join-Path $ndkHome "toolchains\llvm\prebuilt\windows-x86_64"
$toolchainBin = Join-Path $toolchain "bin"
$clangVersion = Get-ChildItem (Join-Path $toolchain "lib\clang") -Directory -ErrorAction SilentlyContinue |
    Sort-Object Name -Descending |
    Select-Object -First 1
if (-not $clangVersion) {
    Fail "Clang resource directory not found under $toolchain"
}
$resourceDir = (Join-Path $toolchain "lib\clang\$($clangVersion.Name)") -replace "\\", "/"
$sysroot = (Join-Path $toolchain "sysroot") -replace "\\", "/"
$env:PATH = "$toolchainBin;$env:PATH"

$infos = Get-TargetInfo
$targetKeys = if ($Target -eq "all") { @("arm64", "x86_64") } else { @($Target) }

if ($SizeOptimized -and -not $Release) {
    Fail "-SizeOptimized requires -Release."
}
if ($SizeOptimized) {
    $env:CARGO_PROFILE_RELEASE_LTO = "true"
    $env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1"
    $env:CARGO_PROFILE_RELEASE_OPT_LEVEL = "z"
    $env:CARGO_PROFILE_RELEASE_PANIC = "abort"
    $env:CARGO_PROFILE_RELEASE_STRIP = "symbols"
}

Write-Output "Java:     $javaHome"
Write-Output "SDK:      $androidHome"
Write-Output "NDK:      $ndkHome"
Write-Output "libclang: $libclang"
Write-Output "Targets:  $($targetKeys -join ', ')"

Push-Location $root
try {
    if (-not $SkipFrontend) {
        Write-Output "Building frontend"
        & npm run build
        if ($LASTEXITCODE -ne 0) {
            Fail "Frontend build failed with exit code $LASTEXITCODE"
        }
    }

    $dist = Join-Path $root "dist"
    if (-not (Test-Path (Join-Path $dist "index.html"))) {
        Fail "dist/index.html not found. Run npm run build first."
    }

    $androidMain = Join-Path $root "src-tauri\gen\android\app\src\main"
    $assetsRoot = Join-Path $androidMain "assets"
    $jniRoot = Join-Path $androidMain "jniLibs"

    New-Item -ItemType Directory -Force $assetsRoot | Out-Null
    $config = Join-Path $assetsRoot "tauri.conf.json"
    if (-not (Test-Path $config)) {
        Copy-Item (Join-Path $root "src-tauri\tauri.conf.json") $config -Force
    }
    Get-ChildItem $assetsRoot -Force |
        Where-Object { $_.Name -ne "tauri.conf.json" } |
        Remove-Item -Recurse -Force
    Copy-Item (Join-Path $dist "*") $assetsRoot -Recurse -Force

    Remove-Item $jniRoot -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force $jniRoot | Out-Null

    $soPaths = @{}
    foreach ($key in $targetKeys) {
        $info = $infos[$key]
        $clang = Join-Path $toolchainBin "$($info.Rust)$ApiLevel-clang.cmd"
        if (-not (Test-Path $clang)) {
            Fail "Android clang wrapper not found: $clang"
        }

        $env:BINDGEN_EXTRA_CLANG_ARGS = "--target=$($info.Rust)$ApiLevel --sysroot=$sysroot -resource-dir=$resourceDir"
        Set-Item -Path "Env:CC_$($info.Env)" -Value $clang
        Set-Item -Path "Env:AR_$($info.Env)" -Value (Join-Path $toolchainBin "llvm-ar.exe")
        Set-Item -Path "Env:CARGO_TARGET_$($info.Env.ToUpper())_LINKER" -Value $clang

        Write-Output "Building Rust for $key ($($info.Rust))"
        $cargoArgs = @(
            "build",
            "--package", "local-reader",
            "--manifest-path", "src-tauri/Cargo.toml",
            "--target", $info.Rust,
            "--features", "tauri/custom-protocol",
            "--lib"
        ) + $cargoProfileArgs
        & cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) {
            Fail "Rust build failed for $key"
        }

        $so = Join-Path $root "src-tauri\target\$($info.Rust)\$profile\liblocal_reader_lib.so"
        if (-not (Test-Path $so)) {
            Fail "Rust library not found: $so"
        }
        $soPaths[$key] = $so
        $jniDir = Join-Path $jniRoot $info.Jni
        New-Item -ItemType Directory -Force $jniDir | Out-Null
        Copy-Item $so (Join-Path $jniDir "liblocal_reader_lib.so") -Force
    }

    $gradleVersion = Get-GradleVersion
    $gradle = Find-Gradle $gradleVersion
    if (-not $gradle) {
        $gradle = Install-Gradle $gradleVersion
    }
    Write-Output "Gradle:   $gradle"

    $tasks = if ($Target -eq "all") {
        @(
            "app:assembleArm64$variant",
            "app:assembleX86_64$variant",
            "app:assembleUniversal$variant"
        )
    } else {
        $taskTarget = if ($Target -eq "arm64") { "Arm64" } else { "X86_64" }
        @("app:assemble$taskTarget$variant")
    }

    $excludeTasks = @(
        ":app:rustBuildArm64$variant",
        ":app:rustBuildX86_64$variant",
        ":app:rustBuildUniversal$variant",
        ":app:rustBuildArm$variant",
        ":app:rustBuildX86$variant"
    )
    $gradleArgs = @($tasks)
    foreach ($task in $excludeTasks) {
        $gradleArgs += @("-x", $task)
    }
    $gradleArgs += @("--no-daemon")

    Push-Location (Join-Path $root "src-tauri\gen\android")
    try {
        & $gradle @gradleArgs
        if ($LASTEXITCODE -ne 0) {
            Fail "Gradle build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }

    $apkRoot = Join-Path $root "src-tauri\gen\android\app\build\outputs\apk"
    $resultKeys = if ($Target -eq "all") { @("arm64", "x86_64", "universal") } else { @($Target) }
    $apks = @()
    foreach ($key in $resultKeys) {
        $apkDir = if ($key -eq "universal") { "universal" } else { $infos[$key].ApkDir }
        $apk = Get-ChildItem (Join-Path $apkRoot "$apkDir\$profile") -Filter "*.apk" -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if (-not $apk) {
            Fail "APK not found for $key"
        }
        $apks += [pscustomobject]@{ Key = $key; File = $apk }
    }

    if (-not $SkipVerify) {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $buildTools = Get-ChildItem (Join-Path $androidHome "build-tools") -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending |
            Select-Object -First 1
        $apksigner = if ($buildTools) { Join-Path $buildTools.FullName "apksigner.bat" } else { $null }

        foreach ($item in $apks) {
            $apk = $item.File
            $zip = [System.IO.Compression.ZipFile]::OpenRead($apk.FullName)
            try {
                $libs = @($zip.Entries |
                    Where-Object { $_.FullName -eq "lib/arm64-v8a/liblocal_reader_lib.so" -or $_.FullName -eq "lib/x86_64/liblocal_reader_lib.so" } |
                    ForEach-Object { $_.FullName })
            } finally {
                $zip.Dispose()
            }

            if ($item.Key -eq "universal") {
                if ($libs.Count -ne 2) {
                    Fail "Universal APK does not contain both arm64 and x86_64 native libraries."
                }
            } elseif ($item.Key -eq "arm64") {
                if ($libs -notcontains "lib/arm64-v8a/liblocal_reader_lib.so") {
                    Fail "arm64 APK is missing arm64 native library."
                }
            } elseif ($item.Key -eq "x86_64") {
                if ($libs -notcontains "lib/x86_64/liblocal_reader_lib.so") {
                    Fail "x86_64 APK is missing x86_64 native library."
                }
            }

            if ($apk.Name -match "unsigned") {
                Write-Output "Unsigned APK, skipping signature verification: $($apk.FullName)"
            } elseif ($apksigner -and (Test-Path $apksigner)) {
                & $apksigner verify $apk.FullName
                if ($LASTEXITCODE -ne 0) {
                    Fail "APK signature verification failed: $($apk.FullName)"
                }
            }
        }
    }

    Write-Output ""
    Write-Output "Build complete:"
    foreach ($item in $apks) {
        $size = [math]::Round($item.File.Length / 1MB, 1)
        Write-Output ("{0,-10} {1} MiB  {2}" -f $item.Key, $size, $item.File.FullName)
    }
} finally {
    Pop-Location
}
