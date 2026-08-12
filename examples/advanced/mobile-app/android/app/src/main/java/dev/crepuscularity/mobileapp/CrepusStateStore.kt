package dev.crepuscularity.mobileapp

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.setValue
import kotlinx.serialization.json.JsonElement

/**
 * Reactive state store backed by Rust JNI eval functions.
 * Bumps [revision] on every [applyResult] call, triggering Compose recomposition.
 */
object CrepusStateStore {
    var revision by mutableLongStateOf(0L)
        private set

    fun refresh() {
        revision += 1
    }

    /** Store a JSON result in Rust and bump revision. */
    fun applyResult(raw: String) {
        val stored = CrepusRustActions.storeResultJson(raw)
        if (stored) refresh()
    }

    fun text(expr: String, scopeName: String? = null, scope: Any? = null): String {
        _ = revision // subscribe
        return CrepusRustActions.evalText(expr)
    }

    fun bool(expr: String, scopeName: String? = null, scope: Any? = null): Boolean {
        _ = revision // subscribe
        return CrepusRustActions.evalBool(expr)
    }

    fun number(expr: String, scopeName: String? = null, scope: Any? = null): Double {
        _ = revision // subscribe
        return CrepusRustActions.evalNumber(expr)
    }

    fun items(expr: String, scopeName: String? = null, scope: Any? = null): List<Any> {
        _ = revision // subscribe
        val json = CrepusRustActions.evalItemsJson(expr)
        return try {
            @Suppress("UNCHECKED_CAST")
            kotlinx.serialization.json.Json.decodeFromString<List<Map<String, JsonElement>>>(json)
        } catch (_: Exception) {
            emptyList()
        }
    }
}
