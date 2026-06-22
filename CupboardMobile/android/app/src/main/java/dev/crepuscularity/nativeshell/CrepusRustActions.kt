package dev.crepuscularity.nativeshell

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.jsonArray

object CrepusStateStore {
    private val json = Json { ignoreUnknownKeys = true }
    private var revision by mutableLongStateOf(0L)

    fun applyResult(raw: String) {
        if (CrepusRustActions.storeResultJson(raw)) {
            revision += 1
        }
    }

    fun text(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): String {
        revision
        return CrepusRustActions.evalText(expr, scopeName, scope?.toString())
    }

    fun bool(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): Boolean {
        revision
        return CrepusRustActions.evalBool(expr, scopeName, scope?.toString())
    }

    fun number(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): Float {
        revision
        return CrepusRustActions.evalNumber(expr, scopeName, scope?.toString()).toFloat()
    }

    fun items(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): List<kotlinx.serialization.json.JsonElement> {
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

    external fun dispatchAction(action: String): Boolean
    external fun dispatchActionJson(action: String): String
    external fun lastResult(): String
    external fun lastError(): String
    external fun storeResultJson(json: String): Boolean
    external fun evalText(expr: String, scopeName: String?, scopeJson: String?): String
    external fun evalBool(expr: String, scopeName: String?, scopeJson: String?): Boolean
    external fun evalNumber(expr: String, scopeName: String?, scopeJson: String?): Double
    external fun evalItemsJson(expr: String, scopeName: String?, scopeJson: String?): String

    fun install() {
        CrepusActions.dispatch = { action -> dispatchActionJson(action) }
        CrepusActions.resultSink = { result -> CrepusActionState.record(result) }
    }
}

object CrepusActionState {
    val lastResult = mutableStateOf("{}")
    val lastError = mutableStateOf<String?>(null)

    fun dispatch(action: String) {
        record(CrepusActions.dispatch(action))
    }

    fun record(result: String) {
        lastResult.value = CrepusRustActions.lastResult()
        lastError.value = CrepusRustActions.lastError().ifBlank { null }
    }
}
