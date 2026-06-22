package dev.crepuscularity.nativeshell

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

object CrepusStateStore {
    private val json = Json { ignoreUnknownKeys = true }
    private var revision by mutableLongStateOf(0L)

    fun applyResult(raw: String) {
        if (CrepusRustActions.storeResultJson(raw)) {
            revision += 1
        }
    }

    fun text(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): String {
        val request = normalize(expr, scopeName, scope)
        revision
        return CrepusRustActions.evalText(request.first, request.second)
    }

    fun bool(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): Boolean {
        val request = normalize(expr, scopeName, scope)
        revision
        return CrepusRustActions.evalBool(request.first, request.second)
    }

    fun number(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): Float {
        val request = normalize(expr, scopeName, scope)
        revision
        return CrepusRustActions.evalNumber(request.first, request.second).toFloat()
    }

    fun items(expr: String, scopeName: String? = null, scope: kotlinx.serialization.json.JsonElement? = null): List<kotlinx.serialization.json.JsonElement> {
        val request = normalize(expr, scopeName, scope)
        revision
        val payload = runCatching { json.parseToJsonElement(CrepusRustActions.evalItemsJson(request.first, request.second)) as? JsonArray }
            .getOrNull()
        return payload?.jsonArray?.toList() ?: emptyList()
    }

    private fun normalize(expr: String, scopeName: String?, scope: kotlinx.serialization.json.JsonElement?): Pair<String, String?> {
        if (scopeName == null || scope == null) {
            return expr to null
        }
        if (expr == scopeName) {
            return "" to scope.toString()
        }
        val prefix = "$scopeName."
        if (expr.startsWith(prefix)) {
            return expr.removePrefix(prefix) to scope.toString()
        }
        return expr to null
    }
}

object CrepusRustActions {
    init {
        System.loadLibrary("crepus_mobile_actions")
    }

    external fun dispatchAction(action: String): Boolean
    external fun dispatchActionJson(action: String): String
    external fun storeResultJson(json: String): Boolean
    external fun evalText(expr: String, scopeJson: String?): String
    external fun evalBool(expr: String, scopeJson: String?): Boolean
    external fun evalNumber(expr: String, scopeJson: String?): Double
    external fun evalItemsJson(expr: String, scopeJson: String?): String

    fun install() {
        CrepusActions.dispatch = { action -> dispatchActionJson(action) }
        CrepusActions.resultSink = { result -> CrepusActionState.record(result) }
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
