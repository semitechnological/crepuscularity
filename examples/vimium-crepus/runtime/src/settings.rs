use serde_json::{json, Value};
use std::collections::HashMap;

pub const VIMIUM_NEW_TAB_URL: &str = "https://vimium.github.io/new-tab/";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewTabDestination {
    BrowserNewTabPage,
    VimiumNewTabPage,
    CustomUrl,
}

impl NewTabDestination {
    pub fn as_str(&self) -> &'static str {
        match self {
            NewTabDestination::BrowserNewTabPage => "browserNewTabPage",
            NewTabDestination::VimiumNewTabPage => "vimiumNewTabPage",
            NewTabDestination::CustomUrl => "customUrl",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "browserNewTabPage" => NewTabDestination::BrowserNewTabPage,
            "customUrl" => NewTabDestination::CustomUrl,
            _ => NewTabDestination::VimiumNewTabPage,
        }
    }
}

pub fn default_settings() -> Value {
    json!({
        "scrollStepSize": 60,
        "smoothScroll": true,
        "keyMappings": "# Insert your preferred key mappings here.",
        "linkHintCharacters": "sadfjklewcmpgh",
        "linkHintNumbers": "0123456789",
        "filterLinkHints": false,
        "hideHud": false,
        "hideUpdateNotifications": false,
        "userDefinedLinkHintCss": "div > .vimiumHintMarker {\nbackground: -webkit-gradient(linear, left top, left bottom, color-stop(0%,#FFF785), color-stop(100%,#FFC542));\nborder: 1px solid #E3BE23;\n}\n\ndiv > .vimiumHintMarker span {\ncolor: black;\nfont-weight: bold;\nfont-size: 12px;\n}\n\ndiv > .vimiumHintMarker > .matchingCharacter {\n}",
        "exclusionRules": [
            { "passKeys": "", "pattern": "https?://mail.google.com/*" }
        ],
        "previousPatterns": "prev,previous,back,older,<,\u{2039},\u{2190},\u{00ab},\u{226a},<<",
        "nextPatterns": "next,more,newer,>,\u{203a},\u{2192},\u{00bb},\u{226b},>>",
        "searchUrl": "https://www.google.com/search?q=",
        "searchEngines": "w: https://www.wikipedia.org/w/index.php?title=Special:Search&search=%s Wikipedia\n\n# More examples.\ng: https://www.google.com/search?q=%s Google\nl: https://www.google.com/search?q=%s&btnI I'm feeling lucky...\ny: https://www.youtube.com/results?search_query=%s Youtube\ngm: https://www.google.com/maps?q=%s Google maps\nd: https://duckduckgo.com/?q=%s DuckDuckGo\n",
        "newTabDestination": "vimiumNewTabPage",
        "newTabCustomUrl": "",
        "openVomnibarOnNewTabPage": true,
        "grabBackFocus": false,
        "regexFindMode": false,
        "waitForEnterForFilteredHints": true,
        "helpDialog_showAdvancedCommands": false,
        "ignoreKeyboardLayout": false
    })
}

pub struct UserSettings {
    pub settings: Value,
    pub defaults: Value,
}

impl UserSettings {
    pub fn new() -> Self {
        UserSettings {
            settings: json!({}),
            defaults: default_settings(),
        }
    }

    pub fn merge(&mut self, stored: Value) {
        let defaults = default_settings();
        let mut merged = defaults.clone();
        if let Value::Object(map) = &stored {
            for (k, v) in map {
                merged[k] = v.clone();
            }
        }
        self.settings = merged;
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.settings
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or_else(|| {
                self.defaults
                    .get(key)
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
    }

    pub fn get_str(&self, key: &str) -> String {
        self.settings
            .get(key)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                self.defaults
                    .get(key)
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            })
    }

    pub fn get_int(&self, key: &str) -> i64 {
        self.settings
            .get(key)
            .and_then(Value::as_i64)
            .unwrap_or_else(|| {
                self.defaults
                    .get(key)
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
            })
    }

    pub fn get_float(&self, key: &str) -> f64 {
        self.settings
            .get(key)
            .and_then(Value::as_f64)
            .unwrap_or_else(|| {
                self.defaults
                    .get(key)
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
            })
    }

    pub fn new_tab_destination(&self) -> NewTabDestination {
        NewTabDestination::from_str(&self.get_str("newTabDestination"))
    }

    pub fn new_tab_url(&self) -> String {
        match self.new_tab_destination() {
            NewTabDestination::BrowserNewTabPage => String::new(),
            NewTabDestination::VimiumNewTabPage => VIMIUM_NEW_TAB_URL.to_string(),
            NewTabDestination::CustomUrl => {
                let url = self.get_str("newTabCustomUrl");
                if url.is_empty() {
                    VIMIUM_NEW_TAB_URL.to_string()
                } else {
                    url
                }
            }
        }
    }

    pub fn parse_search_engines(&self) -> HashMap<String, (String, String)> {
        let engines_text = self.get_str("searchEngines");
        let mut engines = HashMap::new();
        for line in engines_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(colon_pos) = trimmed.find(':') {
                let keyword = trimmed[..colon_pos].trim().to_string();
                let rest = trimmed[colon_pos + 1..].trim();
                let parts: Vec<&str> = rest.rsplitn(2, ' ').collect();
                if parts.len() == 2 {
                    let url = parts[1].trim();
                    let name = parts[0].trim();
                    engines.insert(keyword, (url.to_string(), name.to_string()));
                }
            }
        }
        engines
    }

    pub fn parse_exclusion_rules(&self) -> Vec<ExclusionRule> {
        let rules = self.settings.get("exclusionRules");
        match rules {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|r| {
                    Some(ExclusionRule {
                        pattern: r.get("pattern")?.as_str()?.to_string(),
                        pass_keys: r.get("passKeys")?.as_str().unwrap_or("").to_string(),
                    })
                })
                .collect(),
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExclusionRule {
    pub pattern: String,
    pub pass_keys: String,
}

pub fn prune_defaults(settings: &Value) -> Value {
    let defaults = default_settings();
    match settings {
        Value::Object(map) => {
            let mut pruned = serde_json::Map::new();
            for (k, v) in map {
                if let Some(dv) = defaults.get(k) {
                    if v != dv {
                        pruned.insert(k.clone(), v.clone());
                    }
                } else {
                    pruned.insert(k.clone(), v.clone());
                }
            }
            Value::Object(pruned)
        }
        _ => settings.clone(),
    }
}