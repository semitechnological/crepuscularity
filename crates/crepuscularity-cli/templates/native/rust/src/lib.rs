use std::ffi::CStr;
use std::os::raw::c_char;

#[cfg(target_os = "android")]
use jni::objects::{JClass, JString};
#[cfg(target_os = "android")]
use jni::JNIEnv;

pub fn dispatch_action(action: &str) -> bool {
    matches!(action, "sync" | "preview")
}

#[no_mangle]
pub extern "C" fn crepus_mobile_dispatch(action_ptr: *const c_char, action_len: usize) -> bool {
    if action_ptr.is_null() {
        return false;
    }
    let action = if action_len == 0 {
        // SAFETY: `action_ptr` is checked non-null above and must point to a valid NUL-terminated C string from the platform bridge.
        unsafe { CStr::from_ptr(action_ptr) }.to_string_lossy()
    } else {
        // SAFETY: the platform bridge passes a valid pointer to `action_len` bytes for the duration of this call.
        let bytes = unsafe { std::slice::from_raw_parts(action_ptr.cast::<u8>(), action_len) };
        String::from_utf8_lossy(bytes)
    };
    dispatch_action(action.as_ref())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn dispatch_known_action_returns_true() {
        assert!(dispatch_action("sync"));
        assert!(dispatch_action("preview"));
    }

    #[test]
    fn c_abi_accepts_len_prefixed_action() {
        let action = "sync";
        assert!(crepus_mobile_dispatch(
            action.as_ptr().cast::<c_char>(),
            action.len()
        ));
    }

    #[test]
    fn c_abi_accepts_c_string_action() {
        let action = CString::new("preview").unwrap();
        assert!(crepus_mobile_dispatch(action.as_ptr(), 0));
    }

    #[test]
    fn unknown_action_returns_false() {
        assert!(!dispatch_action("missing"));
    }
}
