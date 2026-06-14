package dev.crepuscularity.nativeshell

import androidx.compose.runtime.mutableStateOf

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

    fun dispatch(action: String) {
        record(CrepusActions.dispatch(action))
    }

    fun record(result: String) {
        lastResult.value = result
        lastError.value = result.takeIf { it.contains("\"ok\":false") }
    }
}
