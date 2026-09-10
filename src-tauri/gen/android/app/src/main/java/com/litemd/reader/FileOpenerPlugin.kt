package com.litemd.reader

import android.app.Activity
import android.content.Intent
import android.content.SharedPreferences
import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.result.ActivityResult
import androidx.documentfile.provider.DocumentFile
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.net.NetworkInterface

/**
 * 安卓文件能力插件：
 * 1) 系统文件选择器打开（复制到私有目录，content:// 授权重启失效所以必须落盘）
 * 2) 接收「其他应用打开 / 分享」的 VIEW/SEND intent（冷启动与运行中），导入私有目录后由前端轮询拉取
 * 3) 内置文件夹浏览：SAF 目录树授权（持久化）→ 列举 md → 打开时连兄弟图片一起拷贝
 * 由 Rust 侧命令经 run_mobile_plugin 调用。
 */
@TauriPlugin
class FileOpenerPlugin(private val activity: Activity) : Plugin(activity) {

    companion object {
        const val PREFS = "litemd_file"
        const val KEY_TREE_URI = "tree_uri"
        val MD_EXTS = setOf("md", "markdown")
        val IMAGE_EXTS = setOf("png", "jpg", "jpeg", "gif", "webp", "svg", "bmp")
    }

    private var pendingInvoke: Invoke? = null

    /** 运行中收到分享/打开后的待拉取结果；null = 暂无 */
    @Volatile
    private var pendingIncoming: JSObject? = null

    // ---------- 系统文件选择器 ----------

    @Command
    fun openAndImport(invoke: Invoke) {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(
                Intent.EXTRA_MIME_TYPES,
                arrayOf("text/markdown", "text/x-markdown", "text/plain", "application/octet-stream"),
            )
        }
        startActivityForResult(invoke, intent, "onOpenDocumentResult")
    }

    @ActivityCallback
    fun onOpenDocumentResult(invoke: Invoke, result: ActivityResult) {
        // Tauri 回调传单个 ActivityResult（含 resultCode 与 data），签名须为 (Invoke, ActivityResult)
        val uri = if (result.resultCode == Activity.RESULT_OK) result.data?.data else null
        if (uri == null) {
            // 用户取消是正常操作：resolve 空结果让前端静默处理，不走错误提示
            invoke.resolve(JSObject().apply { put("cancelled", true) })
            return
        }
        try {
            invoke.resolve(importToPrivateDir(uri))
        } catch (e: Exception) {
            invoke.reject(e.message ?: "打开文件失败")
        }
    }

    // ---------- 接收「其他应用打开 / 分享」 ----------

    /**
     * 前端启动时与定时轮询调用：取一条待处理的外部导入。
     * 冷启动时从 activity.intent 取；运行中由 onNewIntent 异步导入后放入 pendingIncoming。
     * 返回空对象表示暂无。
     */
    @Command
    fun takeIncoming(invoke: Invoke) {
        pendingIncoming?.let {
            pendingIncoming = null
            invoke.resolve(it)
            return
        }
        val obj = extractIncoming(activity.intent) ?: run { invoke.resolve(JSObject()); return }
        // 清掉 intent 防止重复消费（如从最近任务恢复）
        activity.intent = Intent()
        try {
            invoke.resolve(importToPrivateDir(obj.uri!!, obj.suggestedName))
        } catch (e: Exception) {
            invoke.reject(e.message ?: "接收文件失败")
        }
    }

    /** 运行中收到新的打开/分享 intent：异步导入到私有目录，等前端轮询取走 */
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        if (extractIncoming(intent) == null) return
        Thread {
            val payload =
                try {
                    extractIncoming(intent)?.let { importToPrivateDir(it.uri!!, it.suggestedName) }
                } catch (e: Exception) {
                    null
                }
            if (payload != null) pendingIncoming = payload
        }.start()
    }

    private class Incoming(val uri: Uri?, val suggestedName: String?)

    /** 从 VIEW/SEND/SEND_MULTIPLE intent 提取第一个可导入的 uri 与建议文件名；
     *  仅处理带文件流的分享（md 文件分享均带 EXTRA_STREAM），纯文本不适用阅读器场景 */
    private fun extractIncoming(intent: Intent?): Incoming? {
        intent ?: return null
        return when (intent.action) {
            Intent.ACTION_VIEW -> Incoming(intent.data, null)
            Intent.ACTION_SEND -> intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM)
                ?.let { Incoming(it, subjectToName(intent)) }
            else -> null
        }
    }

    private fun subjectToName(intent: Intent): String? =
        intent.getStringExtra(Intent.EXTRA_SUBJECT)?.takeIf { it.isNotBlank() }?.let { s ->
            if (s.endsWith(".md", true) || s.endsWith(".markdown", true)) s else "$s.md"
        }

    // ---------- 内置文件夹浏览 ----------

    @Command
    fun pickFolder(invoke: Invoke) {
        startActivityForResult(invoke, Intent(Intent.ACTION_OPEN_DOCUMENT_TREE), "onPickFolderResult")
    }

    @ActivityCallback
    fun onPickFolderResult(invoke: Invoke, result: ActivityResult) {
        val treeUri = if (result.resultCode == Activity.RESULT_OK) result.data?.data else null
        if (treeUri == null) {
            // 用户取消是正常操作：前端静默刷新回原状态
            invoke.resolve(JSObject().apply { put("cancelled", true) })
            return
        }
        try {
            activity.contentResolver.takePersistableUriPermission(
                treeUri,
                Intent.FLAG_GRANT_READ_URI_PERMISSION,
            )
            prefs().edit().putString(KEY_TREE_URI, treeUri.toString()).apply()
            invoke.resolve(folderPayload())
        } catch (e: Exception) {
            invoke.reject(e.message ?: "目录授权失败")
        }
    }

    @Command
    fun listFolder(invoke: Invoke) {
        invoke.resolve(folderPayload())
    }

    @InvokeArg
    class OpenFolderFileArgs {
        var docUri: String = ""
    }

    /** 打开所选 md 并把同目录图片一起拷贝进私有目录 */
    @Command
    fun openFolderFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(OpenFolderFileArgs::class.java)
            val docUri = Uri.parse(args.docUri)
            invoke.resolve(importWithSiblingImages(docUri))
        } catch (e: Exception) {
            invoke.reject(e.message ?: "打开失败")
        }
    }

    // ---------- Wi-Fi 传书辅助 ----------

    /** 取局域网 IPv4 地址（优先 wlan / eth 网口） */
    @Command
    fun wifiIp(invoke: Invoke) {
        var ip = ""
        try {
            val pairs =
                NetworkInterface.getNetworkInterfaces()?.asSequence()
                    ?.filter { it.isUp && !it.isLoopback }
                    ?.flatMap { n ->
                        n.inetAddresses.asSequence()
                            .filterIsInstance<java.net.Inet4Address>()
                            .filter { !it.isLoopbackAddress }
                            .map { (n.displayName ?: n.name) to it.hostAddress }
                    }
                    ?.toList().orEmpty()
            ip = pairs.firstOrNull { it.first.startsWith("wlan") || it.first.startsWith("eth") }?.second
                ?: pairs.firstOrNull()?.second
                ?: ""
        } catch (_: Exception) {
        }
        invoke.resolve(JSObject().apply { put("ip", ip) })
    }

    // ---------- 私有目录导入实现 ----------

    /** 读 uri 内容写入私有目录，返回本地路径 */
    private fun importToPrivateDir(uri: Uri, suggestedName: String? = null): JSObject {
        val name = suggestedName ?: queryDisplayName(uri) ?: "imported.md"
        val target = uniqueFile(privateImportedDir(), ensureMdExt(name))
        activity.contentResolver.openInputStream(uri)?.use { input ->
            target.outputStream().use { out -> input.copyTo(out) }
        } ?: throw Exception("无法读取所选文件")
        return JSObject().apply {
            put("path", target.absolutePath)
            put("name", target.name)
        }
    }

    /** 打开文件夹中的文档并连同图片拷贝 */
    private fun importWithSiblingImages(docUri: Uri): JSObject {
        val doc = DocumentFile.fromSingleUri(activity, docUri)
            ?: throw Exception("无法访问所选文档")
        val name = doc.name ?: "document.md"
        val main = uniqueFile(privateImportedDir(), ensureMdExt(name))
        copyContent(docUri, main)

        // 同目录图片：仅当通过树授权拿到父目录才可列举；单 URI 无父上下文则跳过
        val parent = doc.parentFile
        if (parent != null && parent.isDirectory) {
            parent.listFiles().forEach { sibling ->
                val ext = sibling.name?.substringAfterLast('.', "")?.lowercase() ?: return@forEach
                if (ext in IMAGE_EXTS) {
                    val dest = File(privateImportedDir(), sibling.name!!)
                    if (!dest.exists()) copyContent(sibling.uri, dest)
                }
            }
        }
        return JSObject().apply {
            put("path", main.absolutePath)
            put("name", main.name)
        }
    }

    /** 列举已授权文件夹内容；granted=false 表示尚未授权过目录 */
    private fun folderPayload(): JSObject {
        val arr = JSArray()
        val treeStr = prefs().getString(KEY_TREE_URI, null)
        var granted = false
        if (treeStr != null) {
            DocumentFile.fromTreeUri(activity, Uri.parse(treeStr))?.let { tree ->
                granted = true
                tree.listFiles()
                    .filter { it.isFile && (it.name?.substringAfterLast('.', "")?.lowercase() ?: "") in MD_EXTS }
                    .sortedByDescending { it.lastModified() }
                    .forEach { f ->
                        val o = JSObject()
                        o.put("uri", f.uri.toString())
                        o.put("name", f.name ?: "")
                        o.put("size", f.length())
                        o.put("lastModified", f.lastModified())
                        arr.put(o)
                    }
            }
        }
        return JSObject().apply {
            put("granted", granted)
            put("list", arr)
        }
    }

    private fun copyContent(uri: Uri, dest: File) {
        activity.contentResolver.openInputStream(uri)?.use { input ->
            dest.outputStream().use { out -> input.copyTo(out) }
        } ?: throw Exception("无法读取 $dest")
    }

    // ---------- 工具 ----------

    private fun prefs(): SharedPreferences = activity.getSharedPreferences(PREFS, 0)

    private fun privateImportedDir(): File = File(activity.filesDir, "imported").apply { mkdirs() }

    private fun ensureMdExt(name: String): String {
        if (name.substringAfterLast('.', "").lowercase() in MD_EXTS) return name
        return "$name.md"
    }

    /** 从 content:// URI 取显示文件名（OpenableColumns.DISPLAY_NAME） */
    private fun queryDisplayName(uri: Uri): String? {
        val cursor =
            activity.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
        cursor?.use {
            if (it.moveToFirst()) return it.getString(0)
        }
        return uri.lastPathSegment
    }

    /** 同名文件追加序号，避免进度 key 冲突 */
    private fun uniqueFile(dir: File, name: String): File {
        var candidate = File(dir, name)
        if (!candidate.exists()) return candidate
        val dot = name.lastIndexOf('.')
        val base = if (dot > 0) name.substring(0, dot) else name
        val ext = if (dot > 0) name.substring(dot) else ""
        var i = 1
        while (candidate.exists()) {
            candidate = File(dir, "${base}_$i$ext")
            i++
        }
        return candidate
    }
}
