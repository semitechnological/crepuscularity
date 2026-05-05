use crepuscularity_webext::wasm::{bookmarks, history, tabs};
use serde_json::{json, Value};

use crate::settings::UserSettings;

pub struct SearchEngines {
    pub google: String,
    pub custom: Vec<(String, String, String)>,
}

impl SearchEngines {
    pub fn from_settings(settings: &UserSettings) -> Self {
        let custom = settings
            .parse_search_engines()
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

#[derive(Debug, Clone)]
pub enum VomnibarMode {
    Full,
    Bookmarks,
    Tabs,
}

pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub title: String,
    pub url: String,
    pub kind: String,
    pub relevance: f64,
}

pub async fn query_vomnibar(query: &str, mode: VomnibarMode) -> Result<CompletionResult, String> {
    let mut items = Vec::new();

    match mode {
        VomnibarMode::Full => {
            let bm = query_bookmarks(query).await?;
            items.extend(bm);
            let hist = query_history(query).await?;
            items.extend(hist);
            let tabs_items = query_tabs(query).await?;
            items.extend(tabs_items);
        }
        VomnibarMode::Bookmarks => {
            items = query_bookmarks(query).await?;
        }
        VomnibarMode::Tabs => {
            items = query_tabs(query).await?;
        }
    }

    items = scored_items(items, query);

    Ok(CompletionResult {
        items,
        prompt: query.to_string(),
    })
}

async fn query_bookmarks(query: &str) -> Result<Vec<CompletionItem>, String> {
    let results = bookmarks::get_recent(200)
        .await
        .map_err(|e| format!("bookmarks: {}", e))?;
    let flat = bookmarks::flatten_tree(&results);
    Ok(flat
        .into_iter()
        .filter_map(|node| {
            Some(CompletionItem {
                title: node.title,
                url: node.url?,
                kind: "bookmark".to_string(),
                relevance: 0.0,
            })
        })
        .collect())
}

async fn query_history(query: &str) -> Result<Vec<CompletionItem>, String> {
    let results = history::search(&history::HistorySearchQuery {
        text: query.to_string(),
        max_results: Some(200),
        ..Default::default()
    })
    .await
    .map_err(|e| format!("history: {}", e))?;
    Ok(results
        .into_iter()
        .filter_map(|item| {
            Some(CompletionItem {
                title: item.title.unwrap_or_default(),
                url: item.url?,
                kind: "history".to_string(),
                relevance: 0.0,
            })
        })
        .collect())
}

async fn query_tabs(_query: &str) -> Result<Vec<CompletionItem>, String> {
    let all = tabs::query(&tabs::QueryInfo {
        ..Default::default()
    })
    .await
    .map_err(|e| format!("tabs: {}", e))?;
    Ok(all
        .into_iter()
        .filter_map(|tab| {
            Some(CompletionItem {
                title: tab.title.unwrap_or_default(),
                url: tab.url?,
                kind: "tab".to_string(),
                relevance: 0.0,
            })
        })
        .collect())
}

pub fn scored_items(items: Vec<CompletionItem>, query: &str) -> Vec<CompletionItem> {
    let query_lower = query.to_lowercase();
    let mut scored: Vec<CompletionItem> = items
        .into_iter()
        .map(|mut item| {
            let title_lower = item.title.to_lowercase();
            let url_lower = item.url.to_lowercase();
            let mut score = 0.0;

            let terms: Vec<&str> = query_lower.split_whitespace().collect();
            for term in &terms {
                if title_lower.starts_with(term) {
                    score += 2.0;
                } else if title_lower.contains(term) {
                    score += 1.0;
                }
                if url_lower.starts_with(term) {
                    score += 1.5;
                } else if url_lower.contains(term) {
                    score += 0.5;
                }
            }
            if title_lower == query_lower {
                score += 3.0;
            }
            item.relevance = score;
            item
        })
        .collect();
    scored.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
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