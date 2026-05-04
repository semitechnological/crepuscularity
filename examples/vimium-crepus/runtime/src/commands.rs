use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub name: String,
    pub desc: String,
    pub group: String,
    pub advanced: bool,
    pub background: bool,
    pub top_frame: bool,
    pub no_repeat: bool,
    pub repeat_limit: Option<i64>,
    pub options: Value,
}

pub fn all_commands() -> Vec<CommandEntry> {
    vec![
        c("scrollDown", "Scroll down", "navigation", false, false, false, false, None, json!({})),
        c("scrollUp", "Scroll up", "navigation", false, false, false, false, None, json!({})),
        c("scrollToTop", "Scroll to the top of the page", "navigation", false, false, false, false, None, json!({})),
        c("scrollToBottom", "Scroll to the bottom of the page", "navigation", false, false, false, false, None, json!({})),
        c("scrollPageDown", "Scroll a half page down", "navigation", false, false, false, false, None, json!({})),
        c("scrollPageUp", "Scroll a half page up", "navigation", false, false, false, false, None, json!({})),
        c("scrollFullPageDown", "Scroll a full page down", "navigation", false, false, false, false, None, json!({})),
        c("scrollFullPageUp", "Scroll a full page up", "navigation", false, false, false, false, None, json!({})),
        c("scrollLeft", "Scroll left", "navigation", false, false, false, false, None, json!({})),
        ca("scrollRight", "Scroll right", "navigation", true, false, false, false, None, json!({})),
        ca("scrollToLeft", "Scroll all the way to the left", "navigation", true, false, false, false, None, json!({})),
        ca("scrollToRight", "Scroll all the way to the right", "navigation", true, false, false, false, None, json!({})),
        c("reload", "Reload the page", "navigation", false, true, false, false, None, json!({"hard": "Perform a hard reload, forcing the browser to bypass its cache."})),
        c("copyCurrentUrl", "Copy the current URL to the clipboard", "navigation", false, false, false, true, None, json!({})),
        ca("openCopiedUrlInCurrentTab", "Open the clipboard's URL in the current tab", "navigation", true, false, false, true, None, json!({})),
        c("openCopiedUrlInNewTab", "Open the clipboard's URL in a new tab", "navigation", false, false, false, true, None, json!({})),
        ca("goUp", "Go up the URL hierarchy", "navigation", true, false, false, false, None, json!({})),
        ca("goToRoot", "Go to the root of current URL hierarchy", "navigation", true, false, false, false, None, json!({})),
        c("enterInsertMode", "Enter insert mode", "navigation", false, false, false, true, None, json!({})),
        c("enterVisualMode", "Enter visual mode", "navigation", false, false, false, true, None, json!({})),
        ca("enterVisualLineMode", "Enter visual line mode", "navigation", true, false, false, true, None, json!({})),
        ca("passNextKey", "Pass the next key to the page", "navigation", true, false, false, false, None, json!({})),
        c("focusInput", "Focus the first text input on the page", "navigation", false, false, false, false, None, json!({})),
        ca("LinkHints.activateMode", "Open a link hints for copying/editing", "navigation", true, false, false, false, None, json!({})),
        c("LinkHints.activateModeToOpenInNewTab", "Open a link in a new tab", "navigation", false, false, false, false, None, json!({})),
        ca("LinkHints.activateModeToOpenInNewForegroundTab", "Open a link in a new tab & switch to it", "navigation", true, false, false, false, None, json!({})),
        ca("LinkHints.activateModeWithQueue", "Open multiple links in a new tab", "navigation", true, false, false, true, None, json!({})),
        ca("LinkHints.activateModeToDownloadLink", "Download link url", "navigation", true, false, false, false, None, json!({})),
        ca("LinkHints.activateModeToOpenIncognito", "Open a link in incognito window", "navigation", true, false, false, false, None, json!({})),
        ca("LinkHints.activateModeToCopyLinkUrl", "Copy a link URL to the clipboard", "navigation", true, false, false, false, None, json!({})),
        ca("goPrevious", "Follow the link labeled previous or <", "navigation", true, false, false, true, None, json!({})),
        ca("goNext", "Follow the link labeled next or >", "navigation", true, false, false, true, None, json!({})),
        ca("nextFrame", "Select the next frame on the page", "navigation", true, true, false, false, None, json!({})),
        ca("mainFrame", "Select the page's main/top frame", "navigation", true, false, true, true, None, json!({})),
        ca("Marks.activateCreateMode", "Create a new mark", "navigation", true, false, false, true, None, json!({})),
        ca("Marks.activateGotoMode", "Jump to a mark", "navigation", true, false, false, true, None, json!({})),
        c("Vomnibar.activate", "Open URL, bookmark or history entry", "vomnibar", false, false, true, false, None, json!({})),
        c("Vomnibar.activateInNewTab", "Open URL, bookmark or history entry in a new tab", "vomnibar", false, false, true, false, None, json!({})),
        ca("Vomnibar.activateBookmarks", "Open a bookmark", "vomnibar", true, false, true, false, None, json!({})),
        ca("Vomnibar.activateBookmarksInNewTab", "Open a bookmark in a new tab", "vomnibar", true, false, true, false, None, json!({})),
        ca("Vomnibar.activateTabSelection", "Search through your open tabs", "vomnibar", true, false, true, false, None, json!({})),
        ca("Vomnibar.activateEditUrl", "Edit the current URL", "vomnibar", true, false, true, false, None, json!({})),
        ca("Vomnibar.activateEditUrlInNewTab", "Edit the current URL and open in a new tab", "vomnibar", true, false, true, false, None, json!({})),
        c("enterFindMode", "Enter find mode", "find", false, false, false, true, None, json!({})),
        c("performFind", "Cycle forward to the next find match", "find", false, false, false, false, None, json!({})),
        c("performBackwardsFind", "Cycle backward to the previous find match", "find", false, false, false, false, None, json!({})),
        ca("findSelected", "Find the selected text", "find", true, false, false, false, None, json!({})),
        ca("findSelectedBackwards", "Find the selected text, searching backwards", "find", true, false, false, false, None, json!({})),
        c("goBack", "Go back in history", "history", false, false, false, false, None, json!({})),
        c("goForward", "Go forward in history", "history", false, false, false, false, None, json!({})),
        c("createTab", "Create new tab", "tabs", false, true, false, false, Some(20), json!({})),
        c("previousTab", "Go one tab left", "tabs", false, true, false, false, None, json!({})),
        c("nextTab", "Go one tab right", "tabs", false, true, false, false, None, json!({})),
        ca("visitPreviousTab", "Go to previously-visited tab", "tabs", true, true, false, false, None, json!({})),
        c("firstTab", "Go to the first tab", "tabs", false, true, false, false, None, json!({})),
        c("lastTab", "Go to the last tab", "tabs", false, true, false, false, None, json!({})),
        ca("duplicateTab", "Duplicate current tab", "tabs", true, true, false, false, Some(20), json!({})),
        ca("togglePinTab", "Pin or unpin current tab", "tabs", true, true, false, false, None, json!({})),
        ca("toggleMuteTab", "Mute or unmute current tab", "tabs", true, true, false, true, None, json!({})),
        c("removeTab", "Close current tab", "tabs", false, true, false, false, Some(25), json!({})),
        ca("restoreTab", "Restore closed tab", "tabs", true, true, false, false, Some(20), json!({})),
        ca("moveTabToNewWindow", "Move tab to new window", "tabs", true, true, false, false, None, json!({})),
        ca("closeTabsOnLeft", "Close tabs on the left", "tabs", true, true, false, false, None, json!({})),
        ca("closeTabsOnRight", "Close tabs on the right", "tabs", true, true, false, false, None, json!({})),
        ca("closeOtherTabs", "Close all other tabs", "tabs", true, true, false, true, None, json!({})),
        ca("moveTabLeft", "Move tab to the left", "tabs", true, true, false, false, None, json!({})),
        ca("moveTabRight", "Move tab to the right", "tabs", true, true, false, false, None, json!({})),
        ca("setZoom", "Set zoom", "tabs", true, true, false, false, None, json!({})),
        ca("zoomIn", "Zoom in", "tabs", true, true, false, false, None, json!({})),
        ca("zoomOut", "Zoom out", "tabs", true, true, false, false, None, json!({})),
        ca("zoomReset", "Reset zoom", "tabs", true, true, false, false, None, json!({})),
        ca("toggleViewSource", "View page source", "misc", true, false, false, true, None, json!({})),
        c("showHelp", "Show help", "misc", false, false, true, true, None, json!({})),
    ]
}

fn c(
    name: &str,
    desc: &str,
    group: &str,
    advanced: bool,
    background: bool,
    top_frame: bool,
    no_repeat: bool,
    repeat_limit: Option<i64>,
    options: Value,
) -> CommandEntry {
    CommandEntry {
        name: name.to_string(),
        desc: desc.to_string(),
        group: group.to_string(),
        advanced,
        background,
        top_frame,
        no_repeat,
        repeat_limit,
        options,
    }
}

fn ca(
    name: &str,
    desc: &str,
    group: &str,
    advanced: bool,
    background: bool,
    top_frame: bool,
    no_repeat: bool,
    repeat_limit: Option<i64>,
    options: Value,
) -> CommandEntry {
    let mut entry = c(name, desc, group, advanced, background, top_frame, no_repeat, repeat_limit, options);
    entry.advanced = advanced;
    entry
}

pub type RegistryEntry = CommandEntry;

#[derive(Debug, Clone)]
pub struct KeyMapping {
    pub key_sequence: Vec<String>,
    pub command_name: String,
    pub options: Value,
    pub registry: RegistryEntry,
}

#[derive(Debug, Clone)]
pub struct KeyMapRegistry {
pub key_to_command: BTreeMap<String, String>,
    pub key_to_registry: BTreeMap<String, RegistryEntry>,
    pub commands_by_name: HashMap<String, RegistryEntry>,
}

impl KeyMapRegistry {
    pub fn from_defaults() -> Self {
        let mut registry = KeyMapRegistry {
            key_to_command: BTreeMap::new(),
            key_to_registry: BTreeMap::new(),
            commands_by_name: HashMap::new(),
        };

        let cmds = all_commands();
        for cmd in &cmds {
            registry.commands_by_name.insert(cmd.name.clone(), cmd.clone());
        }

        let defaults = default_key_bindings();
        for (key_seq, cmd_name) in &defaults {
            let joined = key_seq.join("");
            registry.key_to_command.insert(joined.clone(), cmd_name.clone());
            if let Some(entry) = registry.commands_by_name.get(cmd_name) {
                registry.key_to_registry.insert(joined, entry.clone());
            }
        }

        registry
    }

    pub fn parse_user_mappings(&self, config_text: &str) -> HashMap<String, String> {
        let mut overrides: HashMap<String, String> = HashMap::new();

        for line in config_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with('"')
            {
                continue;
            }
            let cleaned = trimmed.replace("\\\n", " ").replace('\n', " ");
            let tokens: Vec<&str> = cleaned.split_whitespace().collect();
            if tokens.len() < 3 {
                continue;
            }
            let action = tokens[0].to_lowercase();
            match action.as_str() {
                "map" => {
                    let key_seq = tokens[1].to_string();
                    let cmd_name = tokens[2].to_string();
                    if self.commands_by_name.contains_key(&cmd_name) {
                        overrides.insert(key_seq, cmd_name);
                    }
                }
                "unmap" => {
                    let key_seq = tokens[1].to_string();
                    overrides.insert(key_seq, String::new());
                }
                "unmapall" => {
                    overrides.clear();
                }
                _ => {}
            }
        }

        overrides
    }

    pub fn resolve_command<'a>(
        &'a self,
        sequence: &str,
        user_mappings: &'a HashMap<String, String>,
    ) -> Option<(&'a str, Option<&'a RegistryEntry>)> {
        if let Some(cmd_override) = user_mappings.get(sequence) {
            if cmd_override.is_empty() {
                return None;
            }
            let entry = self.commands_by_name.get(cmd_override);
            return Some((cmd_override.as_str(), entry));
        }
        if let Some(cmd_name) = self.key_to_command.get(sequence) {
            let entry = self.commands_by_name.get(cmd_name);
            return Some((cmd_name.as_str(), entry));
        }
        None
    }

    pub fn is_prefix(&self, sequence: &str, user_mappings: &HashMap<String, String>) -> bool {
        for key in self.key_to_command.keys() {
            if key.starts_with(sequence) {
                return true;
            }
        }
        for key in user_mappings.keys() {
            if key.starts_with(sequence) && !user_mappings[key].is_empty() {
                return true;
            }
        }
        false
    }
}

fn default_key_bindings() -> Vec<(Vec<String>, String)> {
    let m = |k: &str, c: &str| (k.chars().map(|ch| ch.to_string()).collect(), c.to_string());
    vec![
        m("?", "showHelp"),
        m("j", "scrollDown"),
        m("k", "scrollUp"),
        m("h", "scrollLeft"),
        m("l", "scrollRight"),
        m("gg", "scrollToTop"),
        m("G", "scrollToBottom"),
        m("d", "scrollPageDown"),
        m("u", "scrollPageUp"),
        m("r", "reload"),
        m("gs", "toggleViewSource"),
        m("i", "enterInsertMode"),
        m("v", "enterVisualMode"),
        m("V", "enterVisualLineMode"),
        m("yy", "copyCurrentUrl"),
        m("yf", "LinkHints.activateModeToCopyLinkUrl"),
        m("p", "openCopiedUrlInCurrentTab"),
        m("P", "openCopiedUrlInNewTab"),
        m("[[", "goPrevious"),
        m("]]", "goNext"),
        m("gi", "focusInput"),
        m("f", "LinkHints.activateModeToOpenInNewTab"),
        m("F", "LinkHints.activateModeToOpenInNewForegroundTab"),
        m("gf", "nextFrame"),
        m("gF", "mainFrame"),
        m("m", "Marks.activateCreateMode"),
        m("`", "Marks.activateGotoMode"),
        m("o", "Vomnibar.activate"),
        m("O", "Vomnibar.activateInNewTab"),
        m("b", "Vomnibar.activateBookmarks"),
        m("B", "Vomnibar.activateBookmarksInNewTab"),
        m("T", "Vomnibar.activateTabSelection"),
        m("ge", "Vomnibar.activateEditUrl"),
        m("gE", "Vomnibar.activateEditUrlInNewTab"),
        m("/", "enterFindMode"),
        m("n", "performFind"),
        m("N", "performBackwardsFind"),
        m("H", "goBack"),
        m("L", "goForward"),
        m("t", "createTab"),
        m("J", "previousTab"),
        m("K", "nextTab"),
        m("gT", "previousTab"),
        m("gt", "nextTab"),
        m("^", "visitPreviousTab"),
        m("g0", "firstTab"),
        m("g$", "lastTab"),
        m("yt", "duplicateTab"),
        m("x", "removeTab"),
        m("X", "restoreTab"),
        m("W", "moveTabToNewWindow"),
        m("<<", "moveTabLeft"),
        m(">>", "moveTabRight"),
    ]
}