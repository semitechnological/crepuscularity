use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub enum VomnibarMode {
    Full,
    Bookmarks,
    Tabs,
}

pub struct SearchEngines {
    pub google: String,
    pub custom: Vec<(String, String, String)>,
}

impl SearchEngines {
    pub fn from_settings(settings: &crate::settings::UserSettings) -> Self {
        let custom = settings.parse_search_engines()
            .into_iter()
            .map(|(k, (url, name))| (k, url, name))
            .collect();
        SearchEngines {
            google: settings.get_str("searchUrl"),
            custom,
        }
    }

pub fn resolve(&self, query: &str) -> (String, String) {
        if let Some(colon_pos) = query.find(':') {
            let keyword = &query[..colon_pos];
            if let Some((_kw, url, _name)) = self.custom.iter().find(|(k, _, _)| k == keyword) {
                let search_term = &query[colon_pos + 1..];
                let resolved = url.replace("%s", &urlencode(search_term));
                return (resolved, query.to_string());
            }
        }
        let resolved = self.google.replace("%s", &urlencode(query));
        (resolved, query.to_string())
    }

    pub fn is_search_query(query: &str) -> bool {
        !query.contains('.') && !query.contains("://") && !query.contains(' ')
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u8)
            }
        })
        .collect()
}

pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub enum CompletionItemKind {
    Url,
    Bookmark,
    History,
    Tab,
    Search,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub title: String,
    pub url: String,
    pub kind: CompletionItemKind,
    pub relevance: f64,
}

pub fn completion_for_query(query: &str, _mode: VomnibarMode) -> CompletionResult {
    CompletionResult {
        items: vec![],
        prompt: query.to_string(),
    }
}

pub fn scored_items(
    items: Vec<CompletionItem>,
    query: &str,
) -> Vec<CompletionItem> {
    let query_lower = query.to_lowercase();
    let mut scored: Vec<CompletionItem> = items
        .into_iter()
        .map(|mut item| {
            let title_lower = item.title.to_lowercase();
            let url_lower = item.url.to_lowercase();
            let mut score = 0.0;

            if title_lower.starts_with(&query_lower) {
                score += 2.0;
            } else if title_lower.contains(&query_lower) {
                score += 1.0;
            }
            if url_lower.starts_with(&query_lower) {
                score += 1.5;
            } else if url_lower.contains(&query_lower) {
                score += 0.5;
            }
            if title_lower == query_lower {
                score += 3.0;
            }
            item.relevance = score;
            item
        })
        .filter(|item| item.relevance > 0.0)
        .collect();
    scored.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(15);
    scored
}

pub fn resolve_navigable(query: &str, engines: &SearchEngines) -> Value {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return json!({"kind": "none"});
    }

    if trimmed.contains("://") {
        return json!({"kind": "url", "url": trimmed});
    }

    if let Some(colon_pos) = trimmed.find(':') {
        let keyword = &trimmed[..colon_pos];
        if engines.custom.iter().any(|(k, _, _)| k == keyword) {
            let (url, display) = engines.resolve(trimmed);
            return json!({"kind": "url", "url": url, "display": display});
        }
    }

    if SearchEngines::is_search_query(trimmed) {
        let (url, display) = engines.resolve(trimmed);
        return json!({"kind": "url", "url": url, "display": display});
    }

    let with_scheme = if !trimmed.starts_with("http") {
        format!("https://{}", trimmed)
    } else {
        trimmed.to_string()
    };

    json!({"kind": "url", "url": with_scheme})
}