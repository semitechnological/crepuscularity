use crepuscularity_webext::wasm::{storage, tabs, windows};
use serde_json::{json, Value};

pub async fn active_tab() -> Result<Option<tabs::Tab>, String> {
    let tabs_list = tabs::query(&tabs::QueryInfo {
        active: Some(true),
        current_window: Some(true),
        ..Default::default()
    })
    .await
    .map_err(|e| format!("query tabs: {}", e))?;
    Ok(tabs_list.into_iter().next())
}

pub async fn execute_background_command(command: &str, _args: &Value) -> Result<(), String> {
    match command {
        "create-tab" => {
            tabs::create(&tabs::CreateProperties::default())
                .await
                .map_err(|e| e.to_string())?;
        }
        "open-url" => {
            let url = _args.get("url").and_then(Value::as_str).unwrap_or("about:blank");
            tabs::create(&tabs::CreateProperties {
                url: Some(url.to_string()),
                active: Some(_args.get("active").and_then(Value::as_bool).unwrap_or(true)),
                ..Default::default()
            })
            .await
            .map_err(|e| e.to_string())?;
        }
        "previous-tab" => activate_relative_tab(-1).await?,
        "next-tab" => activate_relative_tab(1).await?,
        "visit-previous-tab" => {
            let storage_data = storage::session()
                .get_json(json!({"previousTabIds": []}))
                .await
                .map_err(|e| format!("get session: {}", e))?;
            if let Some(ids) = storage_data.get("previousTabIds").and_then(Value::as_array) {
                if let Some(id) = ids.first().and_then(Value::as_i64) {
                    tabs::update(
                        id,
                        &tabs::UpdateProperties {
                            active: Some(true),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        "first-tab" => activate_edge_tab(false).await?,
        "last-tab" => activate_edge_tab(true).await?,
        "duplicate-tab" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    tabs::duplicate(id).await.map_err(|e| e.to_string())?;
                }
            }
        }
        "toggle-pin" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    let pinned = !tab.pinned.unwrap_or(false);
                    tabs::update(
                        id,
                        &tabs::UpdateProperties {
                            pinned: Some(pinned),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        "toggle-mute" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    let session = storage::session();
                    let stored = session
                        .get_json(json!({"mutedTabIds": []}))
                        .await
                        .map_err(|e| format!("get session: {}", e))?;
                    let mut muted_ids: Vec<i64> = stored
                        .get("mutedTabIds")
                        .and_then(Value::as_array)
                        .map(|a| a.iter().filter_map(Value::as_i64).collect())
                        .unwrap_or_default();
                    let currently_muted = muted_ids.contains(&id);
                    tabs::update(
                        id,
                        &tabs::UpdateProperties {
                            muted: Some(!currently_muted),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    if currently_muted {
                        muted_ids.retain(|m| *m != id);
                    } else {
                        muted_ids.push(id);
                    }
                    session
                        .set(&json!({"mutedTabIds": muted_ids}))
                        .await
                        .map_err(|e| format!("save muted: {}", e))?;
                }
            }
        }
        "remove-tab" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    if let Some(url) = &tab.url {
                        storage::sync()
                            .set(&json!({ "lastClosedTabUrl": url }))
                            .await
                            .map_err(|e| format!("save url: {}", e))?;
                    }
                    tabs::remove(id).await.map_err(|e| e.to_string())?;
                }
            }
        }
        "restore-tab" => {
            let saved = storage::sync()
                .get_json(json!({"lastClosedTabUrl": ""}))
                .await
                .map_err(|e| format!("get url: {}", e))?;
            if let Some(url) = saved.get("lastClosedTabUrl").and_then(Value::as_str) {
                if !url.is_empty() {
                    tabs::create(&tabs::CreateProperties {
                        url: Some(url.to_string()),
                        active: Some(true),
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        "move-to-new-window" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    let win = windows::create(&windows::CreateData {
                        tab_id: Some(id),
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| e.to_string())?;
                    let _ = win;
                }
            }
        }
        "close-tabs-left" => {
            let all = query_current_window().await?;
            let active_idx = all
                .iter()
                .find(|t| t.active.unwrap_or(false))
                .and_then(|t| t.index)
                .unwrap_or(0);
            for tab in &all {
                if let (Some(idx), Some(id)) = (tab.index, tab.id) {
                    if idx < active_idx && !tab.pinned.unwrap_or(false) {
                        let _ = tabs::remove(id).await;
                    }
                }
            }
        }
        "close-tabs-right" => {
            let all = query_current_window().await?;
            let active_idx = all
                .iter()
                .find(|t| t.active.unwrap_or(false))
                .and_then(|t| t.index)
                .unwrap_or(0);
            for tab in &all {
                if let (Some(idx), Some(id)) = (tab.index, tab.id) {
                    if idx > active_idx && !tab.pinned.unwrap_or(false) {
                        let _ = tabs::remove(id).await;
                    }
                }
            }
        }
        "close-other-tabs" => {
            let all = query_current_window().await?;
            let active_id = all
                .iter()
                .find(|t| t.active.unwrap_or(false))
                .and_then(|t| t.id);
            for tab in &all {
                if let Some(id) = tab.id {
                    if Some(id) != active_id && !tab.pinned.unwrap_or(false) {
                        let _ = tabs::remove(id).await;
                    }
                }
            }
        }
        "move-tab-left" => {
            if let Some(tab) = active_tab().await? {
                if let (Some(id), Some(idx)) = (tab.id, tab.index) {
                    if idx > 0 {
                        tabs::move_tab(id, idx - 1).await.map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        "move-tab-right" => {
            if let Some(tab) = active_tab().await? {
                if let (Some(id), Some(idx)) = (tab.id, tab.index) {
                    tabs::move_tab(id, idx + 1).await.map_err(|e| e.to_string())?;
                }
            }
        }
        "zoom-in" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    let current = tabs::get_zoom(id).await.unwrap_or(1.0);
                    tabs::set_zoom(id, (current + 0.25).min(5.0))
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        "zoom-out" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    let current = tabs::get_zoom(id).await.unwrap_or(1.0);
                    tabs::set_zoom(id, (current - 0.25).max(0.25))
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        "zoom-reset" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    tabs::set_zoom(id, 1.0)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        "set-zoom" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    if let Some(level) = _args.get("level").and_then(Value::as_f64) {
                        tabs::set_zoom(id, level.clamp(0.25, 5.0))
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        "reload" => {
            if let Some(tab) = active_tab().await? {
                if let Some(id) = tab.id {
                    tabs::reload(id).await.map_err(|e| e.to_string())?;
                }
            }
        }
        _ => return Err(format!("unknown background command: {}", command)),
    }
    Ok(())
}

pub async fn query_current_window() -> Result<Vec<tabs::Tab>, String> {
    tabs::query(&tabs::QueryInfo {
        current_window: Some(true),
        ..Default::default()
    })
    .await
    .map_err(|e| format!("query tabs: {}", e))
}

pub async fn activate_relative_tab(delta: i64) -> Result<(), String> {
    let all = query_current_window().await?;
    if all.is_empty() {
        return Ok(());
    }
    let current = all
        .iter()
        .find(|tab| tab.active.unwrap_or(false))
        .and_then(|tab| tab.index)
        .unwrap_or(0);
    let next_index = (current + delta).rem_euclid(all.len() as i64);
    if let Some(tab_id) = all
        .iter()
        .find(|tab| tab.index == Some(next_index))
        .and_then(|tab| tab.id)
    {
        tabs::update(
            tab_id,
            &tabs::UpdateProperties {
                active: Some(true),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn activate_edge_tab(last: bool) -> Result<(), String> {
    let all = query_current_window().await?;
    let tab_id = if last {
        all.last().and_then(|t| t.id)
    } else {
        all.first().and_then(|t| t.id)
    };
    if let Some(id) = tab_id {
        tabs::update(
            id,
            &tabs::UpdateProperties {
                active: Some(true),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}