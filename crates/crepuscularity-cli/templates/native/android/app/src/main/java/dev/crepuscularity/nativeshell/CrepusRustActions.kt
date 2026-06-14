package dev.crepuscularity.nativeshell

object CrepusRustActions {
    init {
        System.loadLibrary("crepus_mobile_actions")
    }

    external fun dispatchAction(action: String): Boolean
    external fun dispatchActionJson(action: String): String

    fun install() {
        CrepusActions.dispatch = { action -> dispatchActionJson(action) }
    }
}
