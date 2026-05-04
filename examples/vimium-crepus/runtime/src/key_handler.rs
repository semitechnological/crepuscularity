use crate::commands::{CommandEntry, KeyMapRegistry};
use serde_json::{json, Value};

#[derive(Debug, Clone, Default)]
pub struct KeyState {
    pub mode: String,
    pub sequence: String,
    pub count_text: String,
    pub input: String,
}

impl KeyState {
    pub fn new() -> Self {
        KeyState {
            mode: "normal".to_string(),
            sequence: String::new(),
            count_text: String::new(),
            input: String::new(),
        }
    }
}

pub fn handle_key(
    state: &KeyState,
    key: &str,
    editable: bool,
    registry: &KeyMapRegistry,
    user_mappings: &std::collections::HashMap<String, String>,
) -> Value {
    if key == "Esc" {
        return json!({
            "state": { "mode": "normal", "sequence": "", "countText": "", "input": "" },
            "effect": { "kind": "clear-overlays" },
            "prevent": false
        });
    }

    if state.mode == "insert" || editable {
        return json!({ "state": state_to_val(state), "effect": null, "prevent": false });
    }

    if state.mode == "hints" {
        return json!({ "state": state_to_val(state), "effect": null, "prevent": false });
    }

    if state.mode == "visual" {
        return handle_visual_key(state, key, registry, user_mappings);
    }

    let mut sequence = state.sequence.clone();
    let mut count_text = state.count_text.clone();

    if key.len() == 1
        && key.as_bytes()[0].is_ascii_digit()
        && sequence.is_empty()
        && (key != "0" || !count_text.is_empty())
    {
        count_text.push_str(key);
        return json!({
            "state": { "mode": "normal", "sequence": "", "countText": count_text, "input": "" },
            "effect": null,
            "prevent": true
        });
    }

    sequence.push_str(key);

    let count = parse_count(&count_text);
    let is_prefix = registry.is_prefix(&sequence, user_mappings);
    let resolved = registry.resolve_command(&sequence, user_mappings);

    if let Some((cmd_name, entry)) = resolved {
        if !is_prefix {
            let effect = command_effect(cmd_name, count, entry);
            let effect_mode = effect_mode(&effect);
            json!({
                "state": { "mode": effect_mode, "sequence": "", "countText": "", "input": "" },
                "effect": effect,
                "prevent": true
            })
        } else {
            let effect = command_effect(cmd_name, count, entry);
            let effect_mode = effect_mode(&effect);
            json!({
                "state": { "mode": effect_mode, "sequence": "", "countText": "", "input": "" },
                "effect": effect,
                "prevent": true
            })
        }
    } else if is_prefix {
        json!({
            "state": { "mode": "normal", "sequence": sequence, "countText": count_text, "input": "" },
            "effect": null,
            "prevent": true
        })
    } else {
        json!({
            "state": { "mode": "normal", "sequence": "", "countText": "", "input": "" },
            "effect": null,
            "prevent": false
        })
    }
}

pub fn handle_visual_key(
    state: &KeyState,
    key: &str,
    registry: &KeyMapRegistry,
    user_mappings: &std::collections::HashMap<String, String>,
) -> Value {
    let mut sequence = state.sequence.clone();
    let count_text = state.count_text.clone();

    sequence.push_str(key);
    let count = parse_count(&count_text);

    let is_prefix = registry.is_prefix(&sequence, user_mappings);
    let resolved = registry.resolve_command(&sequence, user_mappings);

    if let Some((cmd_name, entry)) = resolved {
        let effect = visual_effect(cmd_name, count, entry);
        let post_mode = match effect.get("kind").and_then(Value::as_str) {
            Some("exit-visual") | Some("none") | None => "normal",
            Some("stay-visual") => "visual",
            _ => "normal",
        };
        json!({
            "state": { "mode": post_mode, "sequence": "", "countText": "", "input": "" },
            "effect": effect,
            "prevent": true
        })
    } else if is_prefix {
        json!({
            "state": { "mode": "visual", "sequence": sequence, "countText": count_text, "input": "" },
            "effect": null,
            "prevent": true
        })
    } else {
        json!({
            "state": { "mode": "normal", "sequence": "", "countText": "", "input": "" },
            "effect": null,
            "prevent": false
        })
    }
}

fn parse_count(count_text: &str) -> i64 {
    count_text
        .parse::<i64>()
        .ok()
        .filter(|v| *v > 0)
        .unwrap_or(1)
}

fn state_to_val(state: &KeyState) -> Value {
    json!({
        "mode": state.mode,
        "sequence": state.sequence,
        "countText": state.count_text,
        "input": state.input
    })
}

pub fn command_effect(cmd_name: &str, count: i64, entry: Option<&CommandEntry>) -> Value {
    let bkg = entry.map_or(false, |e| e.background);
    let no_repeat = entry.map_or(false, |e| e.no_repeat);
    let count = if no_repeat { 1 } else { count };

    match cmd_name {
        "scrollDown" => json!({"kind": "scroll", "x": 0, "y": 80 * count}),
        "scrollUp" => json!({"kind": "scroll", "x": 0, "y": -80 * count}),
        "scrollToTop" => json!({"kind": "scroll-top"}),
        "scrollToBottom" => json!({"kind": "scroll-bottom"}),
        "scrollPageDown" => json!({"kind": "half-scroll", "direction": 1, "count": count}),
        "scrollPageUp" => json!({"kind": "half-scroll", "direction": -1, "count": count}),
        "scrollFullPageDown" => json!({"kind": "full-scroll", "direction": 1, "count": count}),
        "scrollFullPageUp" => json!({"kind": "full-scroll", "direction": -1, "count": count}),
        "scrollLeft" => json!({"kind": "scroll", "x": -120 * count, "y": 0}),
        "scrollRight" => json!({"kind": "scroll", "x": 120 * count, "y": 0}),
        "scrollToLeft" => json!({"kind": "scroll-left"}),
        "scrollToRight" => json!({"kind": "scroll-right"}),
        "reload" => json!({"kind": "reload"}),
        "copyCurrentUrl" => json!({"kind": "copy-url"}),
        "openCopiedUrlInCurrentTab" => json!({"kind": "open-clipboard", "newTab": false}),
        "openCopiedUrlInNewTab" => json!({"kind": "open-clipboard", "newTab": true}),
        "goUp" => json!({"kind": "go-up"}),
        "goToRoot" => json!({"kind": "go-root"}),
        "enterInsertMode" => json!({"kind": "insert-mode"}),
        "enterVisualMode" => json!({"kind": "enter-visual", "mode": "visual"}),
        "enterVisualLineMode" => json!({"kind": "enter-visual", "mode": "visual-line"}),
        "focusInput" => json!({"kind": "focus-input"}),
        "LinkHints.activateModeToOpenInNewTab" => json!({"kind": "hints", "newTab": true, "foreground": false}),
        "LinkHints.activateModeToOpenInNewForegroundTab" => json!({"kind": "hints", "newTab": true, "foreground": true}),
        "LinkHints.activateMode" => json!({"kind": "hints-general"}),
        "LinkHints.activateModeWithQueue" => json!({"kind": "hints-queue"}),
        "LinkHints.activateModeToDownloadLink" => json!({"kind": "hints-download"}),
        "LinkHints.activateModeToOpenIncognito" => json!({"kind": "hints-incognito"}),
        "LinkHints.activateModeToCopyLinkUrl" => json!({"kind": "hints-copy-url"}),
        "goPrevious" => json!({"kind": "follow-pattern", "pattern": "previous"}),
        "goNext" => json!({"kind": "follow-pattern", "pattern": "next"}),
        "nextFrame" => json!({"kind": "cycle-frame", "direction": 1}),
        "mainFrame" => json!({"kind": "focus-main-frame"}),
        "Marks.activateCreateMode" => json!({"kind": "create-mark"}),
        "Marks.activateGotoMode" => json!({"kind": "goto-mark"}),
        "Vomnibar.activate" => json!({"kind": "vomnibar", "newTab": false}),
        "Vomnibar.activateInNewTab" => json!({"kind": "vomnibar", "newTab": true}),
        "Vomnibar.activateBookmarks" => json!({"kind": "vomnibar-bookmarks", "newTab": false}),
        "Vomnibar.activateBookmarksInNewTab" => json!({"kind": "vomnibar-bookmarks", "newTab": true}),
        "Vomnibar.activateTabSelection" => json!({"kind": "vomnibar-tabs"}),
        "Vomnibar.activateEditUrl" => json!({"kind": "vomnibar-edit-url", "newTab": false}),
        "Vomnibar.activateEditUrlInNewTab" => json!({"kind": "vomnibar-edit-url", "newTab": true}),
        "enterFindMode" => json!({"kind": "find"}),
        "performFind" => json!({"kind": "find-next", "reverse": false}),
        "performBackwardsFind" => json!({"kind": "find-next", "reverse": true}),
        "findSelected" => json!({"kind": "find-selected", "reverse": false}),
        "findSelectedBackwards" => json!({"kind": "find-selected", "reverse": true}),
        "goBack" => json!({"kind": "history-back"}),
        "goForward" => json!({"kind": "history-forward"}),
        "createTab" => json!({"kind": "background", "command": "create-tab"}),
        "previousTab" => json!({"kind": "background", "command": "previous-tab"}),
        "nextTab" => json!({"kind": "background", "command": "next-tab"}),
        "visitPreviousTab" => json!({"kind": "background", "command": "visit-previous-tab"}),
        "firstTab" => json!({"kind": "background", "command": "first-tab"}),
        "lastTab" => json!({"kind": "background", "command": "last-tab"}),
        "duplicateTab" => json!({"kind": "background", "command": "duplicate-tab"}),
        "togglePinTab" => json!({"kind": "background", "command": "toggle-pin"}),
        "toggleMuteTab" => json!({"kind": "background", "command": "toggle-mute"}),
        "removeTab" => json!({"kind": "background", "command": "remove-tab"}),
        "restoreTab" => json!({"kind": "background", "command": "restore-tab"}),
        "moveTabToNewWindow" => json!({"kind": "background", "command": "move-to-new-window"}),
        "closeTabsOnLeft" => json!({"kind": "background", "command": "close-tabs-left"}),
        "closeTabsOnRight" => json!({"kind": "background", "command": "close-tabs-right"}),
        "closeOtherTabs" => json!({"kind": "background", "command": "close-other-tabs"}),
        "moveTabLeft" => json!({"kind": "background", "command": "move-tab-left"}),
        "moveTabRight" => json!({"kind": "background", "command": "move-tab-right"}),
        "setZoom" => json!({"kind": "background", "command": "set-zoom"}),
        "zoomIn" => json!({"kind": "background", "command": "zoom-in"}),
        "zoomOut" => json!({"kind": "background", "command": "zoom-out"}),
        "zoomReset" => json!({"kind": "background", "command": "zoom-reset"}),
        "passNextKey" => json!({"kind": "pass-next-key"}),
        "toggleViewSource" => json!({"kind": "view-source"}),
        "showHelp" => json!({"kind": "help"}),
        _ => {
            if bkg {
                json!({"kind": "background", "command": cmd_name})
            } else {
                json!({"kind": "none"})
            }
        }
    }
}

fn visual_effect(cmd_name: &str, _count: i64, _entry: Option<&CommandEntry>) -> Value {
    match cmd_name {
        "j" | "scrollDown" => json!({"kind": "visual-move", "direction": "forward", "granularity": "line"}),
        "k" | "scrollUp" => json!({"kind": "visual-move", "direction": "backward", "granularity": "line"}),
        "h" | "scrollLeft" => json!({"kind": "visual-move", "direction": "backward", "granularity": "character"}),
        "l" | "scrollRight" => json!({"kind": "visual-move", "direction": "forward", "granularity": "character"}),
        "w" => json!({"kind": "visual-move", "direction": "forward", "granularity": "vimword"}),
        "b" => json!({"kind": "visual-move", "direction": "backward", "granularity": "vimword"}),
        "e" => json!({"kind": "visual-move", "direction": "forward", "granularity": "vimword-end"}),
        "0" => json!({"kind": "visual-move", "direction": "backward", "granularity": "lineboundary"}),
        "$" => json!({"kind": "visual-move", "direction": "forward", "granularity": "lineboundary"}),
        "gg" | "scrollToTop" => json!({"kind": "visual-move", "direction": "backward", "granularity": "document-boundary"}),
        "G" | "scrollToBottom" => json!({"kind": "visual-move", "direction": "forward", "granularity": "document-boundary"}),
        "y" => json!({"kind": "visual-copy", "stay": "exit-visual"}),
        "Esc" => json!({"kind": "exit-visual"}),
        _ => json!({"kind": "stay-visual"}),
    }
}

pub fn effect_mode(effect: &Value) -> String {
    match effect.get("kind").and_then(Value::as_str).unwrap_or("") {
        "hints" | "hints-general" | "hints-queue" | "hints-download"
        | "hints-incognito" | "hints-copy-url" => "hints".to_string(),
        "create-mark" | "goto-mark" => "normal".to_string(),
        "enter-visual" => {
            effect.get("mode").and_then(Value::as_str).unwrap_or("visual").to_string()
        }
        "insert-mode" => "insert".to_string(),
        _ => "normal".to_string(),
    }
}

pub fn key_name(event_key: &str) -> String {
    match event_key {
        "Escape" => "Esc".to_string(),
        "ArrowLeft" => "left".to_string(),
        "ArrowUp" => "up".to_string(),
        "ArrowRight" => "right".to_string(),
        "ArrowDown" => "down".to_string(),
        " " => "space".to_string(),
        "\n" => "enter".to_string(),
        _ if event_key.len() == 1 => event_key.to_string(),
        _ => event_key.to_string().to_lowercase(),
    }
}