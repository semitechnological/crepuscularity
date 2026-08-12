package dev.crepuscularity.mobileapp

/**
 * JNI bridge to the Rust mobile-app-actions crate.
 * All methods call into the native staticlib via JNI.
 */
object CrepusRustActions {
    init {
        System.loadLibrary("mobile_app_actions")
    }

    // ── Dispatch ──────────────────────────────────────────────────────────

    fun install() {
        CrepusActions.dispatch = { action -> dispatchAndStoreJson(action) }
        CrepusActions.resultSink = { result -> CrepusActions.applyResult(result) }
    }

    /** Dispatch an action string to Rust and return JSON result. */
    external fun dispatchAndStoreJson(action: String): String

    /** Dispatch a bind change to Rust and return JSON result. */
    external fun dispatchChangeJson(action: String, bind: String, valueJson: String): String

    /** Store a JSON result in Rust view state. Returns true if stored. */
    external fun storeResultJson(json: String): Boolean

    // ── Eval ──────────────────────────────────────────────────────────────

    external fun evalText(expr: String): String
    external fun evalBool(expr: String): Boolean
    external fun evalNumber(expr: String): Double
    external fun evalItemsJson(expr: String): String
}
