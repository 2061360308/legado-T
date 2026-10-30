package com.localreader.platform

import android.app.Activity
import android.content.ComponentName
import android.content.Intent
import android.content.pm.ActivityInfo
import android.content.pm.PackageManager
import android.content.res.Configuration
import android.graphics.Color
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.provider.DocumentsContract
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.view.Window
import android.view.WindowManager
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.util.UUID

@InvokeArg
class KeepScreenOnArgs {
    var enabled: Boolean = false
    var timeoutSeconds: Long? = null
}

@InvokeArg
class OrientationArgs {
    var mode: String = "system"
}

@InvokeArg
class CutoutArgs {
    var mode: String = "never"
}

@InvokeArg
class SystemBarsArgs {
    var hideStatusBar: Boolean = false
    var hideNavigationBar: Boolean = false
    var lightStatusBar: Boolean = false
    var lightNavigationBar: Boolean = false
}

@InvokeArg
class SystemBarStyleArgs {
    var transparentStatusBar: Boolean = true
    var transparentNavigationBar: Boolean = true
    var statusBarColor: String = ""
    var navigationBarColor: String = ""
}

@InvokeArg
class TextZoomArgs {
    var percent: Int = 100
}

@InvokeArg
class LauncherIconArgs {
    var icon: String = "default"
}

@InvokeArg
class ReaderKeyModeArgs {
    var active: Boolean = false
    var volumeKeyEnabled: Boolean = false
    var keyPageOnLongPress: Boolean = false
    var disableReturnKey: Boolean = false
}

@InvokeArg
class SafCleanupArgs {
    var sessionId: String = ""
}

@InvokeArg
class SafListArgs {
    var treeUri: String = ""
    var documentId: String = ""
}

@InvokeArg
class SafPrepareArgs {
    var treeUri: String = ""
    var documentIds: Array<String> = emptyArray()
}

@TauriPlugin
class ReaderPlatformPlugin(private val activity: Activity) : Plugin(activity) {
    private val handler = Handler(Looper.getMainLooper())
    private val clearKeepScreenOn = Runnable {
        activity.window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    }
    private val supportedExtensions = setOf(
        "txt", "epub", "pdf", "umd", "mobi", "azw3", "azw"
    )
    private var readerActive = false
    private var volumeKeyEnabled = false
    private var keyPageOnLongPress = false
    private var disableReturnKey = false
    private var webView: WebView? = null
    private var originalWindowCallback: Window.Callback? = null

    override fun load(webView: WebView) {
        this.webView = webView
        installKeyInterceptor()
    }

    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)
        refreshWebViewLayout()
    }

    private fun refreshWebViewLayout() {
        handler.postDelayed({
            activity.window.decorView.requestLayout()
            activity.window.decorView.invalidate()
            findWebView(activity.window.decorView)?.let { webView ->
                webView.requestLayout()
                webView.invalidate()
            }
        }, 120)
    }

    private fun findWebView(view: View): WebView? {
        if (view is WebView) return view
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) {
                findWebView(view.getChildAt(index))?.let { return it }
            }
        }
        return null
    }

    @Command
    fun setReaderKeyMode(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ReaderKeyModeArgs::class.java)
            readerActive = args.active
            volumeKeyEnabled = args.volumeKeyEnabled
            keyPageOnLongPress = args.keyPageOnLongPress
            disableReturnKey = args.disableReturnKey
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setReaderKeyMode failed")
        }
    }

    private fun installKeyInterceptor() {
        if (originalWindowCallback != null) return
        val callback = activity.window.callback ?: return
        originalWindowCallback = callback
        activity.window.callback = object : Window.Callback by callback {
            override fun dispatchKeyEvent(event: KeyEvent): Boolean {
                if (handleReaderKey(event)) return true
                return callback.dispatchKeyEvent(event)
            }

        }
    }

    private fun handleReaderKey(event: KeyEvent): Boolean {
        if (!readerActive) return false
        if (event.keyCode == KeyEvent.KEYCODE_BACK) {
            if (!disableReturnKey) return false
            if (event.action == KeyEvent.ACTION_UP) {
                emitReaderEvent("reader-back", "back", 0)
            }
            return true
        }
        val key = when (event.keyCode) {
            KeyEvent.KEYCODE_VOLUME_UP -> "volumeUp"
            KeyEvent.KEYCODE_VOLUME_DOWN -> "volumeDown"
            KeyEvent.KEYCODE_PAGE_UP -> "pageUp"
            KeyEvent.KEYCODE_PAGE_DOWN -> "pageDown"
            KeyEvent.KEYCODE_SPACE -> "space"
            else -> null
        }
        if (key == null) return false
        if (key.startsWith("volume") && !volumeKeyEnabled) return false
        if (event.action == KeyEvent.ACTION_DOWN) {
            if (event.repeatCount == 0 || keyPageOnLongPress) {
                emitReaderEvent("reader-key", key, event.repeatCount)
            }
            return true
        }
        if (event.action == KeyEvent.ACTION_UP) return true
        return false
    }

    private fun emitReaderEvent(eventName: String, key: String, repeat: Int) {
        val payload = JSObject()
        payload.put("key", key)
        payload.put("repeat", repeat)
        val script = "window.dispatchEvent(new CustomEvent('" + eventName + "',{detail:" + payload.toString() + "}))"
        webView?.post { webView?.evaluateJavascript(script, null) }
    }

    @Command
    fun platformInfo(invoke: Invoke) {
        try {
            val metrics = activity.resources.displayMetrics
            val result = JSObject()
            result.put("os", "android")
            result.put("apiLevel", Build.VERSION.SDK_INT)
            result.put("abi", Build.SUPPORTED_ABIS.firstOrNull() ?: "")
            result.put("screenWidth", metrics.widthPixels)
            result.put("screenHeight", metrics.heightPixels)
            result.put("density", metrics.density.toDouble())
            result.put("safeArea", safeAreaObject())
            invoke.resolve(result)
        } catch (error: Exception) {
            invoke.reject(error.message ?: "platformInfo failed")
        }
    }

    @Command
    fun getSafeAreaInsets(invoke: Invoke) {
        try {
            invoke.resolve(safeAreaObject())
        } catch (error: Exception) {
            invoke.reject(error.message ?: "getSafeAreaInsets failed")
        }
    }

    @Command
    fun setKeepScreenOn(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(KeepScreenOnArgs::class.java)
            onUiThread {
                handler.removeCallbacks(clearKeepScreenOn)
                if (!args.enabled) {
                    activity.window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
                    return@onUiThread
                }
                activity.window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
                val timeout = args.timeoutSeconds ?: -1L
                if (timeout > 0) {
                    handler.postDelayed(clearKeepScreenOn, timeout * 1000L)
                }
            }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setKeepScreenOn failed")
        }
    }

    @Command
    fun setScreenOrientation(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(OrientationArgs::class.java)
            onUiThread {
                activity.requestedOrientation = when (args.mode) {
                    "portrait" -> ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
                    "landscape" -> ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
                    "sensor" -> ActivityInfo.SCREEN_ORIENTATION_SENSOR
                    "reversePortrait" -> ActivityInfo.SCREEN_ORIENTATION_REVERSE_PORTRAIT
                    else -> ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
                }
            }
            refreshWebViewLayout()
            handler.postDelayed({ refreshWebViewLayout() }, 350)
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setScreenOrientation failed")
        }
    }

    @Command
    fun setCutoutMode(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CutoutArgs::class.java)
            onUiThread {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                    val attributes = activity.window.attributes
                    attributes.layoutInDisplayCutoutMode =
                        if (args.mode == "shortEdges") {
                            WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
                        } else {
                            WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_NEVER
                        }
                    activity.window.attributes = attributes
                }
            }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setCutoutMode failed")
        }
    }

    @Command
    fun setSystemBars(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SystemBarsArgs::class.java)
            onUiThread {
                val controller = WindowCompat.getInsetsController(
                    activity.window,
                    activity.window.decorView
                )
                controller.isAppearanceLightStatusBars = args.lightStatusBar
                controller.isAppearanceLightNavigationBars = args.lightNavigationBar
                if (args.hideStatusBar) {
                    controller.hide(WindowInsetsCompat.Type.statusBars())
                } else {
                    controller.show(WindowInsetsCompat.Type.statusBars())
                }
                if (args.hideNavigationBar) {
                    controller.hide(WindowInsetsCompat.Type.navigationBars())
                } else {
                    controller.show(WindowInsetsCompat.Type.navigationBars())
                }
            }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setSystemBars failed")
        }
    }

    @Command
    fun setSystemBarStyle(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SystemBarStyleArgs::class.java)
            onUiThread {
                val window = activity.window
                window.statusBarColor = if (args.transparentStatusBar) {
                    Color.TRANSPARENT
                } else {
                    parseColor(args.statusBarColor, Color.BLACK)
                }
                window.navigationBarColor = if (args.transparentNavigationBar) {
                    Color.TRANSPARENT
                } else {
                    parseColor(args.navigationBarColor, Color.BLACK)
                }
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    window.isStatusBarContrastEnforced = false
                    window.isNavigationBarContrastEnforced = false
                }
            }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setSystemBarStyle failed")
        }
    }

    @Command
    fun setTextZoom(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(TextZoomArgs::class.java)
            onUiThread {
                webView?.settings?.textZoom = args.percent.coerceIn(50, 200)
            }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setTextZoom failed")
        }
    }

    @Command
    fun setLauncherIcon(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(LauncherIconArgs::class.java)
            val icon = args.icon.ifBlank { "default" }
            val defaultAlias = ComponentName(
                activity.packageName,
                "${activity.packageName}.LauncherDefault"
            )
            val aliases = (1..6).map {
                ComponentName(activity.packageName, "${activity.packageName}.Launcher$it")
            }
            val allAliases = listOf(defaultAlias) + aliases
            val selected = if (icon == "default") {
                defaultAlias
            } else {
                aliases.firstOrNull {
                    it.className.substringAfterLast('.').equals(icon, ignoreCase = true)
                }
            }
            if (selected == null) {
                invoke.reject("未知图标")
                return
            }
            onUiThread {
                val packageManager = activity.packageManager
                allAliases.forEach {
                    packageManager.setComponentEnabledSetting(
                        it,
                        if (it == selected) {
                            PackageManager.COMPONENT_ENABLED_STATE_ENABLED
                        } else {
                            PackageManager.COMPONENT_ENABLED_STATE_DISABLED
                        },
                        PackageManager.DONT_KILL_APP
                    )
                }
                invoke.resolve()
            }
        } catch (error: Exception) {
            invoke.reject(error.message ?: "setLauncherIcon failed")
        }
    }

    @Command
    fun pickImage(invoke: Invoke) {
        try {
            val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = "image/*"
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            startActivityForResult(invoke, intent, "pickImageResult")
        } catch (error: Exception) {
            invoke.reject(error.message ?: "pickImage failed")
        }
    }

    @ActivityCallback
    fun pickImageResult(invoke: Invoke, result: ActivityResult) {
        val uri = result.data?.data
        if (result.resultCode != Activity.RESULT_OK || uri == null) {
            val payload = JSObject()
            payload.put("cancelled", true)
            payload.put("path", "")
            payload.put("name", "")
            invoke.resolve(payload)
            return
        }
        Thread {
            try {
                val name = queryDisplayName(uri) ?: "background.jpg"
                val ext = name.substringAfterLast('.', "jpg").ifBlank { "jpg" }
                val dir = File(activity.filesDir, "theme").apply { mkdirs() }
                val file = File(dir, "background_${System.currentTimeMillis()}.$ext")
                activity.contentResolver.openInputStream(uri)?.use { input ->
                    file.outputStream().use { output -> input.copyTo(output) }
                } ?: throw IllegalStateException("无法读取图片")
                val payload = JSObject()
                payload.put("cancelled", false)
                payload.put("path", file.absolutePath)
                payload.put("name", name)
                onUiThread { invoke.resolve(payload) }
            } catch (error: Exception) {
                onUiThread { invoke.reject(error.message ?: "读取图片失败") }
            }
        }.start()
    }

    @Command
    fun pickBooksDirectory(invoke: Invoke) {
        try {
            val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
                addFlags(Intent.FLAG_GRANT_PREFIX_URI_PERMISSION)
            }
            startActivityForResult(invoke, intent, "pickBooksDirectoryResult")
        } catch (error: Exception) {
            invoke.reject(error.message ?: "pickBooksDirectory failed")
        }
    }

    @ActivityCallback
    fun pickBooksDirectoryResult(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode != Activity.RESULT_OK) {
            val cancelled = JSObject()
            cancelled.put("cancelled", true)
            cancelled.put("treeUri", "")
            cancelled.put("documentId", "")
            cancelled.put("rootName", "")
            invoke.resolve(cancelled)
            return
        }
        val treeUri = result.data?.data
        if (treeUri == null) {
            invoke.reject("未选择目录")
            return
        }
        try {
            val flags = result.data?.flags ?: 0
            val readFlags = flags and Intent.FLAG_GRANT_READ_URI_PERMISSION
            if (readFlags != 0) {
                activity.contentResolver.takePersistableUriPermission(treeUri, readFlags)
            }
        } catch (_: SecurityException) {
            // The tree can still be read for this session even without persistence.
        }
        try {
            val rootId = DocumentsContract.getTreeDocumentId(treeUri)
            val rootName = queryDisplayName(
                DocumentsContract.buildDocumentUriUsingTree(treeUri, rootId)
            ) ?: "已选择目录"
            val payload = JSObject()
            payload.put("cancelled", false)
            payload.put("treeUri", treeUri.toString())
            payload.put("documentId", rootId)
            payload.put("rootName", rootName)
            invoke.resolve(payload)
        } catch (error: Exception) {
            invoke.reject(error.message ?: "读取 SAF 目录失败")
        }
    }

    @Command
    fun listBooksDirectory(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SafListArgs::class.java)
            val treeUri = Uri.parse(args.treeUri)
            val entries = JSArray()
            val projection = arrayOf(
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_MIME_TYPE,
                DocumentsContract.Document.COLUMN_SIZE,
                DocumentsContract.Document.COLUMN_LAST_MODIFIED
            )
            val childrenUri =
                DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, args.documentId)
            activity.contentResolver.query(
                childrenUri, projection, null, null, null
            )?.use { cursor ->
                while (cursor.moveToNext()) {
                    val documentId = cursor.getString(0) ?: continue
                    val name = cursor.getString(1) ?: continue
                    if (name.startsWith(".")) continue
                    val mime = cursor.getString(2) ?: ""
                    val isDir = mime == DocumentsContract.Document.MIME_TYPE_DIR
                    val ext = if (isDir) {
                        ""
                    } else {
                        name.substringAfterLast('.', "").lowercase()
                    }
                    if (!isDir && ext !in supportedExtensions) continue
                    val entry = JSObject()
                    entry.put("documentId", documentId)
                    entry.put("name", name)
                    entry.put("isDir", isDir)
                    entry.put("size", if (isDir || cursor.isNull(3)) 0L else cursor.getLong(3))
                    entry.put("modified", if (cursor.isNull(4)) 0L else cursor.getLong(4) / 1000L)
                    entry.put("ext", ext)
                    entry.put("supported", !isDir)
                    entries.put(entry)
                }
            }
            invoke.resolveObject(entries)
        } catch (error: Exception) {
            invoke.reject(error.message ?: "listBooksDirectory failed")
        }
    }

    @Command
    fun prepareBooksImport(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SafPrepareArgs::class.java)
            val treeUri = Uri.parse(args.treeUri)
            Thread {
                try {
                    val payload = copySelectedToCache(treeUri, args.documentIds)
                    onUiThread { invoke.resolve(payload) }
                } catch (error: Exception) {
                    onUiThread { invoke.reject(error.message ?: "准备导入文件失败") }
                }
            }.start()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "prepareBooksImport failed")
        }
    }

    @Command
    fun cleanupSafImport(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SafCleanupArgs::class.java)
            val safeId = args.sessionId.replace(Regex("[^a-zA-Z0-9-]"), "")
            if (safeId.isNotEmpty()) {
                val root = File(activity.cacheDir, "saf-import").canonicalFile
                val target = File(root, safeId).canonicalFile
                if (target.path.startsWith(root.path + File.separator)) {
                    target.deleteRecursively()
                }
            }
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject(error.message ?: "cleanupSafImport failed")
        }
    }

    private fun copySelectedToCache(treeUri: Uri, documentIds: Array<String>): JSObject {
        val sessionId = UUID.randomUUID().toString()
        val destDir = File(activity.cacheDir, "saf-import/$sessionId")
        if (!destDir.exists() && !destDir.mkdirs()) {
            throw IllegalStateException("无法创建导入缓存目录")
        }
        val files = JSArray()
        var skipped = 0
        for (documentId in documentIds) {
            val documentUri =
                DocumentsContract.buildDocumentUriUsingTree(treeUri, documentId)
            val meta = queryDocumentMeta(documentUri)
            if (meta == null) {
                skipped++
                continue
            }
            val ext = meta.name.substringAfterLast('.', "").lowercase()
            if (ext !in supportedExtensions) {
                skipped++
                continue
            }
            try {
                val dest = uniqueFile(destDir, meta.name)
                activity.contentResolver.openInputStream(documentUri)?.use { input ->
                    dest.outputStream().use { output -> input.copyTo(output) }
                } ?: throw IllegalStateException("无法读取 ${meta.name}")
                val entry = JSObject()
                entry.put("name", meta.name)
                entry.put("path", dest.absolutePath)
                entry.put("size", dest.length().coerceAtLeast(meta.size))
                entry.put("relativePath", meta.name)
                files.put(entry)
            } catch (_: Exception) {
                skipped++
            }
        }
        val result = JSObject()
        result.put("sessionId", sessionId)
        result.put("files", files)
        result.put("skipped", skipped)
        return result
    }

    private data class DocumentMeta(val name: String, val size: Long)

    private fun queryDocumentMeta(uri: Uri): DocumentMeta? {
        val projection = arrayOf(
            DocumentsContract.Document.COLUMN_DISPLAY_NAME,
            DocumentsContract.Document.COLUMN_SIZE
        )
        return try {
            activity.contentResolver.query(uri, projection, null, null, null)?.use {
                if (!it.moveToFirst()) return null
                val name = it.getString(0) ?: return null
                val size = if (it.isNull(1)) 0L else it.getLong(1)
                DocumentMeta(name, size)
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun parseColor(value: String, fallback: Int): Int {
        return try {
            if (value.isBlank()) fallback else Color.parseColor(value)
        } catch (_: Exception) {
            fallback
        }
    }

    private fun queryDisplayName(uri: Uri): String? {
        val projection = arrayOf(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
        return try {
            activity.contentResolver.query(uri, projection, null, null, null)?.use {
                if (it.moveToFirst()) it.getString(0) else null
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun uniqueFile(dir: File, name: String): File {
        val safeName = name.replace(Regex("[\\\\/:*?\"<>|]"), "_").ifBlank { "book.txt" }
        val dot = safeName.lastIndexOf('.')
        val base = if (dot > 0) safeName.substring(0, dot) else safeName
        val ext = if (dot > 0) safeName.substring(dot) else ""
        var candidate = File(dir, safeName)
        var index = 1
        while (candidate.exists()) {
            candidate = File(dir, "$base ($index)$ext")
            index++
        }
        return candidate
    }

    private fun safeAreaObject(): JSObject {
        val result = JSObject()
        val insets = ViewCompat.getRootWindowInsets(activity.window.decorView)
        if (insets == null) {
            result.put("left", 0)
            result.put("top", 0)
            result.put("right", 0)
            result.put("bottom", 0)
            result.put("cutoutLeft", 0)
            result.put("cutoutTop", 0)
            result.put("cutoutRight", 0)
            result.put("cutoutBottom", 0)
            return result
        }
        val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
        val cutout = insets.getInsets(WindowInsetsCompat.Type.displayCutout())
        result.put("left", maxOf(bars.left, cutout.left))
        result.put("top", maxOf(bars.top, cutout.top))
        result.put("right", maxOf(bars.right, cutout.right))
        result.put("bottom", maxOf(bars.bottom, cutout.bottom))
        result.put("cutoutLeft", cutout.left)
        result.put("cutoutTop", cutout.top)
        result.put("cutoutRight", cutout.right)
        result.put("cutoutBottom", cutout.bottom)
        return result
    }

    private fun onUiThread(block: () -> Unit) {
        if (Looper.myLooper() == Looper.getMainLooper()) {
            block()
        } else {
            activity.runOnUiThread(block)
        }
    }
}
