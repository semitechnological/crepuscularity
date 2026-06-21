package dev.crepuscularity.nativeshell

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.net.Uri
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.mutableStateOf
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.json.JSONObject

object CrepusRustActions {
    init {
        System.loadLibrary("crepus_mobile_actions")
    }

    private lateinit var appContext: Context
    private var pendingPickerAction: String? = null
    private var openDocuments: (() -> Unit)? = null
    private var openMedia: (() -> Unit)? = null

    external fun dispatchAction(action: String): Boolean
    external fun dispatchActionJson(action: String): String

    fun install(activity: ComponentActivity) {
        appContext = activity.applicationContext
        val filePicker =
            activity.registerForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
                val action = pendingPickerAction ?: return@registerForActivityResult
                pendingPickerAction = null
                emit(filePickerResultJson(action, uris))
            }
        openDocuments = { filePicker.launch(arrayOf("*/*")) }
        openMedia = { filePicker.launch(arrayOf("image/*", "video/*")) }
        CrepusActions.dispatch = { action -> dispatchHostAction(action) ?: dispatchActionJson(action) }
        CrepusActions.resultSink = { result -> CrepusActionState.record(result) }
    }

    private fun dispatchHostAction(action: String): String? {
        dispatchNamedHostAction(action)?.let { return it }
        val request = runCatching { JSONObject(action) }.getOrNull() ?: return null
        if (request.optString("kind") != "plugin") return null
        if (request.optString("capability") != "clipboard") return null
        val method = request.optString("method")
        val actionName = "clipboard.$method"
        return runCatching {
            val value = clipboardValue(method, request.optJSONObject("payload"))
            JSONObject()
                .put("ok", true)
                .put("action", actionName)
                .put(
                    "value",
                    JSONObject()
                        .put("capability", "clipboard")
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
    private val json = Json { ignoreUnknownKeys = true }

    fun dispatch(action: String) {
        record(CrepusActions.dispatch(action))
    }

    fun record(result: String) {
        lastResult.value = result
        lastError.value =
            runCatching {
                val payload = json.parseToJsonElement(result).jsonObject
                result.takeIf { payload["ok"]?.jsonPrimitive?.booleanOrNull == false }
            }.getOrElse {
                result.takeIf { it.isNotBlank() }
            }
    }
}
