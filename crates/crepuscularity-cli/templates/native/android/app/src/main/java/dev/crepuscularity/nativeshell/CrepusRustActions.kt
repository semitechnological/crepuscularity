package dev.crepuscularity.nativeshell

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.jsonArray
import org.json.JSONObject

object CrepusStateStore {
    private val json = Json { ignoreUnknownKeys = true }
    private var revision by mutableLongStateOf(0L)

    fun applyResult(raw: String) {
        if (CrepusRustActions.storeResultJson(raw)) {
            revision += 1
        }
    }

    fun text(expr: String, scopeName: String? = null, scope: JsonElement? = null): String {
        revision
        return CrepusRustActions.evalText(expr, scopeName, scope?.toString())
    }

    fun bool(expr: String, scopeName: String? = null, scope: JsonElement? = null): Boolean {
        revision
        return CrepusRustActions.evalBool(expr, scopeName, scope?.toString())
    }

    fun number(expr: String, scopeName: String? = null, scope: JsonElement? = null): Float {
        revision
        return CrepusRustActions.evalNumber(expr, scopeName, scope?.toString()).toFloat()
    }

    fun items(expr: String, scopeName: String? = null, scope: JsonElement? = null): List<JsonElement> {
        revision
        val payload = runCatching { json.parseToJsonElement(CrepusRustActions.evalItemsJson(expr, scopeName, scope?.toString())) as? JsonArray }
            .getOrNull()
        return payload?.jsonArray?.toList() ?: emptyList()
    }
}

object CrepusRustActions {
    init {
        System.loadLibrary("crepus_mobile_actions")
    }

    private lateinit var appContext: Context
    private lateinit var activity: ComponentActivity
    private var pendingPickerAction: String? = null
    private var openDocuments: (() -> Unit)? = null
    private var openMedia: (() -> Unit)? = null

    external fun dispatchAction(action: String): Boolean
    external fun dispatchActionJson(action: String): String
    external fun dispatchAndStoreJson(action: String): String
    external fun lastResult(): String
    external fun storeResultJson(json: String): Boolean
    external fun evalText(expr: String, scopeName: String?, scopeJson: String?): String
    external fun evalBool(expr: String, scopeName: String?, scopeJson: String?): Boolean
    external fun evalNumber(expr: String, scopeName: String?, scopeJson: String?): Double
    external fun evalItemsJson(expr: String, scopeName: String?, scopeJson: String?): String
    external fun initAndroid(context: Context)
    external fun lastError(): String
    external fun startAutoScan(): String
    external fun shutdownAndroid()

    fun install(activity: ComponentActivity) {
        this.activity = activity
        appContext = activity.applicationContext
        initAndroid(appContext)
        val filePicker =
            activity.registerForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
                val action = pendingPickerAction ?: return@registerForActivityResult
                pendingPickerAction = null
                emit(filePickerResultJson(action, uris))
            }
        openDocuments = { filePicker.launch(arrayOf("*/*")) }
        openMedia = { filePicker.launch(arrayOf("image/*", "video/*")) }
        CrepusActions.dispatch = { action -> dispatchHostAction(action) ?: dispatchAndStoreJson(action) }
        CrepusActions.resultSink = { result -> CrepusActions.applyResult(result) }
    }

    private fun dispatchHostAction(action: String): String? {
        dispatchNamedHostAction(action)?.let { return it }
        val request = runCatching { JSONObject(action) }.getOrNull() ?: return null
        if (request.optString("kind") != "plugin") return null
        val capability = request.optString("capability")
        val method = request.optString("method")
        val actionName = "$capability.$method"
        return runCatching {
            val value = hostPluginValue(capability, method, request.optJSONObject("payload"))
            JSONObject()
                .put("ok", true)
                .put("action", actionName)
                .put(
                    "value",
                    JSONObject()
                        .put("capability", capability)
                        .put("method", method)
                        .put("value", value),
                ).toString()
        }.getOrElse { error ->
            JSONObject()
                .put("ok", false)
                .put("action", actionName)
                .put("error", error.message ?: error.toString())
                .toString()
        }
    }

    private fun hostPluginValue(capability: String, method: String, payload: JSONObject?): Any =
        when (capability) {
            "app" -> appValue(method)
            "clipboard" -> clipboardValue(method, payload)
            "device" -> deviceValue(method)
            "haptics" -> hapticsValue(method, payload)
            "preferences" -> preferencesValue(method, payload)
            "browser", "linking" -> openUrlValue(capability, method, payload)
            "share" -> shareValue(method, payload)
            else -> error("unsupported host capability: $capability")
        }

    private fun dispatchNamedHostAction(action: String): String? =
        when (action) {
            "pick_media" -> {
                pendingPickerAction = action
                openMedia?.invoke() ?: emit(errorJson(action, "media picker unavailable"))
                pendingJson(action)
            }
            "import_files" -> {
                pendingPickerAction = action
                openDocuments?.invoke() ?: emit(errorJson(action, "file picker unavailable"))
                pendingJson(action)
            }
            else -> null
        }

    private fun clipboardValue(method: String, payload: JSONObject?): JSONObject {
        val clipboard =
            appContext.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        return when (method) {
            "get" -> {
                val text = clipboard.primaryClip?.getItemAt(0)?.coerceToText(appContext)?.toString()
                JSONObject().put("text", text)
            }
            "set" -> {
                val text = payload?.optString("text", null)
                    ?: error("clipboard.set requires payload.text")
                clipboard.setPrimaryClip(ClipData.newPlainText("Crepus", text))
                JSONObject().put("text", text)
            }
            "clear" -> {
                clipboard.setPrimaryClip(ClipData.newPlainText("", ""))
                JSONObject().put("cleared", true)
            }
            else -> error("unsupported clipboard method: $method")
        }
    }

    private fun deviceValue(method: String): JSONObject {
        if (method != "info") error("unsupported device method: $method")
        return JSONObject()
            .put("targetOs", "android")
            .put("targetArch", Build.SUPPORTED_ABIS.firstOrNull() ?: "unknown")
            .put("targetFamily", "linux")
            .put("tempDir", appContext.cacheDir.absolutePath)
            .put("manufacturer", Build.MANUFACTURER)
            .put("model", Build.MODEL)
            .put("device", Build.DEVICE)
            .put("sdkInt", Build.VERSION.SDK_INT)
            .put("release", Build.VERSION.RELEASE)
    }

    private fun appValue(method: String): JSONObject {
        if (method != "info") error("unsupported app method: $method")
        val packageInfo = appContext.packageManager.getPackageInfo(appContext.packageName, 0)
        return JSONObject()
            .put("bundleId", appContext.packageName)
            .put("name", appContext.applicationInfo.loadLabel(appContext.packageManager).toString())
            .put("version", packageInfo.versionName ?: JSONObject.NULL)
            .put("build", packageInfo.longVersionCode)
    }

    private fun preferencesValue(method: String, payload: JSONObject?): Any {
        val prefs = appContext.getSharedPreferences("crepus_preferences", Context.MODE_PRIVATE)
        return when (method) {
            "get" -> {
                val key = payload?.optString("key", null) ?: error("preferences.get requires payload.key")
                prefs.all[key] ?: JSONObject.NULL
            }
            "set" -> {
                val key = payload?.optString("key", null) ?: error("preferences.set requires payload.key")
                val value = payload.opt("value") ?: error("preferences.set requires payload.value")
                val editor = prefs.edit()
                when (value) {
                    JSONObject.NULL -> editor.remove(key)
                    is Boolean -> editor.putBoolean(key, value)
                    is Int -> editor.putInt(key, value)
                    is Long -> editor.putLong(key, value)
                    is Double -> editor.putFloat(key, value.toFloat())
                    is Float -> editor.putFloat(key, value)
                    else -> editor.putString(key, value.toString())
                }
                editor.apply()
                JSONObject().put("key", key).put("value", value)
            }
            "remove" -> {
                val key = payload?.optString("key", null) ?: error("preferences.remove requires payload.key")
                val removed = prefs.contains(key)
                prefs.edit().remove(key).apply()
                JSONObject().put("key", key).put("removed", removed)
            }
            "keys" -> prefs.all.keys.sorted()
            "clear" -> {
                prefs.edit().clear().apply()
                JSONObject().put("cleared", true)
            }
            else -> error("unsupported preferences method: $method")
        }
    }

    private fun hapticsValue(method: String, payload: JSONObject?): JSONObject {
        val vibrator =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val manager = appContext.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager
                manager.defaultVibrator
            } else {
                @Suppress("DEPRECATION")
                appContext.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
            }
        val duration =
            when (method) {
                "impact" -> when (payload?.optString("style", "medium")) {
                    "light" -> 10L
                    "heavy" -> 30L
                    else -> 20L
                }
                "selection" -> 10L
                "notification" -> when (payload?.optString("type", "success")) {
                    "warning" -> 25L
                    "error" -> 35L
                    else -> 20L
                }
                else -> error("unsupported haptics method: $method")
            }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createOneShot(duration, VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(duration)
        }
        return when (method) {
            "impact" -> JSONObject().put("triggered", true).put("style", payload?.optString("style", "medium") ?: "medium")
            "selection" -> JSONObject().put("triggered", true)
            "notification" -> JSONObject().put("triggered", true).put("type", payload?.optString("type", "success") ?: "success")
            else -> error("unsupported haptics method: $method")
        }
    }

    private fun openUrlValue(capability: String, method: String, payload: JSONObject?): JSONObject {
        if (method != "open") error("unsupported $capability method: $method")
        val url = payload?.optString("url", null) ?: error("$capability.open requires payload.url")
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        activity.startActivity(intent)
        return JSONObject().put("url", url).put("opened", true)
    }

    private fun shareValue(method: String, payload: JSONObject?): JSONObject {
        if (method != "share") error("unsupported share method: $method")
        val text = payload?.optString("text", null)
        val url = payload?.optString("url", null)
        val title = payload?.optString("title", null)
        if (text == null && url == null) error("share.share requires payload.text or payload.url")
        val body = listOfNotNull(text, url).joinToString(separator = "\n").ifBlank {
            error("share.share requires payload.text or payload.url")
        }
        val intent =
            Intent(Intent.ACTION_SEND)
                .setType("text/plain")
                .putExtra(Intent.EXTRA_TEXT, body)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        if (title != null) {
            intent.putExtra(Intent.EXTRA_SUBJECT, title)
        }
        activity.startActivity(Intent.createChooser(intent, title ?: "Share"))
        return JSONObject().put("shared", true).put("text", text).put("url", url).put("title", title)
    }

    private fun emit(result: String) {
        CrepusActions.resultSink(result)
    }

    private fun pendingJson(action: String): String =
        JSONObject()
            .put("ok", true)
            .put("action", action)
            .put("pending", true)
            .toString()

    private fun errorJson(action: String, error: String): String =
        JSONObject()
            .put("ok", false)
            .put("action", action)
            .put("error", error)
            .toString()

    private fun filePickerResultJson(action: String, uris: List<Uri>): String {
        val files =
            uris.mapNotNull { uri ->
                runCatching {
                    appContext.contentResolver.openInputStream(uri)?.use { input ->
                        val bytes = input.readBytes()
                        JSONObject()
                            .put("name", queryDisplayName(uri))
                            .put("mimeType", appContext.contentResolver.getType(uri) ?: "application/octet-stream")
                            .put("bytes", bytes.size)
                            .put("dataBase64", android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP))
                    }
                }.getOrNull()
            }
        return JSONObject()
            .put("ok", true)
            .put("action", action)
            .put("value", JSONObject().put("files", files))
            .toString()
    }

    private fun queryDisplayName(uri: Uri): String {
        appContext.contentResolver.query(uri, arrayOf(android.provider.OpenableColumns.DISPLAY_NAME), null, null, null)
            ?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val index = cursor.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                    if (index >= 0) return cursor.getString(index)
                }
            }
        return uri.lastPathSegment ?: "file"
    }
}

object CrepusActionState {
    val lastResult = mutableStateOf("{}")
    val lastError = mutableStateOf<String?>(null)

    fun startAutoScan() {
        record(CrepusRustActions.startAutoScan())
    }

    fun dispatch(action: String) {
        record(CrepusActions.dispatch(action))
    }

    fun record(result: String) {
        lastResult.value = CrepusRustActions.lastResult()
        lastError.value = CrepusRustActions.lastError().ifBlank { null }
    }
}
