use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "android")]
use jni::objects::{JClass, JString};
#[cfg(target_os = "android")]
use jni::JNIEnv;

#[derive(Default)]
struct MobileActionState {
    sync_count: u64,
    preview_count: u64,
    last_action: String,
    last_payload: Option<String>,
}

static ACTION_STATE: OnceLock<Mutex<MobileActionState>> = OnceLock::new();
type ActionHandler =
    fn(&mut MobileActionState, Option<&serde_json::Value>) -> Result<serde_json::Value, String>;

fn action_state() -> &'static Mutex<MobileActionState> {
    ACTION_STATE.get_or_init(|| Mutex::new(MobileActionState::default()))
}

pub fn dispatch_action(action: &str) -> bool {
    action_handler(action).is_some()
}

pub fn dispatch_action_json(action: &str) -> String {
    let request = parse_action_request(action);
    let action_name = request
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or(action);
    let payload = request.get("payload");
    let mut state = action_state().lock().unwrap();
    let Some(handler) = action_handler(action_name) else {
        return action_json(false, action_name, None, Some("unknown action"), &state);
    };
    state.last_action = action_name.to_string();
    state.last_payload = payload.map(serde_json::Value::to_string);
    match handler(&mut state, payload) {
        Ok(value) => action_json(true, action_name, Some(value), None, &state),
        Err(error) => action_json(false, action_name, None, Some(&error), &state),
    }
}

fn action_handler(action: &str) -> Option<ActionHandler> {
    match action {
        "sync" => Some(sync_action),
        "preview" => Some(preview_action),
        _ => None,
    }
}

fn sync_action(
    state: &mut MobileActionState,
    payload: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    state.sync_count += 1;
    Ok(serde_json::json!({
        "message": payload_message(payload).unwrap_or("synced"),
        "syncCount": state.sync_count,
    }))
}

fn preview_action(
    state: &mut MobileActionState,
    payload: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    state.preview_count += 1;
    Ok(serde_json::json!({
        "message": payload_message(payload).unwrap_or("previewed"),
        "previewCount": state.preview_count,
    }))
}

fn payload_message(payload: Option<&serde_json::Value>) -> Option<&str> {
    payload
        .and_then(|value| value.get("message"))
        .and_then(|value| value.as_str())
}

fn parse_action_request(input: &str) -> serde_json::Value {
    serde_json::from_str(input)
        .ok()
        .filter(|value: &serde_json::Value| value.get("action").is_some())
        .unwrap_or_else(|| serde_json::json!({ "action": input }))
}

fn action_json(
    ok: bool,
    action: &str,
    value: Option<serde_json::Value>,
    error: Option<&str>,
    state: &MobileActionState,
) -> String {
    let mut out = serde_json::json!({
        "ok": ok,
        "action": action,
        "state": {
            "syncCount": state.sync_count,
            "previewCount": state.preview_count,
            "lastAction": state.last_action,
            "lastPayload": state.last_payload,
        }
    });
    if let Some(value) = value {
        out["value"] = value;
    }
    if let Some(error) = error {
        out["error"] = serde_json::Value::String(error.to_string());
    }
    out.to_string()
}

#[no_mangle]
pub extern "C" fn crepus_mobile_dispatch(action_ptr: *const c_char, action_len: usize) -> bool {
    action_from_ffi(action_ptr, action_len)
        .map(|action| dispatch_action(action.as_ref()))
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn crepus_mobile_dispatch_json(
    action_ptr: *const c_char,
    action_len: usize,
    output_ptr: *mut c_char,
    output_len: usize,
) -> usize {
    let result = action_from_ffi(action_ptr, action_len)
        .map(|action| dispatch_action_json(action.as_ref()))
        .unwrap_or_else(|| "{\"ok\":false,\"action\":\"\",\"error\":\"invalid action pointer\",\"state\":{\"syncCount\":0,\"previewCount\":0,\"lastAction\":\"\"}}".to_string());
    copy_json_to_output(&result, output_ptr, output_len)
}

fn action_from_ffi(action_ptr: *const c_char, action_len: usize) -> Option<std::borrow::Cow<'static, str>> {
    if action_ptr.is_null() {
        return None;
    }
    Some(if action_len == 0 {
        // SAFETY: `action_ptr` is checked non-null above and must point to a valid NUL-terminated C string from the platform bridge.
        unsafe { CStr::from_ptr(action_ptr) }.to_string_lossy()
    } else {
        // SAFETY: the platform bridge passes a valid pointer to `action_len` bytes for the duration of this call.
        let bytes = unsafe { std::slice::from_raw_parts(action_ptr.cast::<u8>(), action_len) };
        String::from_utf8_lossy(bytes)
    }
    .into_owned()
    .into())
}

fn copy_json_to_output(result: &str, output_ptr: *mut c_char, output_len: usize) -> usize {
    let bytes = result.as_bytes();
    if output_ptr.is_null() || output_len == 0 {
        return bytes.len();
    }
    let copy_len = bytes.len().min(output_len.saturating_sub(1));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output_ptr.cast::<u8>(), copy_len);
        *output_ptr.add(copy_len) = 0;
    }
    bytes.len()
}

#[cfg(test)]
fn reset_action_state() {
    *action_state().lock().unwrap() = MobileActionState::default();
}

#[cfg(test)]
fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_dispatchAction(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    action: JString<'_>,
) -> bool {
    match env.get_string(&action) {
        Ok(action) => dispatch_action(action.to_string_lossy().as_ref()),
        Err(_) => false,
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_dispatchActionJson<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    action: JString<'a>,
) -> JString<'a> {
    let result = match env.get_string(&action) {
        Ok(action) => dispatch_action_json(action.to_string_lossy().as_ref()),
        Err(_) => "{\"ok\":false,\"action\":\"\",\"error\":\"invalid action string\",\"state\":{\"syncCount\":0,\"previewCount\":0,\"lastAction\":\"\"}}".to_string(),
    };
    env.new_string(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn dispatch_action_json_mutates_state() {
        let _guard = test_lock();
        reset_action_state();
        let first = dispatch_action_json("sync");
        let second = dispatch_action_json("preview");

        let first: serde_json::Value = serde_json::from_str(&first).expect("json");
        let second: serde_json::Value = serde_json::from_str(&second).expect("json");
        assert_eq!(first["ok"], true);
        assert_eq!(first["action"], "sync");
        assert_eq!(first["state"]["syncCount"], 1);
        assert_eq!(second["ok"], true);
        assert_eq!(second["action"], "preview");
        assert_eq!(second["state"]["syncCount"], 1);
        assert_eq!(second["state"]["previewCount"], 1);
        assert_eq!(second["state"]["lastAction"], "preview");
    }

    #[test]
    fn dispatch_action_json_reports_unknown_actions() {
        let _guard = test_lock();
        reset_action_state();
        let result = dispatch_action_json("missing");

        let result: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(result["ok"], false);
        assert_eq!(result["action"], "missing");
        assert_eq!(result["error"], "unknown action");
    }

    #[test]
    fn dispatch_known_action_checks_registry_without_mutation() {
        let _guard = test_lock();
        reset_action_state();
        assert!(dispatch_action("sync"));
        assert!(dispatch_action("preview"));
        let state = action_state().lock().unwrap();
        assert_eq!(state.sync_count, 0);
        assert_eq!(state.preview_count, 0);
    }

    #[test]
    fn dispatch_action_json_accepts_typed_payloads() {
        let _guard = test_lock();
        reset_action_state();
        let result = dispatch_action_json(r#"{"action":"sync","payload":{"message":"hydrate"}}"#);

        let result: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(result["ok"], true);
        assert_eq!(result["action"], "sync");
        assert_eq!(result["value"]["message"], "hydrate");
        assert_eq!(result["state"]["lastPayload"], r#"{"message":"hydrate"}"#);
    }

    #[test]
    fn c_abi_accepts_len_prefixed_action() {
        let _guard = test_lock();
        reset_action_state();
        let action = "sync";
        assert!(crepus_mobile_dispatch(
            action.as_ptr().cast::<c_char>(),
            action.len()
        ));
    }

    #[test]
    fn c_abi_accepts_c_string_action() {
        let _guard = test_lock();
        reset_action_state();
        let action = CString::new("preview").unwrap();
        assert!(crepus_mobile_dispatch(action.as_ptr(), 0));
    }

    #[test]
    fn unknown_action_returns_false() {
        let _guard = test_lock();
        reset_action_state();
        assert!(!dispatch_action("missing"));
    }
}
