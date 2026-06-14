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
}

static ACTION_STATE: OnceLock<Mutex<MobileActionState>> = OnceLock::new();

fn action_state() -> &'static Mutex<MobileActionState> {
    ACTION_STATE.get_or_init(|| Mutex::new(MobileActionState::default()))
}

pub fn dispatch_action(action: &str) -> bool {
    dispatch_action_json(action).contains("\"ok\":true")
}

pub fn dispatch_action_json(action: &str) -> String {
    let mut state = action_state().lock().unwrap();
    match action {
        "sync" => {
            state.sync_count += 1;
            state.last_action = action.to_string();
            action_json(true, action, None, &state)
        }
        "preview" => {
            state.preview_count += 1;
            state.last_action = action.to_string();
            action_json(true, action, None, &state)
        }
        _ => action_json(false, action, Some("unknown action"), &state),
    }
}

fn action_json(
    ok: bool,
    action: &str,
    error: Option<&str>,
    state: &MobileActionState,
) -> String {
    let error = error
        .map(|error| format!(",\"error\":\"{}\"", json_escape(error)))
        .unwrap_or_default();
    format!(
        "{{\"ok\":{ok},\"action\":\"{}\"{error},\"state\":{{\"syncCount\":{},\"previewCount\":{},\"lastAction\":\"{}\"}}}}",
        json_escape(action),
        state.sync_count,
        state.preview_count,
        json_escape(&state.last_action)
    )
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
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

        assert!(first.contains("\"ok\":true"));
        assert!(first.contains("\"action\":\"sync\""));
        assert!(first.contains("\"syncCount\":1"));
        assert!(second.contains("\"ok\":true"));
        assert!(second.contains("\"action\":\"preview\""));
        assert!(second.contains("\"syncCount\":1"));
        assert!(second.contains("\"previewCount\":1"));
        assert!(second.contains("\"lastAction\":\"preview\""));
    }

    #[test]
    fn dispatch_action_json_reports_unknown_actions() {
        let _guard = test_lock();
        reset_action_state();
        let result = dispatch_action_json("missing");

        assert!(result.contains("\"ok\":false"));
        assert!(result.contains("\"action\":\"missing\""));
        assert!(result.contains("\"error\":\"unknown action\""));
    }

    #[test]
    fn dispatch_known_action_returns_true() {
        let _guard = test_lock();
        reset_action_state();
        assert!(dispatch_action("sync"));
        assert!(dispatch_action("preview"));
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
