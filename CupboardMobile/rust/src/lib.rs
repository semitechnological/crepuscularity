use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

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
    last_result: String,
    last_error: Option<String>,
}

#[derive(Clone, Copy)]
enum ActionKind {
    Named(ActionHandler),
    Plugin,
}

static ACTION_STATE: OnceLock<Mutex<MobileActionState>> = OnceLock::new();
static VIEW_STATE: OnceLock<Mutex<serde_json::Value>> = OnceLock::new();
type ActionHandler =
    fn(&mut MobileActionState, Option<&serde_json::Value>) -> Result<serde_json::Value, String>;

fn action_state() -> &'static Mutex<MobileActionState> {
    ACTION_STATE.get_or_init(|| Mutex::new(MobileActionState::default()))
}

fn view_state() -> &'static Mutex<serde_json::Value> {
    VIEW_STATE.get_or_init(|| Mutex::new(serde_json::json!({})))
}

fn lock_action_state() -> MutexGuard<'static, MobileActionState> {
    action_state()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn lock_view_state() -> MutexGuard<'static, serde_json::Value> {
    view_state()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn dispatch_action(action: &str) -> bool {
    action_kind(action).is_some()
}

pub fn dispatch_action_json(action: &str) -> String {
    let request = parse_action_request(action);
    let Some(kind) = action_kind(action) else {
        let state = lock_action_state();
        let action_name = legacy_action_name(&request).unwrap_or(action);
        return action_json(false, action_name, None, Some("unknown action"), &state);
    };
    let mut state = lock_action_state();
    let action_name = request_action_name(&request);
    state.last_action = action_name.clone();
    state.last_payload = request_payload(&request).map(serde_json::Value::to_string);
    let result = match kind {
        ActionKind::Named(handler) => handler(&mut state, request_payload(&request)),
        ActionKind::Plugin => plugin_action(&mut state, &request),
    };
    let result = match result {
        Ok(value) => action_json(true, &action_name, Some(value), None, &state),
        Err(error) => action_json(false, &action_name, None, Some(&error), &state),
    };
    state.last_error = parse_action_error(&result);
    state.last_result = result.clone();
    result
}

fn parse_action_error(result: &str) -> Option<String> {
    let payload = serde_json::from_str::<serde_json::Value>(result).ok()?;
    if payload.get("ok").and_then(|value| value.as_bool()) == Some(false) {
        return Some(
            payload
                .get("error")
                .and_then(|value| value.as_str())
                .unwrap_or(result)
                .to_string(),
        );
    }
    None
}

fn action_kind(action: &str) -> Option<ActionKind> {
    let request = parse_action_request(action);
    if request_kind(&request) == Some("plugin") {
        return Some(ActionKind::Plugin);
    }
    match legacy_action_name(&request).unwrap_or(action) {
        "sync" => Some(ActionKind::Named(sync_action)),
        "preview" => Some(ActionKind::Named(preview_action)),
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

fn plugin_action(
    state: &mut MobileActionState,
    request: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let capability = request
        .get("capability")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "plugin request missing capability".to_string())?;
    let method = request
        .get("method")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "plugin request missing method".to_string())?;
    let payload = request.get("payload");
    let value = match capability {
        "preferences" => preferences_backend(method, payload)?,
        "filesystem" => filesystem_backend(method, payload)?,
        "device" => device_backend(method)?,
        "app" => app_backend(state, method)?,
        other => return Err(format!("unsupported capability: {other}")),
    };
    Ok(serde_json::json!({
        "capability": capability,
        "method": method,
        "value": value,
    }))
}

fn preferences_backend(
    method: &str,
    payload: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let mut prefs = load_json_map(&preferences_file_path())?;
    match method {
        "get" => {
            let key = payload_string(payload, "key")?;
            Ok(prefs
                .get(&key)
                .cloned()
                .unwrap_or(serde_json::Value::Null))
        }
        "set" => {
            let key = payload_string(payload, "key")?;
            let value = payload
                .and_then(|value| value.get("value"))
                .cloned()
                .ok_or_else(|| "preferences.set requires payload.value".to_string())?;
            prefs.insert(key.clone(), value.clone());
            save_json_map(&preferences_file_path(), &prefs)?;
            Ok(serde_json::json!({ "key": key, "value": value }))
        }
        "remove" => {
            let key = payload_string(payload, "key")?;
            let removed = prefs.remove(&key).is_some();
            save_json_map(&preferences_file_path(), &prefs)?;
            Ok(serde_json::json!({ "key": key, "removed": removed }))
        }
        "keys" => {
            let mut keys = prefs.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            Ok(serde_json::json!(keys))
        }
        "clear" => {
            prefs.clear();
            save_json_map(&preferences_file_path(), &prefs)?;
            Ok(serde_json::json!({ "cleared": true }))
        }
        other => Err(format!("unsupported preferences method: {other}")),
    }
}

fn filesystem_backend(
    method: &str,
    payload: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    match method {
        "readText" => {
            let path = scoped_data_path(&payload_string(payload, "path")?)?;
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("read {}: {error}", path.display()))?;
            Ok(serde_json::json!({ "path": path_string(&path), "text": text }))
        }
        "writeText" => {
            let path = scoped_data_path(&payload_string(payload, "path")?)?;
            let text = payload_string(payload, "text")?;
            ensure_parent_dir(&path)?;
            fs::write(&path, text.as_bytes())
                .map_err(|error| format!("write {}: {error}", path.display()))?;
            Ok(file_stat_json(&path)?)
        }
        "delete" => {
            let path = scoped_data_path(&payload_string(payload, "path")?)?;
            let deleted = if path.is_dir() {
                fs::remove_dir_all(&path).is_ok()
            } else {
                fs::remove_file(&path).is_ok()
            };
            Ok(serde_json::json!({ "path": path_string(&path), "deleted": deleted }))
        }
        "mkdir" => {
            let path = scoped_data_path(&payload_string(payload, "path")?)?;
            fs::create_dir_all(&path)
                .map_err(|error| format!("mkdir {}: {error}", path.display()))?;
            Ok(file_stat_json(&path)?)
        }
        "list" => {
            let path = scoped_data_path(&payload_string(payload, "path")?)?;
            let mut entries = fs::read_dir(&path)
                .map_err(|error| format!("list {}: {error}", path.display()))?
                .map(|entry| {
                    let entry = entry.map_err(|error| error.to_string())?;
                    let entry_path = entry.path();
                    let metadata = entry
                        .metadata()
                        .map_err(|error| format!("stat {}: {error}", entry_path.display()))?;
                    Ok(serde_json::json!({
                        "name": entry.file_name().to_string_lossy().to_string(),
                        "path": path_string(&entry_path),
                        "isDir": metadata.is_dir(),
                        "bytes": metadata.len(),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            entries.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
            Ok(serde_json::json!({ "path": path_string(&path), "entries": entries }))
        }
        "stat" => {
            let path = scoped_data_path(&payload_string(payload, "path")?)?;
            file_stat_json(&path)
        }
        other => Err(format!("unsupported filesystem method: {other}")),
    }
}

fn device_backend(method: &str) -> Result<serde_json::Value, String> {
    match method {
        "info" => Ok(serde_json::json!({
            "targetOs": std::env::consts::OS,
            "targetArch": std::env::consts::ARCH,
            "targetFamily": std::env::consts::FAMILY,
            "tempDir": path_string(&data_root_dir()),
        })),
        other => Err(format!("unsupported device method: {other}")),
    }
}

fn app_backend(state: &MobileActionState, method: &str) -> Result<serde_json::Value, String> {
    match method {
        "info" => Ok(serde_json::json!({
            "syncCount": state.sync_count,
            "previewCount": state.preview_count,
            "lastAction": state.last_action,
            "lastPayload": state.last_payload,
            "dataRoot": path_string(&data_root_dir()),
        })),
        other => Err(format!("unsupported app method: {other}")),
    }
}

fn parse_action_request(input: &str) -> serde_json::Value {
    serde_json::from_str(input)
        .ok()
        .filter(|value: &serde_json::Value| value.is_object())
        .unwrap_or_else(|| serde_json::json!({ "action": input }))
}

fn legacy_action_name<'a>(request: &'a serde_json::Value) -> Option<&'a str> {
    request.get("action").and_then(|value| value.as_str())
}

fn request_kind(request: &serde_json::Value) -> Option<&str> {
    request.get("kind").and_then(|value| value.as_str())
}

fn request_action_name(request: &serde_json::Value) -> String {
    if let Some(action) = legacy_action_name(request) {
        return action.to_string();
    }
    if request_kind(request) == Some("plugin") {
        return format!(
            "{}.{}",
            request
                .get("capability")
                .and_then(|value| value.as_str())
                .unwrap_or("plugin"),
            request
                .get("method")
                .and_then(|value| value.as_str())
                .unwrap_or("action")
        );
    }
    request_kind(request).unwrap_or("").to_string()
}

fn request_payload(request: &serde_json::Value) -> Option<&serde_json::Value> {
    request.get("payload")
}

fn payload_string(
    payload: Option<&serde_json::Value>,
    key: &str,
) -> Result<String, String> {
    payload
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| format!("payload missing string field {key}"))
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
        .unwrap_or_else(|| {
            action_json(
                false,
                "",
                None,
                Some("invalid action pointer"),
                &MobileActionState::default(),
            )
        });
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

fn alloc_c_string(result: &str) -> *mut c_char {
    CString::new(result).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}

fn data_root_dir() -> PathBuf {
    std::env::var_os("CREPUS_MOBILE_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("TMPDIR")
                .map(PathBuf::from)
                .map(|path| path.join("crepus-mobile-data"))
        })
        .unwrap_or_else(|| std::env::temp_dir().join("crepus-mobile-data"))
}

fn preferences_file_path() -> PathBuf {
    data_root_dir().join("preferences.json")
}

fn load_json_map(
    path: &PathBuf,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&raw)
            .map_err(|error| format!("parse {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::Map::new()),
        Err(error) => Err(format!("read {}: {error}", path.display())),
    }
}

fn save_json_map(
    path: &PathBuf,
    value: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("mkdir {}: {error}", parent.display()))?;
    }
    let raw = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| format!("write {}: {error}", path.display()))
}

fn path_string(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent)
        .map_err(|error| format!("mkdir {}: {error}", parent.display()))
}

fn scoped_data_path(raw: &str) -> Result<PathBuf, String> {
    let relative = sanitize_relative_path(raw)?;
    Ok(data_root_dir().join(relative))
}

fn sanitize_relative_path(raw: &str) -> Result<PathBuf, String> {
    let path = Path::new(raw);
    if path.is_absolute() {
        return Err("absolute paths are not allowed".to_string());
    }
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir => return Err("parent segments are not allowed".to_string()),
            _ => return Err("unsupported path component".to_string()),
        }
    }
    if clean.as_os_str().is_empty() {
        return Err("path must not be empty".to_string());
    }
    Ok(clean)
}

fn file_stat_json(path: &Path) -> Result<serde_json::Value, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("stat {}: {error}", path.display()))?;
    Ok(serde_json::json!({
        "path": path_string(&path.to_path_buf()),
        "isDir": metadata.is_dir(),
        "bytes": metadata.len(),
    }))
}

fn store_view_state(raw: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => {
            *lock_view_state() = value;
            true
        }
        Err(_) => false,
    }
}

fn eval_text(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> String {
    stringify_value(&resolve_expr(expr, scope_name, scope_json))
}

fn eval_bool(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> bool {
    truthy_value(&resolve_expr(expr, scope_name, scope_json))
}

fn eval_number(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> f64 {
    number_value(&resolve_expr(expr, scope_name, scope_json)).unwrap_or(0.0)
}

fn eval_items_json(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> String {
    match resolve_path(expr, scope_name, scope_json) {
        serde_json::Value::Array(items) => serde_json::Value::Array(items).to_string(),
        _ => "[]".to_string(),
    }
}

fn resolve_expr(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> serde_json::Value {
    let trimmed = expr.trim();
    if let Some(rest) = trimmed.strip_prefix('!') {
        return serde_json::Value::Bool(!truthy_value(&resolve_expr(rest, scope_name, scope_json)));
    }
    for op in [">=", "<=", "==", "!=", ">", "<"] {
        if let Some(index) = trimmed.find(op) {
            let left = resolve_path(trimmed[..index].trim(), scope_name, scope_json);
            let right = resolve_path(trimmed[index + op.len()..].trim(), scope_name, scope_json);
            return serde_json::Value::Bool(compare_values(&left, &right, op));
        }
    }
    resolve_path(trimmed, scope_name, scope_json)
}

fn resolve_path(expr: &str, scope_name: Option<&str>, scope_json: Option<&str>) -> serde_json::Value {
    if let Some(literal) = literal_value(expr) {
        return literal;
    }
    if let (Some(scope_name), Some(scope)) = (
        scope_name,
        scope_json.and_then(|value| serde_json::from_str(value).ok()),
    ) {
        if expr == scope_name {
            return scope;
        }
        let prefix = format!("{scope_name}.");
        if let Some(stripped) = expr.strip_prefix(&prefix) {
            return lookup_path(stripped, &scope).unwrap_or(serde_json::Value::Null);
        }
    }
    let state = lock_view_state();
    lookup_path(expr, &state).unwrap_or(serde_json::Value::Null)
}

fn lookup_path(path: &str, root: &serde_json::Value) -> Option<serde_json::Value> {
    if path.is_empty() {
        return Some(root.clone());
    }
    let mut current = root;
    for segment in path.split('.') {
        match current {
            serde_json::Value::Object(map) => current = map.get(segment)?,
            serde_json::Value::Array(items) => current = items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        }
    }
    Some(current.clone())
}

fn literal_value(expr: &str) -> Option<serde_json::Value> {
    match expr {
        "true" => Some(serde_json::Value::Bool(true)),
        "false" => Some(serde_json::Value::Bool(false)),
        "null" => Some(serde_json::Value::Null),
        _ if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 => Some(
            serde_json::Value::String(expr[1..expr.len() - 1].to_string()),
        ),
        _ => expr
            .parse::<i64>()
            .map(|value| serde_json::json!(value))
            .or_else(|_| expr.parse::<f64>().map(|value| serde_json::json!(value)))
            .ok(),
    }
}

fn compare_values(left: &serde_json::Value, right: &serde_json::Value, op: &str) -> bool {
    if let (Some(left), Some(right)) = (number_value(left), number_value(right)) {
        return match op {
            ">=" => left >= right,
            "<=" => left <= right,
            ">" => left > right,
            "<" => left < right,
            "==" => left == right,
            "!=" => left != right,
            _ => false,
        };
    }
    let left = stringify_value(left);
    let right = stringify_value(right);
    match op {
        "==" => left == right,
        "!=" => left != right,
        _ => false,
    }
}

fn number_value(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
}

fn truthy_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(value) => *value,
        serde_json::Value::Number(number) => number.as_f64().unwrap_or_default() != 0.0,
        serde_json::Value::String(text) => !text.is_empty(),
        serde_json::Value::Array(items) => !items.is_empty(),
        serde_json::Value::Object(map) => !map.is_empty(),
    }
}

fn stringify_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(number) => number.to_string(),
        _ => value.to_string(),
    }
}

#[no_mangle]
pub extern "C" fn crepus_mobile_store_result_json(
    json_ptr: *const c_char,
    json_len: usize,
) -> bool {
    action_from_ffi(json_ptr, json_len)
        .map(|json| store_view_state(json.as_ref()))
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn crepus_mobile_last_result() -> *mut c_char {
    let state = lock_action_state();
    alloc_c_string(&state.last_result)
}

#[no_mangle]
pub extern "C" fn crepus_mobile_last_error() -> *mut c_char {
    let state = lock_action_state();
    alloc_c_string(state.last_error.as_deref().unwrap_or(""))
}

#[no_mangle]
pub unsafe extern "C" fn crepus_mobile_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
pub extern "C" fn crepus_mobile_eval_text(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
    output_ptr: *mut c_char,
    output_len: usize,
) -> usize {
    let result = action_from_ffi(expr_ptr, expr_len)
        .map(|expr| {
            let scope_name = action_from_ffi(scope_name_ptr, scope_name_len);
            let scope = action_from_ffi(scope_ptr, scope_len);
            eval_text(expr.as_ref(), scope_name.as_deref(), scope.as_deref())
        })
        .unwrap_or_default();
    copy_json_to_output(&result, output_ptr, output_len)
}

#[no_mangle]
pub extern "C" fn crepus_mobile_eval_bool(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
) -> bool {
    action_from_ffi(expr_ptr, expr_len)
        .map(|expr| {
            let scope_name = action_from_ffi(scope_name_ptr, scope_name_len);
            let scope = action_from_ffi(scope_ptr, scope_len);
            eval_bool(expr.as_ref(), scope_name.as_deref(), scope.as_deref())
        })
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn crepus_mobile_eval_number(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
) -> f64 {
    action_from_ffi(expr_ptr, expr_len)
        .map(|expr| {
            let scope_name = action_from_ffi(scope_name_ptr, scope_name_len);
            let scope = action_from_ffi(scope_ptr, scope_len);
            eval_number(expr.as_ref(), scope_name.as_deref(), scope.as_deref())
        })
        .unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn crepus_mobile_eval_items_json(
    expr_ptr: *const c_char,
    expr_len: usize,
    scope_name_ptr: *const c_char,
    scope_name_len: usize,
    scope_ptr: *const c_char,
    scope_len: usize,
    output_ptr: *mut c_char,
    output_len: usize,
) -> usize {
    let result = action_from_ffi(expr_ptr, expr_len)
        .map(|expr| {
            let scope_name = action_from_ffi(scope_name_ptr, scope_name_len);
            let scope = action_from_ffi(scope_ptr, scope_len);
            eval_items_json(expr.as_ref(), scope_name.as_deref(), scope.as_deref())
        })
        .unwrap_or_else(|| "[]".to_string());
    copy_json_to_output(&result, output_ptr, output_len)
}

#[cfg(test)]
fn reset_action_state() {
    *lock_action_state() = MobileActionState::default();
}

#[cfg(test)]
fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
fn reset_test_data_root() {
    let _ = fs::remove_dir_all(data_root_dir());
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

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_storeResultJson(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    json: JString<'_>,
) -> bool {
    match env.get_string(&json) {
        Ok(json) => store_view_state(json.to_string_lossy().as_ref()),
        Err(_) => false,
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_lastResult<'a>(
    env: JNIEnv<'a>,
    _class: JClass<'a>,
) -> JString<'a> {
    let state = lock_action_state();
    env.new_string(state.last_result.clone()).unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_lastError<'a>(
    env: JNIEnv<'a>,
    _class: JClass<'a>,
) -> JString<'a> {
    let state = lock_action_state();
    env.new_string(state.last_error.clone().unwrap_or_default()).unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_evalText<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    expr: JString<'a>,
    scope_name: JString<'a>,
    scope: JString<'a>,
) -> JString<'a> {
    let expr = env
        .get_string(&expr)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let scope_name = env
        .get_string(&scope_name)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    let scope = env
        .get_string(&scope)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    env.new_string(eval_text(&expr, scope_name.as_deref(), scope.as_deref())).unwrap()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_evalBool(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    expr: JString<'_>,
    scope_name: JString<'_>,
    scope: JString<'_>,
) -> bool {
    let expr = env
        .get_string(&expr)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let scope_name = env
        .get_string(&scope_name)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    let scope = env
        .get_string(&scope)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    eval_bool(&expr, scope_name.as_deref(), scope.as_deref())
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_evalNumber(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    expr: JString<'_>,
    scope_name: JString<'_>,
    scope: JString<'_>,
) -> f64 {
    let expr = env
        .get_string(&expr)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let scope_name = env
        .get_string(&scope_name)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    let scope = env
        .get_string(&scope)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    eval_number(&expr, scope_name.as_deref(), scope.as_deref())
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_crepuscularity_nativeshell_CrepusRustActions_evalItemsJson<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    expr: JString<'a>,
    scope_name: JString<'a>,
    scope: JString<'a>,
) -> JString<'a> {
    let expr = env
        .get_string(&expr)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let scope_name = env
        .get_string(&scope_name)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    let scope = env
        .get_string(&scope)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty());
    env.new_string(eval_items_json(&expr, scope_name.as_deref(), scope.as_deref())).unwrap()
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
        let state = lock_action_state();
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
    fn preferences_round_trip_persists_values() {
        let _guard = test_lock();
        reset_action_state();
        reset_test_data_root();
        let set = dispatch_action_json(
            r#"{"kind":"plugin","capability":"preferences","method":"set","payload":{"key":"theme","value":"light"}}"#,
        );
        let get = dispatch_action_json(
            r#"{"kind":"plugin","capability":"preferences","method":"get","payload":{"key":"theme"}}"#,
        );

        let set: serde_json::Value = serde_json::from_str(&set).expect("json");
        let get: serde_json::Value = serde_json::from_str(&get).expect("json");
        assert_eq!(set["ok"], true);
        assert_eq!(get["value"]["value"], "light");
    }

    #[test]
    fn device_and_app_requests_return_real_values() {
        let _guard = test_lock();
        reset_action_state();
        let device =
            dispatch_action_json(r#"{"kind":"plugin","capability":"device","method":"info"}"#);
        let app = dispatch_action_json(r#"{"kind":"plugin","capability":"app","method":"info"}"#);

        let device: serde_json::Value = serde_json::from_str(&device).expect("json");
        let app: serde_json::Value = serde_json::from_str(&app).expect("json");
        assert_eq!(device["ok"], true);
        assert!(device["value"]["value"]["targetOs"].is_string());
        assert_eq!(app["ok"], true);
        assert_eq!(app["value"]["value"]["syncCount"], 0);
    }

    #[test]
    fn filesystem_round_trip_handles_text_files() {
        let _guard = test_lock();
        reset_action_state();
        reset_test_data_root();
        let write = dispatch_action_json(
            r#"{"kind":"plugin","capability":"filesystem","method":"writeText","payload":{"path":"notes/hello.txt","text":"hi"}}"#,
        );
        let read = dispatch_action_json(
            r#"{"kind":"plugin","capability":"filesystem","method":"readText","payload":{"path":"notes/hello.txt"}}"#,
        );
        let list = dispatch_action_json(
            r#"{"kind":"plugin","capability":"filesystem","method":"list","payload":{"path":"notes"}}"#,
        );

        let write: serde_json::Value = serde_json::from_str(&write).expect("json");
        let read: serde_json::Value = serde_json::from_str(&read).expect("json");
        let list: serde_json::Value = serde_json::from_str(&list).expect("json");
        assert_eq!(write["ok"], true);
        assert_eq!(read["value"]["value"]["text"], "hi");
        assert_eq!(list["value"]["value"]["entries"][0]["name"], "hello.txt");
    }

    #[test]
    fn filesystem_rejects_parent_escape() {
        let _guard = test_lock();
        reset_action_state();
        let result = dispatch_action_json(
            r#"{"kind":"plugin","capability":"filesystem","method":"readText","payload":{"path":"../secret.txt"}}"#,
        );

        let result: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"], "parent segments are not allowed");
    }

    #[test]
    fn dispatch_action_json_recovers_from_poisoned_state_lock() {
        let _guard = test_lock();
        reset_action_state();
        let _ = std::thread::spawn(|| {
            let _state = action_state().lock().unwrap();
            panic!("poison state lock");
        })
        .join();

        let result = dispatch_action_json("sync");

        let result: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(result["ok"], true);
        assert_eq!(result["action"], "sync");
        assert_eq!(result["state"]["syncCount"], 1);
        reset_action_state();
    }

    #[test]
    fn c_abi_json_null_pointer_reports_full_state_shape() {
        let _guard = test_lock();
        reset_action_state();
        let mut output = [0 as c_char; 256];

        let written = crepus_mobile_dispatch_json(std::ptr::null(), 0, output.as_mut_ptr(), output.len());

        assert!(written > 0);
        let result = unsafe { CStr::from_ptr(output.as_ptr()) }.to_string_lossy();
        let result: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"], "invalid action pointer");
        assert_eq!(result["state"]["syncCount"], 0);
        assert_eq!(result["state"]["previewCount"], 0);
        assert_eq!(result["state"]["lastAction"], "");
        assert!(result["state"].get("lastPayload").is_some());
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

    #[test]
    fn scoped_expr_resolution_prefers_scope_name_and_falls_back_to_state() {
        let _guard = test_lock();
        *lock_view_state() = serde_json::json!({
            "dashboard": { "title": "Root" },
            "status": "ready"
        });

        let scope = r#"{"title":"Scoped"}"#;
        assert_eq!(eval_text("dashboard.title", Some("dashboard"), Some(scope)), "Scoped");
        assert_eq!(eval_text("dashboard", Some("dashboard"), Some(scope)), r#"{"title":"Scoped"}"#);
        assert_eq!(eval_text("status", Some("dashboard"), Some(scope)), "ready");
    }
}
