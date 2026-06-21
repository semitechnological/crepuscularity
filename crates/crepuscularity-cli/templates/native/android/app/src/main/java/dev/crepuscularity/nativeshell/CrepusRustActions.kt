package dev.crepuscularity.nativeshell

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
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

    external fun dispatchAction(action: String): Boolean
    external fun dispatchActionJson(action: String): String

    fun install(context: Context) {
        appContext = context.applicationContext
        CrepusActions.dispatch = { action -> dispatchHostAction(action) ?: dispatchActionJson(action) }
        CrepusActions.resultSink = { result -> CrepusActionState.record(result) }
    }

    private fun dispatchHostAction(action: String): String? {
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
