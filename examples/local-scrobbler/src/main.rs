use std::collections::BTreeMap;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crepuscularity_gpui::prelude::*;
use gpui::{bounds, point, size, Application, ClickEvent, WindowOptions};

#[cfg(has_zig_scrobbler)]
extern "C" {
    fn crepus_current_track(source: i32, out: *mut u8, out_len: usize) -> i32;
}

#[cfg(not(has_zig_scrobbler))]
unsafe fn crepus_current_track(_: i32, out: *mut u8, out_len: usize) -> i32 {
    let msg = b"zig unavailable\0";
    if out_len == 0 {
        return -1;
    }
    let n = msg.len().min(out_len);
    std::ptr::copy_nonoverlapping(msg.as_ptr(), out, n);
    (n.saturating_sub(1)) as i32
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Spotify,
    AppleMusic,
}

impl Source {
    fn id(self) -> i32 {
        match self {
            Self::Spotify => 0,
            Self::AppleMusic => 1,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Spotify => "Spotify",
            Self::AppleMusic => "Apple Music",
        }
    }
}

#[derive(Clone)]
struct Track {
    source: Source,
    artist: String,
    title: String,
    album: String,
    state: String,
}

struct ScrobblerApp {
    source: Source,
    track: Option<Track>,
    status: String,
    last_response: String,
}

impl ScrobblerApp {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            source: Source::Spotify,
            track: None,
            status: "Ready".to_string(),
            last_response:
                "Set LASTFM_API_KEY, LASTFM_API_SECRET, and LASTFM_SESSION_KEY to scrobble."
                    .to_string(),
        }
    }

    fn use_spotify(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.source = Source::Spotify;
        self.refresh_current();
        cx.notify();
    }

    fn use_apple_music(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.source = Source::AppleMusic;
        self.refresh_current();
        cx.notify();
    }

    fn refresh(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.refresh_current();
        cx.notify();
    }

    fn now_playing(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.refresh_current();
        if let Some(track) = self.track.clone() {
            match lastfm_call("track.updateNowPlaying", &track, None) {
                Ok(body) => {
                    self.status = "Now playing sent".to_string();
                    self.last_response = body;
                }
                Err(err) => {
                    self.status = "Now playing failed".to_string();
                    self.last_response = err;
                }
            }
        }
        cx.notify();
    }

    fn scrobble(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.refresh_current();
        if let Some(track) = self.track.clone() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs().to_string())
                .unwrap_or_else(|_| "0".to_string());
            match lastfm_call("track.scrobble", &track, Some(timestamp)) {
                Ok(body) => {
                    self.status = "Scrobble sent".to_string();
                    self.last_response = body;
                }
                Err(err) => {
                    self.status = "Scrobble failed".to_string();
                    self.last_response = err;
                }
            }
        }
        cx.notify();
    }

    fn refresh_current(&mut self) {
        match current_track(self.source) {
            Ok(track) => {
                self.status = format!("Loaded {}", track.source.label());
                self.last_response = format!("{} - {}", track.artist, track.title);
                self.track = Some(track);
            }
            Err(err) => {
                self.status = "No playable track".to_string();
                self.last_response = err;
                self.track = None;
            }
        }
    }
}

impl Render for ScrobblerApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source = self.source.label();
        let status = self.status.clone();
        let title = self
            .track
            .as_ref()
            .map(|track| track.title.clone())
            .unwrap_or_else(|| "No track loaded".to_string());
        let artist = self
            .track
            .as_ref()
            .map(|track| track.artist.clone())
            .unwrap_or_else(|| "Open Spotify or Music, then refresh.".to_string());
        let album = self
            .track
            .as_ref()
            .map(|track| track.album.clone())
            .unwrap_or_else(|| "Local macOS player bridge is provided by Zig.".to_string());
        let player_state = self
            .track
            .as_ref()
            .map(|track| track.state.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let last_response = self.last_response.clone();
        let is_spotify = self.source == Source::Spotify;
        let is_apple_music = self.source == Source::AppleMusic;

        view! {r#"
            div w-full h-full bg-zinc-950 text-white flex justify-center items-center font-['Helvetica']
                div w-[720px] flex flex-col gap-6
                    div flex flex-col gap-2
                        div text-4xl font-bold
                            "Local Scrobbler"
                        div text-base text-zinc-400
                            "{source} / {status}"

                    div bg-zinc-900 border border-zinc-800 rounded-lg p-5 flex flex-col gap-3
                        div text-sm uppercase text-zinc-500 font-bold
                            "{player_state}"
                        div text-3xl font-bold
                            "{title}"
                        div text-xl text-zinc-300
                            "{artist}"
                        div text-sm text-zinc-500
                            "{album}"

                    div flex gap-2
                        button px-4 py-2 rounded-lg border border-zinc-700 @click=use_spotify
                            if {is_spotify}
                                "Spotify on"
                            else
                                "Spotify"
                        button px-4 py-2 rounded-lg border border-zinc-700 @click=use_apple_music
                            if {is_apple_music}
                                "Apple Music on"
                            else
                                "Apple Music"
                        button px-4 py-2 rounded-lg bg-zinc-800 @click=refresh
                            "Refresh"

                    div flex gap-2
                        button px-4 py-2 rounded-lg bg-emerald-500 text-black font-bold @click=now_playing
                            "Now Playing"
                        button px-4 py-2 rounded-lg bg-sky-500 text-black font-bold @click=scrobble
                            "Scrobble"

                    div bg-black border border-zinc-800 rounded-lg p-4 text-sm text-zinc-400 leading-relaxed
                        "{last_response}"
        "#}
    }
}

fn current_track(source: Source) -> Result<Track, String> {
    let mut buf = [0u8; 2048];
    let n = unsafe { crepus_current_track(source.id(), buf.as_mut_ptr(), buf.len()) };
    if n <= 0 {
        return Err("Zig media bridge returned no track.".to_string());
    }
    let raw = String::from_utf8_lossy(&buf[..n as usize])
        .trim()
        .to_string();
    if raw == "stopped" || raw == "unavailable" || raw == "zig unavailable" {
        return Err(raw);
    }
    let mut parts = raw.split('\t');
    let artist = parts.next().unwrap_or_default().trim().to_string();
    let title = parts.next().unwrap_or_default().trim().to_string();
    let album = parts.next().unwrap_or_default().trim().to_string();
    let state = parts.next().unwrap_or_default().trim().to_string();
    if artist.is_empty() || title.is_empty() {
        return Err(raw);
    }
    Ok(Track {
        source,
        artist,
        title,
        album,
        state,
    })
}

fn lastfm_call(method: &str, track: &Track, timestamp: Option<String>) -> Result<String, String> {
    let api_key = std::env::var("LASTFM_API_KEY").map_err(|_| "LASTFM_API_KEY is missing")?;
    let api_secret =
        std::env::var("LASTFM_API_SECRET").map_err(|_| "LASTFM_API_SECRET is missing")?;
    let session_key =
        std::env::var("LASTFM_SESSION_KEY").map_err(|_| "LASTFM_SESSION_KEY is missing")?;

    let mut params = BTreeMap::new();
    params.insert("album".to_string(), track.album.clone());
    params.insert("api_key".to_string(), api_key);
    params.insert("artist".to_string(), track.artist.clone());
    params.insert("method".to_string(), method.to_string());
    params.insert("sk".to_string(), session_key);
    params.insert("track".to_string(), track.title.clone());
    if let Some(timestamp) = timestamp {
        params.insert("timestamp".to_string(), timestamp);
    }

    let signature = api_signature(&params, &api_secret);
    let mut command = Command::new("curl");
    command.args(["-fsS", "-X", "POST", "https://ws.audioscrobbler.com/2.0/"]);
    for (key, value) in &params {
        command
            .arg("--data-urlencode")
            .arg(format!("{key}={value}"));
    }
    command
        .arg("--data-urlencode")
        .arg(format!("api_sig={signature}"))
        .arg("--data-urlencode")
        .arg("format=json");

    let output = command
        .output()
        .map_err(|err| format!("curl failed: {err}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(if stdout.is_empty() {
            "Last.fm accepted the request.".to_string()
        } else {
            stdout
        })
    } else if stderr.is_empty() {
        Err(format!("Last.fm request failed with {}", output.status))
    } else {
        Err(stderr)
    }
}

fn api_signature(params: &BTreeMap<String, String>, secret: &str) -> String {
    let mut raw = String::new();
    for (key, value) in params {
        raw.push_str(key);
        raw.push_str(value);
    }
    raw.push_str(secret);
    format!("{:x}", md5::compute(raw))
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds(
                point(gpui::px(80.), gpui::px(80.)),
                size(gpui::px(860.), gpui::px(620.)),
            ))),
            titlebar: None,
            focus: true,
            show: true,
            kind: gpui::WindowKind::Normal,
            is_movable: true,
            is_resizable: true,
            is_minimizable: true,
            display_id: None,
            window_background: gpui::WindowBackgroundAppearance::Opaque,
            app_id: Some("crepuscularity.local-scrobbler".to_string()),
            window_min_size: Some(size(gpui::px(720.), gpui::px(540.))),
            window_decorations: None,
            tabbing_identifier: None,
        };

        cx.open_window(options, |_window, cx| cx.new(ScrobblerApp::new))
            .unwrap();
    });
}
