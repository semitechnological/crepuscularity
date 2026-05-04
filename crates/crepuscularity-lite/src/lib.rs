//! **crepuscularity-lite** — embed a V8 guest + Capacitor-shaped Rust bridge inside a GPUI (or other) host.
//!
//! Development can run TypeScript / TSX guest entries through the built-in Oxc transpiler. Production
//! bundling, chunking, and code splitting are still the embedder’s or CLI toolchain’s job; use the
//! `cl build` esbuild path for bundled output, or multiple sources with
//! [`config::CrepusLiteConfig::guest_prelude`] / separate worker scripts (see `docs/THREADING.md`).
//!
//! ## How this relates to GPUI and Rust
//!
//! - **Rust plugins** are normal Rust: types implementing [`NativePlugin`](bridge::NativePlugin). They are registered on a [`Bridge`](bridge::Bridge) and reached from JS via `Crepus.invoke(...)`.
//! - **GPUI is not “hooked” automatically.** V8 does not register GPUI `actions!`, keymaps, or subscriptions for you. You call [`V8Host::eval`](v8_host::V8Host) (or run scripts) from *your* GPUI event closures—the same places you would call any other Rust code.
//! - **Window / UI-thread work** from plugins is deferred (e.g. `HostDeferred::SetWindowTitle`). After guest code runs, call [`integration::apply_window_deferred`] with a live GPUI `Window` so those operations hit the real window.
//! - **Hot reload:** [`guest_watch::spawn_guest_file_watcher`] watches guest file paths; the callback must be `Send` and should only signal the UI thread (see `docs/THREADING.md` and the package binary).
//! - **Bridge in V8:** each [`V8Host`] installs an embedder slot on the isolate’s context (`CrepusBridgeSlot` in `v8_host.rs`); **`Crepus.invoke`** reads the bridge from the current context (no process-global `ACTIVE_BRIDGE`).
//! - **Optional TOML caps:** [`config::CrepusLiteConfig::build_bridge`] respects `[capabilities]` (see `examples/*/crepus-lite.example.toml`).
//!
//! For a full demo, run the package binary (see `src/main.rs`).

pub mod bench_eval;
pub mod bench_plugin;
pub mod bridge;
pub mod clipboard;
pub mod download_plugin;
mod fs_paths;
pub mod guest_compiler;
pub mod host;
pub mod host_queue;
pub mod integration;
pub mod plugins;
pub mod v8_host;

pub mod config;
pub mod guest_watch;
pub mod v8_thread;
pub mod worker;

pub use bench_eval::{
    bench_matrix_shared_for_configs, eval_guest_from_config_file, BenchMatrixShared,
};
pub use bench_plugin::BenchPlugin;
pub use bridge::{Bridge, BridgeError, Capability, NativePlugin};
pub use download_plugin::DownloadPlugin;
pub use guest_compiler::prepare_guest_source;
pub use host::{HostEventRecord, HostNode, HostRoute, HostSnapshot, HostState, HostStyle};
pub use host_queue::{DeferredWindowDecorations, HostCommandQueue, HostDeferred};
pub use plugins::{AppPlugin, ClipboardPlugin, CorePlugin, FsPlugin, HostPlugin, WindowPlugin};
pub use v8_host::V8Host;
pub use v8_thread::{V8ThreadHandle, V8ThreadRequest, V8ThreadRuntime};
pub use worker::{WorkerHandle, WorkerRuntime};
