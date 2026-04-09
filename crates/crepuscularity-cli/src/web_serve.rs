//! `crepus web serve` — hot-reload dev server for `.crepus` templates.
//!
//! # How it works
//!
//! 1. Walks `site_dir` and loads all `*.crepus` files into an in-memory
//!    **virtual file map** (`Arc<RwLock<HashMap<String, String>>>`).
//! 2. A `notify` watcher fires on every `.crepus` change; a debounce thread
//!    batches events within a 50 ms window then updates the virtual file map
//!    and bumps a generation counter.
//! 3. A plain `std::net::TcpListener` accept-loop spawns one thread per
//!    connection — fine for a dev server with ≤ a handful of browser tabs.
//! 4. `GET /dev-reload` is a **Server-Sent Events** endpoint: the browser
//!    keeps the connection open; when the generation counter changes the
//!    server sends `data: reload\n\n` and closes the connection. The browser's
//!    `EventSource` auto-reconnects on the next page load.
//! 5. Every served HTML page gets a small `<script>` injected before `</body>`
//!    that opens the `/dev-reload` SSE stream.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crepuscularity_core::context::TemplateContext;
use crepuscularity_core::preprocess::google_fonts_head_markup;
use crepuscularity_web::render_from_files;

use crate::web::merged_site_google_fonts;

// ── Options ──────────────────────────────────────────────────────────────────

/// Configuration for `crepus web serve`.
pub struct ServeOptions {
    /// Root directory containing `.crepus` source files.
    pub site_dir: PathBuf,
    /// TCP port to listen on (default 4000).
    pub port: u16,
    /// Entry-point template relative to `site_dir` (default `"index.crepus"`).
    pub entry: String,
}

// ── Hot-reload script ────────────────────────────────────────────────────────

const RELOAD_SCRIPT: &str = "<script>
(function(){
  function showErrorOverlay(msg){
    var el=document.getElementById('__crepus_err__');
    if(!el){
      el=document.createElement('div');
      el.id='__crepus_err__';
      el.style.cssText='position:fixed;inset:0;z-index:99999;background:rgba(10,0,0,0.93);color:#ff6e6e;font-family:monospace;font-size:14px;padding:40px;white-space:pre-wrap;overflow-y:auto;display:flex;flex-direction:column;gap:12px';
      document.body.appendChild(el);
    }
    while(el.firstChild){el.removeChild(el.firstChild);}
    var h=document.createElement('strong');
    h.style.cssText='font-size:20px;color:#ff9999';
    h.textContent='\u{26a0} Crepus error';
    var b=document.createElement('pre');
    b.style.cssText='margin:0;white-space:pre-wrap;color:#ff6e6e';
    b.textContent=msg;
    var hint=document.createElement('span');
    hint.style.cssText='color:#555;font-size:12px';
    hint.textContent='Fix the template and save to dismiss.';
    el.appendChild(h);el.appendChild(b);el.appendChild(hint);
  }
  function dismissErrorOverlay(){var el=document.getElementById('__crepus_err__');if(el)el.remove();}
  function showToast(text){
    var t=document.createElement('div');
    t.style.cssText='position:fixed;bottom:16px;right:16px;z-index:99998;background:#18181b;color:#a1a1aa;font-family:monospace;font-size:12px;padding:8px 14px;border-radius:6px;border:1px solid #3f3f46;pointer-events:none;transition:opacity 0.3s';
    t.textContent=text;
    document.body.appendChild(t);
    setTimeout(function(){t.style.opacity='0';setTimeout(function(){t.remove();},300);},1800);
  }
  var es=new EventSource('/dev-reload');
  es.addEventListener('crepus-reload',function(e){
    dismissErrorOverlay();
    var info={};try{info=JSON.parse(e.data);}catch(_){}
    showToast('\u{21bb} '+(info.file||'template')+' updated');
    setTimeout(function(){location.reload();},150);
  });
  es.addEventListener('crepus-error',function(e){
    showErrorOverlay(e.data.replace(/\\\\n/g,'\\n'));
  });
  es.onmessage=function(e){if(e.data==='reload'){dismissErrorOverlay();location.reload();}};
  es.onerror=function(){};
})();
</script>";

// ── Startup validation ───────────────────────────────────────────────────────

/// Walk `site_dir` and parse every `.crepus` file, printing pass/fail per file.
fn validate_templates(site_dir: &std::path::Path) {
    use crepuscularity_core::parser::parse_template;
    let mut found = false;
    for entry in walkdir::WalkDir::new(site_dir)
        .into_iter()
        .flatten()
        .filter(|e| {
            !e.file_type().is_dir()
                && e.path().extension().and_then(|x| x.to_str()) == Some("crepus")
        })
    {
        found = true;
        let rel = entry.path().strip_prefix(site_dir).unwrap_or(entry.path());
        match std::fs::read_to_string(entry.path()).map(|s| parse_template(&s)) {
            Ok(Ok(_)) => eprintln!("  {} {}", console::style("✓").green(), rel.display()),
            Ok(Err(e)) => eprintln!("  {} {} — {}", console::style("✗").red(), rel.display(), e),
            Err(e) => eprintln!(
                "  {} {} — read error: {}",
                console::style("✗").red(),
                rel.display(),
                e
            ),
        }
    }
    if !found {
        eprintln!("  {} no .crepus files found", console::style("⚠").yellow());
    }
}

// ── Public entry point ───────────────────────────────────────────────────────

/// Run the dev server, blocking until the process is killed.
pub fn run(opts: ServeOptions) {
    let site_dir = opts.site_dir.clone();

    // Validate all templates at startup.
    eprintln!("\n  {} validating templates…", console::style("→").dim());
    validate_templates(&site_dir);

    // Build initial virtual file map.
    let vfm: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    load_all_crepus(&site_dir, &vfm);

    // Generation counter — bumped on every hot reload.
    let generation: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));

    // Last SSE message — updated by the watcher, read by SSE long-poll handlers.
    let last_sse_msg: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));

    // Spawn watcher + debounce.
    start_watcher(
        site_dir.clone(),
        Arc::clone(&vfm),
        Arc::clone(&generation),
        Arc::clone(&last_sse_msg),
    );

    // Bind TCP listener.
    let addr = format!("127.0.0.1:{}", opts.port);
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("crepus web serve: cannot bind {addr}: {e}");
        std::process::exit(1);
    });

    eprintln!(
        "\n  {} crepus web serve\n  {} http://localhost:{}\n  {} edit .crepus files for instant reload\n",
        console::style("▶").green().bold(),
        console::style("→").dim(),
        opts.port,
        console::style("→").dim(),
    );

    let entry = opts.entry.clone();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let vfm = Arc::clone(&vfm);
                let gen = Arc::clone(&generation);
                let sse_msg = Arc::clone(&last_sse_msg);
                let entry = entry.clone();
                let site_dir = site_dir.clone();
                std::thread::spawn(move || {
                    handle_connection(s, vfm, gen, sse_msg, &entry, &site_dir);
                });
            }
            Err(e) => {
                eprintln!("crepus web serve: accept error: {e}");
            }
        }
    }
}

// ── Virtual file map ─────────────────────────────────────────────────────────

/// Walk `site_dir` recursively and load every `*.crepus` file into `vfm`.
fn load_all_crepus(site_dir: &Path, vfm: &Arc<RwLock<HashMap<String, String>>>) {
    let mut map = vfm.write().unwrap();
    load_dir_recursive(site_dir, site_dir, &mut map);
}

fn load_dir_recursive(root: &Path, dir: &Path, map: &mut HashMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_dir_recursive(root, &path, map);
        } else if path.extension().is_some_and(|e| e == "crepus") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let key = relative_key(root, &path);
                map.insert(key, content);
            }
        }
    }
}

/// Return a forward-slash relative path string from `root` to `abs`.
fn relative_key(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .unwrap_or(abs)
        .to_string_lossy()
        .replace('\\', "/")
}

// ── SSE message builder ──────────────────────────────────────────────────────

/// Try to parse `path` and return the appropriate SSE event string.
fn build_sse_message(path: &Path) -> String {
    use crepuscularity_core::parser::parse_template;
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    match std::fs::read_to_string(path).map(|s| parse_template(&s)) {
        Ok(Ok(_)) => format!(
            "event: crepus-reload\ndata: {{\"file\":\"{}\",\"ts\":{}}}\n\n",
            filename, ts
        ),
        Ok(Err(e)) => {
            let escaped = e.replace('\n', "\\n");
            format!("event: crepus-error\ndata: {escaped}\n\n")
        }
        Err(e) => {
            let escaped = e.to_string().replace('\n', "\\n");
            format!("event: crepus-error\ndata: read error: {escaped}\n\n")
        }
    }
}

// ── File watcher + debounce ──────────────────────────────────────────────────

fn start_watcher(
    site_dir: PathBuf,
    vfm: Arc<RwLock<HashMap<String, String>>>,
    generation: Arc<AtomicU64>,
    last_sse_msg: Arc<RwLock<String>>,
) {
    let (tx, rx) = mpsc::channel::<PathBuf>();

    // notify watcher thread.
    let watch_dir = site_dir.clone();
    std::thread::spawn(move || {
        let tx2 = tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                ) {
                    for path in &event.paths {
                        if path.extension().is_some_and(|e| e == "crepus") {
                            let _ = tx2.send(path.clone());
                        }
                    }
                }
            }
        })
        .expect("crepus web serve: cannot create file watcher");

        watcher
            .watch(&watch_dir, RecursiveMode::Recursive)
            .expect("crepus web serve: cannot watch site dir");

        // Keep watcher alive.
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    });

    // Debounce thread: batch events within 50 ms then apply.
    std::thread::spawn(move || {
        let mut pending: Vec<PathBuf> = Vec::new();
        let mut last_event = Instant::now();

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(p) => {
                    pending.push(p);
                    last_event = Instant::now();
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !pending.is_empty() && last_event.elapsed() >= Duration::from_millis(50) {
                        // Apply all pending changes.
                        let mut map = vfm.write().unwrap();
                        let mut changed = 0usize;
                        // Track the last path for SSE message composition.
                        let mut last_changed_path: Option<PathBuf> = None;
                        for path in pending.drain(..) {
                            let key = relative_key(&site_dir, &path);
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    map.insert(key, content);
                                    changed += 1;
                                    last_changed_path = Some(path);
                                }
                                Err(_) => {
                                    // File may have been deleted.
                                    map.remove(&key);
                                    changed += 1;
                                }
                            }
                        }
                        drop(map);
                        if changed > 0 {
                            // Compose the SSE message for this reload event.
                            let sse = if let Some(ref path) = last_changed_path {
                                build_sse_message(path)
                            } else {
                                "data: reload\n\n".to_string()
                            };
                            if let Ok(mut msg) = last_sse_msg.write() {
                                *msg = sse;
                            }
                            generation.fetch_add(1, Ordering::Release);
                            let _span = tracing::info_span!("hot_reload", changed_files = changed)
                                .entered();
                            tracing::info!(changed_files = changed, "hot reloaded");
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

// ── HTTP connection handler ──────────────────────────────────────────────────

fn handle_connection(
    mut stream: TcpStream,
    vfm: Arc<RwLock<HashMap<String, String>>>,
    generation: Arc<AtomicU64>,
    last_sse_msg: Arc<RwLock<String>>,
    entry: &str,
    site_dir: &Path,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

    // Read the request line (first line only).
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };
    let request = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let first_line = request.lines().next().unwrap_or("");

    // Parse method + path.
    let mut parts = first_line.splitn(3, ' ');
    let method = parts.next().unwrap_or("GET");
    let raw_path = parts.next().unwrap_or("/");
    // Strip query string.
    let path = raw_path.split('?').next().unwrap_or(raw_path);

    let _span = tracing::info_span!("dev_request", %method, %path).entered();

    match (method, path) {
        ("GET", "/dev-reload") => {
            long_poll_sse(stream, generation, last_sse_msg);
        }

        ("GET", "/" | "/index.html") => {
            serve_template(&mut stream, &vfm, entry, site_dir);
        }

        ("GET", p) if p.ends_with(".crepus") || p.ends_with(".html") => {
            let template_key = p.trim_start_matches('/');
            let template_key = if template_key.ends_with(".html") {
                template_key.replace(".html", ".crepus")
            } else {
                template_key.to_string()
            };
            serve_template(&mut stream, &vfm, &template_key, site_dir);
        }

        ("GET", p) if p.starts_with("/static/") => {
            serve_static_file(&mut stream, p, site_dir);
        }

        _ => {
            let body = "<html><body><h1>404 Not Found</h1></body></html>";
            let resp = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    }
}

fn serve_template(
    stream: &mut TcpStream,
    vfm: &Arc<RwLock<HashMap<String, String>>>,
    entry: &str,
    site_dir: &Path,
) {
    let files = vfm.read().unwrap().clone();
    let ctx = TemplateContext::new();

    let result = render_from_files(&files, entry, &ctx);

    let mut html = match result {
        Ok(h) => h,
        Err(e) => format!(
            "<html><body style='font-family:monospace;padding:2rem'>\
             <h2 style='color:#ef4444'>Template error</h2><pre>{}</pre></body></html>",
            html_escape(&e)
        ),
    };

    let fonts = merged_site_google_fonts(site_dir, &files);
    let font_markup = google_fonts_head_markup(&fonts);
    if !font_markup.is_empty() {
        html = format!("{font_markup}\n{html}");
    }

    // Inject hot-reload script.
    if let Some(pos) = html.rfind("</body>") {
        html.insert_str(pos, RELOAD_SCRIPT);
    } else {
        html.push_str(RELOAD_SCRIPT);
    }

    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn serve_static_file(stream: &mut TcpStream, url_path: &str, site_dir: &Path) {
    let rel = url_path.trim_start_matches('/');
    let file_path = site_dir.join(rel);

    match std::fs::read(&file_path) {
        Ok(bytes) => {
            let mime = guess_mime(file_path.extension().and_then(|e| e.to_str()).unwrap_or(""));
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n\r\n",
                bytes.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&bytes);
        }
        Err(_) => {
            let body = b"Not found";
            let _ = stream.write_all(
                format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                )
                .as_bytes(),
            );
            let _ = stream.write_all(body);
        }
    }
}

/// Server-Sent Events long-poll: blocks until the generation counter changes,
/// then sends a typed SSE message (crepus-reload or crepus-error) and closes.
fn long_poll_sse(
    mut stream: TcpStream,
    generation: Arc<AtomicU64>,
    last_sse_msg: Arc<RwLock<String>>,
) {
    let headers =
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
    if stream.write_all(headers.as_bytes()).is_err() {
        return;
    }
    let _ = stream.flush();

    let _ = stream.set_read_timeout(Some(Duration::from_secs(60)));
    let start_gen = generation.load(Ordering::Acquire);

    loop {
        std::thread::sleep(Duration::from_millis(100));
        if generation.load(Ordering::Acquire) != start_gen {
            let msg = last_sse_msg
                .read()
                .map(|g| g.clone())
                .unwrap_or_else(|_| "data: reload\n\n".to_string());
            let msg = if msg.is_empty() {
                "data: reload\n\n".to_string()
            } else {
                msg
            };
            let _ = stream.write_all(msg.as_bytes());
            let _ = stream.flush();
            break;
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn guess_mime(ext: &str) -> &'static str {
    match ext {
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        _ => "application/octet-stream",
    }
}
