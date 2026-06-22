package dev.crepuscularity.nativeshell

import androidx.compose.runtime.mutableStateOf
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

object CrepusRustActions {
    init {
        System.loadLibrary("crepus_mobile_actions")
    }

    external fun dispatchAction(action: String): Boolean
    external fun dispatchActionJson(action: String): String

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
