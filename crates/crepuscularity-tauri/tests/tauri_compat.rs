use crepuscularity_tauri as tauri;

#[tauri::command]
fn greet(name: &str) -> Result<String, String> {
    Ok(format!("Hello, {name}"))
}

#[tauri::command]
async fn count(value: u32) -> u32 {
    value + 1
}

struct Prefix(String);

#[tauri::command]
fn stateful(prefix: tauri::State<'_, Prefix>, name: String) -> String {
    format!("{} {name}", prefix.0)
}

#[test]
fn standard_tauri_command_shape_routes_to_native_bridge() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler!(greet))
        .run(tauri::generate_context!())
        .unwrap();
    assert_eq!(
        app.invoke("greet", serde_json::json!({ "name": "Ada" }))
            .unwrap(),
        serde_json::json!("Hello, Ada")
    );
}

#[test]
fn async_commands_route_to_native_bridge() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler!(count))
        .build();
    assert_eq!(
        app.invoke("count", serde_json::json!({ "value": 41 }))
            .unwrap(),
        42
    );
}

#[test]
fn managed_state_routes_to_commands() {
    let app = tauri::Builder::default()
        .manage(Prefix("Hello,".into()))
        .invoke_handler(tauri::generate_handler!(stateful))
        .build();
    assert_eq!(
        app.invoke("stateful", serde_json::json!({ "name": "Ada" }))
            .unwrap(),
        "Hello, Ada"
    );
}
