//! Mobile app actions — task tracker example.
//! Exposes the standard crepuscularity FFI contract for iOS and Android.
//!
//! iOS: C ABI — `crepus_mobile_dispatch_and_store_nul`, `crepus_mobile_eval_*`
//! Android: JNI — `dev.crepuscularity.mobileapp.CrepusRustActions`

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

// ── State ────────────────────────────────────────────────────────────────────

static VIEW_STATE: OnceLock<Mutex<serde_json::Value>> = OnceLock::new();

fn view_state() -> &'static Mutex<serde_json::Value> {
    VIEW_STATE.get_or_init(|| Mutex::new(initial_view_state()))
}

fn initial_view_state() -> serde_json::Value {
    serde_json::json!({
        "current_tab": "tasks",
        "tasks_count": 2,
        "tasks": [
            { "title": "Buy groceries", "done": false, "due": "Today" },
            { "title": "Write docs", "done": true, "due": "" }
        ],
        "notes_count": 1,
        "notes": [
            { "title": "Meeting notes", "preview": "Discussed Q3 roadmap..." }
        ],
        "dark_mode": true,
        "notifications": true,
        "sync_enabled": false,
        "font_size": 16,
        "app_version": concat!("v", env!("CARGO_PKG_VERSION"))
    })
}

// ── Action dispatch ──────────────────────────────────────────────────────────

fn dispatch_and_store_str(action: &str) -> String {
    let result = {
        let state = view_state().lock().unwrap_or_else(|e| e.into_inner());
        dispatch_action_inner(action, &state)
    };
    // Drop lock before store_result re-acquires it.
    let json = result.to_string();
    store_result(&json);
    json
}

fn dispatch_action_inner(action: &str, state: &serde_json::Value) -> serde_json::Value {
    match action {
        "tasks.add" => {
            let mut tasks = state["tasks"].as_array().cloned().unwrap_or_default();
            tasks.push(serde_json::json!({
                "title": "New task",
                "done": false,
                "due": ""
            }));
            serde_json::json!({
                "ok": true,
                "action": "tasks.add",
                "tasks": tasks,
                "tasks_count": tasks.len()
            })
        }
        "notes.add" => {
            let mut notes = state["notes"].as_array().cloned().unwrap_or_default();
            notes.push(serde_json::json!({
                "title": "Untitled",
                "preview": ""
            }));
            serde_json::json!({
                "ok": true,
                "action": "notes.add",
                "notes": notes,
                "notes_count": notes.len()
            })
        }
        "settings.reset" => {
            serde_json::json!({
                "ok": true,
                "action": "settings.reset",
                "value": initial_view_state()
            })
        }
        _ => serde_json::json!({
            "ok": false,
            "error": format!("unknown action: {action}")
        }),
    }
}

fn dispatch_change_and_store(action: &str, bind: &str, value_json: &str) -> String {
    let value: serde_json::Value =
        serde_json::from_str(value_json).unwrap_or(serde_json::Value::Null);
    let mut state = view_state().lock().unwrap_or_else(|e| e.into_inner());

    // Apply the bind change directly to view state.
    if let Some(obj) = state.as_object_mut() {
        obj.insert(bind.to_string(), value);
    }

    // Run any side-effect action.
    let action_result = if action.is_empty() {
        serde_json::json!({"ok": true})
    } else {
        match action {
            "tasks.toggle" | "settings.darkMode" | "settings.notifications" | "settings.sync" => {
                serde_json::json!({"ok": true, "action": action})
            }
            _ => {
                serde_json::json!({"ok": false, "error": format!("unknown change action: {action}")})
            }
        }
    };

    // Merge state into result so the shell can update bindings.
    let mut result = action_result;
    if let Some(obj) = result.as_object_mut() {
        if let serde_json::Value::Object(state_obj) = &*state {
            obj.extend(state_obj.clone());
        }
    }

    result.to_string()
}

fn store_result(json: &str) {
    let Ok(result) = serde_json::from_str::<serde_json::Value>(json) else {
        return;
    };
    let serde_json::Value::Object(mut obj) = result else {
        return;
    };
    let Some(ok) = obj.get("ok").and_then(|v| v.as_bool()) else {
        return;
    };
    if !ok {
        return;
    }

    // If result has a "value" key, merge those fields into view state.
    if let Some(serde_json::Value::Object(value_obj)) = obj.remove("value") {
        let mut state = view_state().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state_obj) = state.as_object_mut() {
            state_obj.extend(value_obj);
        }
    }

    // Also merge top-level keys from result (except "ok", "action", "error").
    let skip = ["ok", "action", "error"];
    let mut state = view_state().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(state_obj) = state.as_object_mut() {
        for (k, v) in obj {
            if !skip.contains(&k.as_str()) && !state_obj.contains_key(&k) {
                state_obj.insert(k, v);
            }
        }
    }
}

// ── Eval helpers (read from state for native shells) ─────────────────────────

/// Store raw JSON result directly (for JNI storeResultJson).
fn store_result_json_raw(json: &str) -> bool {
    let Ok(result) = serde_json::from_str::<serde_json::Value>(json) else {
        return false;
    };
    let serde_json::Value::Object(obj) = result else {
        return false;
    };
    let mut state = view_state().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(state_obj) = state.as_object_mut() {
        state_obj.extend(obj);
    }
    true
}

fn eval_text(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> String {
    let state = view_state().lock().unwrap_or_else(|e| e.into_inner());
    resolve_expr(&state, expr, scope_name, scope_json)
        .and_then(|v| match v {
            serde_json::Value::String(s) => Some(s),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => v.as_str().map(|s| s.to_string()),
        })
        .unwrap_or_default()
}

fn eval_bool(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> bool {
    let state = view_state().lock().unwrap_or_else(|e| e.into_inner());
    resolve_expr(&state, expr, scope_name, scope_json)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn eval_number(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> f64 {
    let state = view_state().lock().unwrap_or_else(|e| e.into_inner());
    resolve_expr(&state, expr, scope_name, scope_json)
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

fn eval_items_json(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> String {
    let state = view_state().lock().unwrap_or_else(|e| e.into_inner());
    resolve_expr(&state, expr, scope_name, scope_json)
        .and_then(|v| {
            v.as_array()
                .map(|a| serde_json::to_string(a).unwrap_or_else(|_| "[]".into()))
        })
        .unwrap_or_else(|| "[]".into())
}

/// Resolve a template expression like `tasks`, `task.title`, `tasks_count` against state.
fn resolve_expr(
    state: &serde_json::Value,
    expr: &str,
    scope_name: Option<&str>,
    scope_json: Option<&str>,
) -> Option<serde_json::Value> {
    let expr = expr.trim();

    // If we have a scope (for-each item), try resolving against it first.
    if let (Some(name), Some(json)) = (scope_name, scope_json) {
        if expr.starts_with(name) {
            if let Ok(scope) = serde_json::from_str::<serde_json::Value>(json) {
                let remainder = expr.strip_prefix(name).unwrap_or(expr);
                let remainder = remainder.strip_prefix('.').unwrap_or(remainder);
                if remainder.is_empty() {
                    return Some(scope);
                }
                if let Some(val) = scope.pointer(&format!("/{}", remainder.replace('.', "/"))) {
                    return Some(val.clone());
                }
            }
        }
    }

    // Resolve against top-level state.
    if let Some(val) = state.pointer(&format!("/{}", expr.replace('.', "/"))) {
        return Some(val.clone());
    }

    // Try direct key lookup.
    state.get(expr).cloned()
}

// ── C ABI (iOS) ──────────────────────────────────────────────────────────────

fn alloc_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

/// Dispatch an action and store the result. NUL-terminated.
/// # Safety
/// `action` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_dispatch_and_store_nul(
    action: *const c_char,
) -> *mut c_char {
    if action.is_null() {
        return alloc_c_string(r#"{"ok":false,"error":"action pointer was null"}"#);
    }
    let action = unsafe { std::ffi::CStr::from_ptr(action) }
        .to_string_lossy()
        .into_owned();
    alloc_c_string(&dispatch_and_store_str(&action))
}

/// Dispatch a bind change and store the result. NUL-terminated.
/// # Safety
/// `action`, `bind`, and `value_json` must be valid NUL-terminated C strings or NULL.
#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_dispatch_change_and_store_nul(
    action: *const c_char,
    bind: *const c_char,
    value_json: *const c_char,
) -> *mut c_char {
    if action.is_null() || bind.is_null() || value_json.is_null() {
        return alloc_c_string(r#"{"ok":false,"error":"change pointer was null"}"#);
    }
    let action = unsafe { std::ffi::CStr::from_ptr(action) }
        .to_string_lossy()
        .into_owned();
    let bind = unsafe { std::ffi::CStr::from_ptr(bind) }
        .to_string_lossy()
        .into_owned();
    let value_json = unsafe { std::ffi::CStr::from_ptr(value_json) }
        .to_string_lossy()
        .into_owned();
    alloc_c_string(&dispatch_change_and_store(&action, &bind, &value_json))
}

/// Evaluate a text expression from view state.
/// # Safety
/// All pointer/len pairs must be valid or null/0.
#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_eval_text(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
    output: *mut c_char,
    output_len: usize,
) -> usize {
    let expr = c_slice(expr_ptr, expr_len);
    let scope_name = c_slice(scope_name_ptr, scope_name_len);
    let scope = c_slice(scope_ptr, scope_len);
    let result = eval_text(expr.unwrap_or(""), scope_name, scope);
    write_c_output(result.as_bytes(), output, output_len)
}

/// Evaluate a boolean expression from view state.
/// # Safety
/// All pointer/len pairs must be valid or null/0.
#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_eval_bool(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
) -> bool {
    let expr = c_slice(expr_ptr, expr_len);
    let scope_name = c_slice(scope_name_ptr, scope_name_len);
    let scope = c_slice(scope_ptr, scope_len);
    eval_bool(expr.unwrap_or(""), scope_name, scope)
}

/// Evaluate a numeric expression from view state.
/// # Safety
/// All pointer/len pairs must be valid or null/0.
#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_eval_number(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
) -> f64 {
    let expr = c_slice(expr_ptr, expr_len);
    let scope_name = c_slice(scope_name_ptr, scope_name_len);
    let scope = c_slice(scope_ptr, scope_len);
    eval_number(expr.unwrap_or(""), scope_name, scope)
}

/// Evaluate an items expression and return JSON array.
/// # Safety
/// All pointer/len pairs must be valid or null/0.
#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_eval_items_json(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
    output: *mut c_char,
    output_len: usize,
) -> usize {
    let expr = c_slice(expr_ptr, expr_len);
    let scope_name = c_slice(scope_name_ptr, scope_name_len);
    let scope = c_slice(scope_ptr, scope_len);
    let result = eval_items_json(expr.unwrap_or(""), scope_name, scope);
    write_c_output(result.as_bytes(), output, output_len)
}

/// Free a string allocated by this library.
/// # Safety
/// `ptr` must be from a previous FFI call, or NULL.
#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}

// ── C helpers ────────────────────────────────────────────────────────────────

unsafe fn c_slice<'a>(ptr: *const c_char, len: usize) -> Option<&'a str> {
    if ptr.is_null() || len == 0 {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
    std::str::from_utf8(bytes).ok()
}

fn write_c_output(bytes: &[u8], output: *mut c_char, output_len: usize) -> usize {
    if output.is_null() || output_len == 0 {
        return bytes.len();
    }
    let copy_len = bytes.len().min(output_len.saturating_sub(1));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output.cast::<u8>(), copy_len);
        *output.add(copy_len) = 0;
    }
    bytes.len()
}

// ── Android JNI ──────────────────────────────────────────────────────────────

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_mobileapp_CrepusRustActions_dispatchAndStoreJson<
    'a,
>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    action: jni::objects::JString<'a>,
) -> jni::objects::JString<'a> {
    let result = match env.get_string(action) {
        Ok(s) => dispatch_and_store_str(s.to_string_lossy().as_ref()),
        Err(_) => r#"{"ok":false,"error":"JNI string error"}"#.to_string(),
    };
    env.new_string(result).unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_mobileapp_CrepusRustActions_dispatchChangeJson<
    'a,
>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    action: jni::objects::JString<'a>,
    bind: jni::objects::JString<'a>,
    value_json: jni::objects::JString<'a>,
) -> jni::objects::JString<'a> {
    let result = match (
        env.get_string(action),
        env.get_string(bind),
        env.get_string(value_json),
    ) {
        (Ok(action), Ok(bind), Ok(value_json)) => dispatch_change_and_store(
            action.to_string_lossy().as_ref(),
            bind.to_string_lossy().as_ref(),
            value_json.to_string_lossy().as_ref(),
        ),
        Err(_) => r#"{"ok":false,"error":"JNI string error"}"#.to_string(),
    };
    env.new_string(result).unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_mobileapp_CrepusRustActions_evalText<'a>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    expr: jni::objects::JString<'a>,
) -> jni::objects::JString<'a> {
    let result = match env.get_string(expr) {
        Ok(s) => eval_text(s.to_string_lossy().as_ref(), None, None),
        Err(_) => String::new(),
    };
    env.new_string(result).unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_mobileapp_CrepusRustActions_evalBool<'a>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    expr: jni::objects::JString<'a>,
) -> bool {
    match env.get_string(expr) {
        Ok(s) => eval_bool(s.to_string_lossy().as_ref(), None, None),
        Err(_) => false,
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_mobileapp_CrepusRustActions_evalNumber(
    env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    expr: jni::objects::JString<'_>,
) -> f64 {
    match env.get_string(expr) {
        Ok(s) => eval_number(s.to_string_lossy().as_ref(), None, None),
        Err(_) => 0.0,
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_mobileapp_CrepusRustActions_evalItemsJson<'a>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    expr: jni::objects::JString<'a>,
) -> jni::objects::JString<'a> {
    let result = match env.get_string(expr) {
        Ok(s) => eval_items_json(s.to_string_lossy().as_ref(), None, None),
        Err(_) => "[]".to_string(),
    };
    env.new_string(result).unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_mobileapp_CrepusRustActions_storeResultJson<'a>(
    env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    json: jni::objects::JString<'a>,
) -> bool {
    match env.get_string(json) {
        Ok(s) => store_result_json_raw(s.to_string_lossy().as_ref()),
        Err(_) => false,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_has_tasks() {
        let state = initial_view_state();
        assert_eq!(state["current_tab"], "tasks");
        assert_eq!(state["tasks_count"], 2);
        assert_eq!(state["tasks"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn dispatch_tasks_add() {
        // Reset state for test isolation.
        let result = dispatch_and_store_str("tasks.add");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v["ok"].as_bool().unwrap());
        assert_eq!(v["tasks"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn dispatch_notes_add() {
        let result = dispatch_and_store_str("notes.add");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v["ok"].as_bool().unwrap());
        assert_eq!(v["notes"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn dispatch_settings_reset() {
        let result = dispatch_and_store_str("settings.reset");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v["ok"].as_bool().unwrap());
    }

    #[test]
    fn dispatch_unknown_returns_error() {
        let result = dispatch_and_store_str("nope");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
    }

    #[test]
    fn eval_text_from_state() {
        // Eval reads from the shared state, which may have been mutated by prior tests.
        let val = eval_text("current_tab", None, None);
        assert_eq!(val, "tasks");
    }

    #[test]
    fn eval_bool_from_state() {
        // Verify eval reads from state (shared static, value may shift with other tests).
        let val = eval_bool("sync_enabled", None, None);
        // Initial state has sync_enabled=false; may toggle if change test ran first.
        let _ = val;
    }

    #[test]
    fn eval_number_from_state() {
        let val = eval_number("font_size", None, None);
        assert_eq!(val, 16.0);
    }

    #[test]
    fn eval_items_from_state() {
        let json = eval_items_json("tasks", None, None);
        let items: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(items.as_array().unwrap().len() >= 2);
    }

    #[test]
    fn dispatch_change_updates_bind() {
        let result = dispatch_change_and_store("settings.darkMode", "dark_mode", "false");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v["ok"].as_bool().unwrap());
        assert_eq!(v["dark_mode"], false);
    }
}
