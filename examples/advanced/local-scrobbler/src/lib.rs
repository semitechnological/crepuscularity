use std::ffi::c_void;

use crepuscularity_gpui::prelude::*;
use gpui::{bounds, point, size, Application, ClickEvent};

type CurrentTrackCallback = unsafe extern "C" fn(*mut c_void, i32, *mut u8, usize) -> i32;
type TrackActionCallback = unsafe extern "C" fn(
    *mut c_void,
    i32,
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
    usize,
    *mut u8,
    usize,
) -> i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CrepusScrobblerCallbacks {
    pub user_data: *mut c_void,
    pub current_track: Option<CurrentTrackCallback>,
    pub now_playing: Option<TrackActionCallback>,
    pub scrobble: Option<TrackActionCallback>,
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
    callbacks: CrepusScrobblerCallbacks,
    source: Source,
    track: Option<Track>,
    status: String,
    last_response: String,
}

impl ScrobblerApp {
    fn new(callbacks: CrepusScrobblerCallbacks, _cx: &mut Context<Self>) -> Self {
        Self {
            callbacks,
            source: Source::Spotify,
            track: None,
            status: "Ready".to_string(),
            last_response:
                "Zig owns Spotify, Apple Music, and Last.fm. Rust only hosts Crepus GPUI."
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
            self.last_response = self.send_track(self.callbacks.now_playing, &track);
            self.status = "Now playing requested".to_string();
        }
        cx.notify();
    }

    fn scrobble(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.refresh_current();
        if let Some(track) = self.track.clone() {
            self.last_response = self.send_track(self.callbacks.scrobble, &track);
            self.status = "Scrobble requested".to_string();
        }
        cx.notify();
    }

    fn refresh_current(&mut self) {
        match self.current_track() {
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

    fn current_track(&self) -> Result<Track, String> {
        let Some(callback) = self.callbacks.current_track else {
            return Err("Zig current_track callback missing".to_string());
        };
        let mut buf = [0u8; 2048];
        let n = unsafe {
            callback(
                self.callbacks.user_data,
                self.source.id(),
                buf.as_mut_ptr(),
                buf.len(),
            )
        };
        if n <= 0 {
            return Err("Zig media bridge returned no track".to_string());
        }
        let raw = String::from_utf8_lossy(&buf[..n as usize])
            .trim()
            .to_string();
        if raw == "stopped" || raw == "unavailable" {
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
            source: self.source,
            artist,
            title,
            album,
            state,
        })
    }

    fn send_track(&self, callback: Option<TrackActionCallback>, track: &Track) -> String {
        let Some(callback) = callback else {
            return "Zig Last.fm callback missing".to_string();
        };
        let mut out = [0u8; 4096];
        let n = unsafe {
            callback(
                self.callbacks.user_data,
                track.source.id(),
                track.artist.as_ptr(),
                track.artist.len(),
                track.title.as_ptr(),
                track.title.len(),
                track.album.as_ptr(),
                track.album.len(),
                out.as_mut_ptr(),
                out.len(),
            )
        };
        if n <= 0 {
            return "Zig Last.fm callback returned no response".to_string();
        }
        String::from_utf8_lossy(&out[..n as usize])
            .trim()
            .to_string()
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
            .unwrap_or_else(|| "Zig app logic calls Rust Crepus GPUI library.".to_string());
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
                            "Zig Local Scrobbler"
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

#[no_mangle]
pub extern "C" fn crepus_local_scrobbler_run(callbacks: CrepusScrobblerCallbacks) -> i32 {
    Application::new().run(move |cx: &mut App| {
        let options = gpui_window_options(
            "crepuscularity.local-scrobbler.zig",
            "Local Scrobbler",
            Some(gpui::WindowBounds::Windowed(bounds(
                point(gpui::px(80.), gpui::px(80.)),
                size(gpui::px(860.), gpui::px(620.)),
            ))),
            Some(size(gpui::px(720.), gpui::px(540.))),
        );

        cx.open_window(options, |_window, cx| {
            cx.new(|cx| ScrobblerApp::new(callbacks, cx))
        })
        .unwrap();
    });
    0
}
