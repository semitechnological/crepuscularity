package dev.crepuscularity.nativeshell

object CrepusRustActions {
    init {
        System.loadLibrary("crepus_mobile_actions")
    }

    external fun dispatchAction(action: String): Boolean

    fun install() {
        CrepusActions.dispatch = { action -> dispatchAction(action) }
    }
}
