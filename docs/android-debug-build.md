# Android Debug APK 从头构建

本文记录 Windows 下从零构建 `local-reader` Android Debug APK 的步骤、已知问题和当前体积原因。

## 当前产物

| 产物 | 路径 | 体积 |
|---|---|---:|
| arm64 真机 | `src-tauri/gen/android/app/build/outputs/apk/arm64/debug/app-arm64-debug.apk` | 约 388 MiB |
| x86_64 模拟器 | `src-tauri/gen/android/app/build/outputs/apk/x86_64/debug/app-x86_64-debug.apk` | 约 392 MiB |
| 通用包 | `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk` | 约 771 MiB |

体积来自 Rust 原生库，不是前端资源：

- arm64 `.so`：约 379.2 MiB
- x86_64 `.so`：约 383.5 MiB
- 前端资源 + 图标 + classes：约 9 MiB
- universal = 两份 `.so` + 共享资源

Debug APK 保留完整 Rust 调试符号，且 `.so` 在 APK 中按未压缩方式存储，因此单 ABI 包接近 390 MiB。需要发布体积时应使用 release 构建并按 ABI 拆分，不要直接用 debug APK 分发。

## Release 体积实测

使用 `-Release -SizeOptimized` 构建，arm64 实测：

| 产物 | 体积 |
|---|---:|
| arm64 Release APK | **23.5 MiB** |
| x86_64 Release APK | **26.4 MiB** |
| universal Release APK | **47.2 MiB** |
| arm64 Release `.so` | **20.25 MiB** |
| arm64 Debug `.so` | 397.6 MB |
| arm64 Debug APK | 387.8 MiB |

`-SizeOptimized` 会为 Cargo release 设置：

```text
CARGO_PROFILE_RELEASE_LTO=true
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
CARGO_PROFILE_RELEASE_OPT_LEVEL=z
CARGO_PROFILE_RELEASE_PANIC=abort
CARGO_PROFILE_RELEASE_STRIP=symbols
```

命令：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-android.ps1 `
  -Target all -Release -SizeOptimized
```

Release APK 默认是 `*-release-unsigned.apk`，脚本会跳过签名校验。安装或分发前必须配置签名。

体积从约 388 MiB 降到约 23 MiB，主要来自移除调试符号、LTO 和尺寸优化，不是业务代码被删除。该配置优先体积，尚未在真机/模拟器上完整验证 PDF、EPUB、JS 规则等运行时行为。

## 推荐：一条命令构建

```powershell
cd C:\Users\20613\Desktop\legado\local-reader
powershell -ExecutionPolicy Bypass -File .\scripts\build-android.ps1 -Target all
```

只构建单 ABI：

```powershell
.\scripts\build-android.ps1 -Target arm64
.\scripts\build-android.ps1 -Target x86_64
```

构建体积优先的 Release：

```powershell
.\scripts\build-android.ps1 -Target all -Release -SizeOptimized
```

脚本会自动：

1. 解析 Android SDK、NDK、JDK、libclang。
2. 构建前端 `dist`。
3. 编译 arm64/x86_64 Rust 原生库。
4. 同步 `dist` 到 Android assets，复制 `.so` 到 `jniLibs`。
5. 查找或下载 Gradle 9.6.1。
6. 排除会再次调用 Tauri CLI 的 `rustBuild*` 任务后运行 Gradle。
7. 校验 APK 中的 `.so` 和签名，并打印产物路径。

## 必需环境

当前机器示例：

```powershell
$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
$env:ANDROID_HOME = "C:\Users\20613\AppData\Local\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Users\20613\AppData\Local\Android\Sdk\ndk\27.0.12077973"
$env:LIBCLANG_PATH = "C:\Users\20613\.pyenv\pyenv-win\versions\3.13.13\Lib\site-packages\clang\native"
```

要求：

- Android SDK，含 `compileSdk 37`、`build-tools`。
- Android NDK，当前使用 `27.0.12077973`。
- JDK 17+；当前使用 Android Studio 自带 JBR。
- Rust Android target：`aarch64-linux-android`、`x86_64-linux-android`。
- `libclang.dll`：LLVM、Android Studio LLVM 或 Python `libclang` 包都可以。
- Node 依赖已安装：`npm install`。
- Gradle 9.6.1；脚本会查找本地缓存，找不到时用 `curl --ssl-no-revoke` 下载到临时目录。

## 手动构建流程

### 1. 构建前端

```powershell
cd C:\Users\20613\Desktop\legado\local-reader
npm run build
```

生成 `dist/`。Android 需要的是 Tauri 模式构建，不是 `build:web`。

### 2. 准备 Android 编译变量

```powershell
$ndk = "$env:ANDROID_HOME\ndk\27.0.12077973"
$toolchain = "$ndk\toolchains\llvm\prebuilt\windows-x86_64"
$env:ANDROID_NDK_ROOT = $ndk
$env:ANDROID_NDK_HOME = $ndk
$env:LIBCLANG_PATH = "C:\Users\20613\.pyenv\pyenv-win\versions\3.13.13\Lib\site-packages\clang\native"
$env:PATH = "$toolchain\bin;$env:PATH"
```

### 3. 编译 arm64 Rust 库

```powershell
$env:CC_aarch64_linux_android = "$toolchain\bin\aarch64-linux-android24-clang.cmd"
$env:AR_aarch64_linux_android = "$toolchain\bin\llvm-ar.exe"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "$toolchain\bin\aarch64-linux-android24-clang.cmd"

cargo build --package local-reader --manifest-path src-tauri/Cargo.toml `
  --target aarch64-linux-android `
  --features "tauri/custom-protocol" --lib
```

### 4. 编译 x86_64 Rust 库

```powershell
$env:CC_x86_64_linux_android = "$toolchain\bin\x86_64-linux-android24-clang.cmd"
$env:AR_x86_64_linux_android = "$toolchain\bin\llvm-ar.exe"
$env:CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER = "$toolchain\bin\x86_64-linux-android24-clang.cmd"

cargo build --package local-reader --manifest-path src-tauri/Cargo.toml `
  --target x86_64-linux-android `
  --features "tauri/custom-protocol" --lib
```

如果不设置 `CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER`，会报：

```text
error: linker `cc` not found
```

### 5. 同步前端和原生库

```powershell
$android = "src-tauri\gen\android\app\src\main"
$assets = "$android\assets"
$jni = "$android\jniLibs"

# 保留 tauri.conf.json，其余旧资源删掉，避免旧 hash 文件堆积
Get-ChildItem $assets -Force |
  Where-Object { $_.Name -ne "tauri.conf.json" } |
  Remove-Item -Recurse -Force

Copy-Item "dist\*" $assets -Recurse -Force

Remove-Item $jni -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$jni\arm64-v8a", "$jni\x86_64" | Out-Null

Copy-Item "src-tauri\target\aarch64-linux-android\debug\liblocal_reader_lib.so" `
  "$jni\arm64-v8a\liblocal_reader_lib.so" -Force
Copy-Item "src-tauri\target\x86_64-linux-android\debug\liblocal_reader_lib.so" `
  "$jni\x86_64\liblocal_reader_lib.so" -Force
```

### 6. 运行 Gradle

Gradle 必须排除 `rustBuild*` 任务。否则 Gradle 会重新调用 Tauri CLI，并报 `.tauri/cli-options-server.json` 不存在。

```powershell
$gradle = "C:\Users\20613\AppData\Local\Temp\opencode\gradle-9.6.1\bin\gradle.bat"

cd src-tauri\gen\android
& $gradle `
  app:assembleArm64Debug `
  app:assembleX86_64Debug `
  app:assembleUniversalDebug `
  --no-daemon `
  -x :app:rustBuildArm64Debug `
  -x :app:rustBuildX86_64Debug `
  -x :app:rustBuildUniversalDebug `
  -x :app:rustBuildArmDebug `
  -x :app:rustBuildX86Debug
```

注意任务名是 `rustBuildX86_64Debug`，不是 `rustBuildX8664Debug`。

## 已踩过的问题

### 1. 找不到 libclang

错误：

```text
Unable to find libclang: couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']
```

解决：设置 `LIBCLANG_PATH` 到包含 `libclang.dll` 的目录。

### 2. x86_64 找不到 linker

错误：

```text
error: linker `cc` not found
```

解决：设置 `CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER`，并同时设置 `CC_x86_64_linux_android`。

### 3. Tauri CLI 创建符号链接失败

错误：

```text
Creation symbolic link is not allowed for this system.
For Windows 10 or newer: You should use developer mode.
```

原因：Windows 未开启开发者模式，普通用户不能创建符号链接。

解决：

- 首选脚本的手动复制方案：自己编译 `.so` 并复制到 `jniLibs`。
- 或开启 Windows 开发者模式后重试 Tauri CLI。

### 4. Gradle wrapper 下载证书失败

错误：

```text
PKIX path building failed
```

或 curl：

```text
CRYPT_E_NO_REVOCATION_CHECK
```

解决：用 curl 关闭吊销检查下载 Gradle，再直接调用解压后的 `gradle.bat`：

```powershell
curl.exe --ssl-no-revoke -L --fail `
  -o "$env:TEMP\gradle-9.6.1-bin.zip" `
  https://services.gradle.org/distributions/gradle-9.6.1-bin.zip

Expand-Archive "$env:TEMP\gradle-9.6.1-bin.zip" `
  -DestinationPath "$env:TEMP\gradle-9.6.1" -Force
```

### 5. Gradle 调用 Tauri CLI 失败

错误：

```text
failed to read ...\gen\android\.tauri\cli-options-server.json;
the `tauri android dev` or `tauri android build` command must be running
```

解决：已经手动编译并复制 `.so` 后，排除 `:app:rustBuild*Debug` 任务。

### 6. APK 缺少 native 库

Gradle 可能提示：

```text
There are no .so files available to package in the APK for armeabi-v7a, x86.
```

这是正常的，因为只提供了 arm64-v8a 和 x86_64。只要目标 ABI 的 APK 中存在：

```text
lib/arm64-v8a/liblocal_reader_lib.so
lib/x86_64/liblocal_reader_lib.so
```

即可。不要构建或分发没有 `.so` 的 arm/x86 空包。

## 校验产物

检查 APK 内原生库：

```powershell
tar -tf app-universal-debug.apk | Select-String liblocal_reader_lib
```

检查签名：

```powershell
& "$env:ANDROID_HOME\build-tools\36.0.0\apksigner.bat" verify --print-certs `
  app-universal-debug.apk
```

预期包含：

```text
lib/arm64-v8a/liblocal_reader_lib.so
lib/x86_64/liblocal_reader_lib.so
```

## 为什么 universal 是 771 MiB

`app-universal-debug.apk` 同时打包：

- arm64 `.so` ≈ 379.2 MiB
- x86_64 `.so` ≈ 383.5 MiB
- 前端资源和 Java/Kotlin classes ≈ 9 MiB

合计约 771 MiB。`app-arm64-debug.apk` 只有一份 arm64 `.so`，所以约 388 MiB；x86_64 的 `.so` 稍大，所以约 392 MiB。

Debug 配置保留了 native 调试符号（`src-tauri/gen/android/app/build.gradle.kts` 中的 `jniLibs.keepDebugSymbols`），且 APK 中的 `.so` 未压缩。因此 debug APK 很大是预期行为，不代表构建异常。

如果要减小体积：

- 使用 release 构建。
- 使用 `strip` 移除调试符号。
- 按 ABI 拆分，不要发 universal 包。
- 不要把 debug APK 当正式分发包。

## 清理与重建

以下目录可以安全删除后重建：

- `src-tauri/target`
- `src-tauri/gen/android/app/build`
- `src-tauri/gen/android/app/src/main/assets/assets`
- `src-tauri/gen/android/app/src/main/jniLibs`
- `dist`

不要删除：

- `src-tauri/gen/android/app/src/main/assets/tauri.conf.json`
- `src-tauri/gen/android` 下的 Gradle/Kotlin/Manifest 工程源码
- `src-tauri/gen/android/gradle/wrapper`

重新构建后，Rust `target` 可能增长到约 29 GB；确认 APK 可用后可按需再次清理。
