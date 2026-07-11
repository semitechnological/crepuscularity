//! Typed CLI surface (`clap`). Command handlers live in sibling modules.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::build_options::BuildOptionsArgs;

#[derive(Parser, Debug)]
#[command(
    name = "crepus",
    version,
    about = "Crepuscularity CLI — scaffold and build multi-target .crepus apps",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    New {
        name: String,
    },
    Init {
        kind: String,
        name: String,
    },
    #[cfg(feature = "desktop")]
    Dev {
        #[arg(long)]
        bin: Option<String>,
        #[arg(long)]
        emit_events: bool,
        #[command(flatten)]
        build: BuildOptionsArgs,
    },
    Build {
        #[command(flatten)]
        build: BuildOptionsArgs,
        #[arg(long = "target", short = 't')]
        target_id: Option<String>,
        #[arg(long)]
        manifest: Option<PathBuf>,
        #[arg(long)]
        all: bool,
        selector: Option<String>,
    },
    #[cfg(feature = "desktop")]
    Preview {
        file: PathBuf,
    },
    Render {
        file: PathBuf,
        #[arg(long)]
        ctx: Option<PathBuf>,
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        #[arg(long)]
        component: Option<String>,
    },
    Web {
        #[command(subcommand)]
        command: WebCommands,
    },
    Webext {
        #[command(subcommand)]
        command: WebextCommands,
    },
    Ios {
        #[command(subcommand)]
        command: IosCommands,
    },
    Tui {
        #[command(subcommand)]
        command: TuiCommands,
    },
    Native {
        #[command(subcommand)]
        command: NativeCommands,
    },
    Mobile {
        #[command(subcommand)]
        command: MobileCommands,
    },
    Aurora {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        aurorality_args: Vec<String>,
    },
    Embedded {
        #[command(subcommand)]
        command: EmbeddedCommands,
    },
    /// Declarative benchmark.toml runs (flags work without `all`/`run` subcommand)
    Benchmark {
        #[command(subcommand)]
        command: Option<BenchmarkCommands>,
        #[command(flatten)]
        flat: BenchmarkRunArgs,
    },
    /// Plugin packages and ABI bindgen (`equilibrium-ffi`, like `eq generate`)
    Plugins {
        #[command(subcommand)]
        command: PluginsCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum WebCommands {
    New {
        name: String,
    },
    Build {
        #[command(flatten)]
        build: BuildOptionsArgs,
        #[arg(long)]
        site: Option<PathBuf>,
        #[arg(long)]
        out_dir: Option<PathBuf>,
        #[arg(short = 'o', long = "output", alias = "out")]
        output: Option<PathBuf>,
        #[arg(long)]
        entry: Option<String>,
        #[arg(long = "target", short = 't')]
        target_id: Option<String>,
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
    #[command(alias = "serve")]
    Dev {
        #[arg(long)]
        site: Option<PathBuf>,
        #[arg(long, default_value_t = 4000)]
        port: u16,
        #[arg(long, default_value = "index.crepus")]
        entry: String,
        #[arg(long = "target", short = 't')]
        target_id: Option<String>,
        #[arg(long)]
        manifest: Option<PathBuf>,
        #[arg(long)]
        axum: bool,
    },
    #[command(name = "build-full")]
    BuildFull {
        #[arg(long)]
        site: Option<PathBuf>,
        #[arg(long)]
        wasm: bool,
        #[arg(long)]
        server: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum WebextCommands {
    New {
        name: String,
    },
    Build {
        #[command(flatten)]
        build: BuildOptionsArgs,
        #[arg(long)]
        app: Option<PathBuf>,
        #[arg(long)]
        browser: Option<BrowserArg>,
    },
    Dev {
        #[command(flatten)]
        build: BuildOptionsArgs,
        #[arg(long)]
        app: Option<PathBuf>,
        #[arg(long)]
        browser: Option<BrowserArg>,
    },
    Manifest {
        #[arg(long)]
        app: Option<PathBuf>,
        #[arg(long)]
        browser: Option<BrowserArg>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum BrowserArg {
    Chromium,
    Firefox,
    Safari,
}

#[derive(Subcommand, Debug)]
pub enum IosCommands {
    New {
        name: String,
    },
    #[command(alias = "gen")]
    Generate {
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        spec: Option<String>,
    },
    Build {
        #[command(flatten)]
        build: BuildOptionsArgs,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        spec: Option<String>,
        #[arg(long)]
        scheme: Option<String>,
        #[arg(long)]
        destination: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TuiCommands {
    New {
        name: String,
    },
    Build {
        #[command(flatten)]
        build: BuildOptionsArgs,
        #[arg(last = true)]
        extra: Vec<String>,
    },
    Run {
        #[command(flatten)]
        build: BuildOptionsArgs,
        #[arg(last = true)]
        extra: Vec<String>,
    },
    Preview {
        file: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum NativeCommands {
    New {
        name: String,
    },
    Extension {
        #[command(subcommand)]
        extension: NativeExtensionCommands,
    },
    Ir {
        #[command(flatten)]
        args: NativeIrArgs,
    },
    Sync {
        #[command(flatten)]
        args: NativeSyncArgs,
    },
    Codegen {
        #[command(flatten)]
        args: NativeCodegenCliArgs,
    },
    Build {
        #[command(subcommand)]
        platform: NativeBuildCommands,
    },
    Run {
        #[command(subcommand)]
        platform: NativeRunCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum NativeExtensionCommands {
    #[command(name = "ios-share")]
    IosShare {
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        #[arg(long, default_value = "CrepusShareExtension")]
        name: String,
    },
    #[command(name = "macos-share")]
    MacosShare {
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        #[arg(long, default_value = "CrepusMacShareExtension")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum NativeBuildCommands {
    Ios {
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value = "simulator")]
        target: NativeIosTargetArg,
        #[arg(long, default_value = "Debug")]
        configuration: String,
    },
    Android {
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value = "Debug")]
        flavor: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum NativeRunCommands {
    Ios {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    Android {
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value = "Debug")]
        flavor: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum NativeIosTargetArg {
    Simulator,
    Device,
}

#[derive(Args, Debug, Clone)]
pub struct NativeIrArgs {
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub component: Option<String>,
    #[arg(long)]
    pub ctx: Option<PathBuf>,
    #[arg(long = "var", value_name = "KEY=VALUE")]
    pub vars: Vec<String>,
    #[arg(long)]
    pub pretty: bool,
    #[arg(long)]
    pub stdin: bool,
    #[arg(long = "stdin-json")]
    pub stdin_json: bool,
    #[arg(long)]
    pub base_dir: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct NativeSyncArgs {
    pub template: Option<PathBuf>,
    #[arg(long, default_value = ".")]
    pub dir: PathBuf,
    #[arg(long)]
    pub out: Vec<PathBuf>,
    #[arg(long = "no-defaults")]
    pub no_defaults: bool,
    #[arg(long)]
    pub component: Option<String>,
    #[arg(long)]
    pub ctx: Option<PathBuf>,
    #[arg(long = "var", value_name = "KEY=VALUE")]
    pub vars: Vec<String>,
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Args, Debug, Clone)]
pub struct NativeCodegenCliArgs {
    pub template: Option<PathBuf>,
    #[arg(long)]
    pub platform: Option<NativeCodegenPlatformArg>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub view_name: Option<String>,
    #[arg(long)]
    pub component: Option<String>,
    #[arg(long)]
    pub ctx: Option<PathBuf>,
    #[arg(long = "var", value_name = "KEY=VALUE")]
    pub vars: Vec<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum NativeCodegenPlatformArg {
    #[value(name = "swiftui", alias = "swift", alias = "ios")]
    SwiftUi,
    #[value(name = "compose", alias = "kotlin", alias = "android")]
    Compose,
}

#[derive(Subcommand, Debug)]
pub enum MobileCommands {
    New {
        name: String,
    },
    Dev {
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        #[arg(long, default_value_t = 4001)]
        port: u16,
        #[arg(long, default_value = "all")]
        platform: MobilePlatformArg,
        #[arg(long, default_value = "views/main.crepus")]
        template: PathBuf,
        #[arg(long)]
        ctx: Option<PathBuf>,
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
    },
    Build {
        #[arg(long, default_value = "all")]
        platform: MobilePlatformArg,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value = "simulator")]
        target: NativeIosTargetArg,
        #[arg(long, default_value = "Debug")]
        configuration: String,
        #[arg(long, default_value = "Debug")]
        flavor: String,
    },
    Run {
        #[arg(long, default_value = "android")]
        platform: MobilePlatformArg,
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value = "Debug")]
        flavor: String,
    },
    Doctor {
        #[arg(long, default_value = "all")]
        platform: MobilePlatformArg,
    },
    Sync {
        #[arg(default_value = "views/main.crepus")]
        template: PathBuf,
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        #[arg(long)]
        pretty: bool,
    },
    Codegen {
        template: Option<PathBuf>,
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        #[arg(long)]
        platform: Option<MobileCodegenPlatformArg>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        view_name: Option<String>,
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MobilePlatformArg {
    #[value(name = "ios", alias = "swift", alias = "swiftui")]
    Ios,
    #[value(name = "android", alias = "compose", alias = "kotlin")]
    Android,
    All,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MobileCodegenPlatformArg {
    #[value(name = "ios", alias = "swift", alias = "swiftui")]
    Ios,
    #[value(name = "android", alias = "compose", alias = "kotlin")]
    Android,
}

#[derive(Subcommand, Debug)]
pub enum EmbeddedCommands {
    Check {
        file: PathBuf,
        #[arg(long)]
        component: Option<String>,
    },
    Snapshot {
        file: PathBuf,
        #[arg(long)]
        width: u16,
        #[arg(long)]
        height: u16,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        ctx: Option<PathBuf>,
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        #[arg(long)]
        component: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum BenchmarkCommands {
    All {
        #[command(flatten)]
        args: BenchmarkRunArgs,
    },
    Run {
        #[command(flatten)]
        args: BenchmarkRunArgs,
    },
    Check {
        #[command(flatten)]
        args: BenchmarkCheckArgs,
    },
}

#[derive(Args, Debug, Clone, Default)]
pub struct BenchmarkCommonArgs {
    #[arg(short = 'c', long = "config")]
    pub config: Option<PathBuf>,
    #[arg(short = 's', long = "suite")]
    pub suite: Option<String>,
    #[arg(long)]
    pub only: Option<String>,
    #[arg(long)]
    pub repo: Option<PathBuf>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct BenchmarkRunArgs {
    #[command(flatten)]
    pub common: BenchmarkCommonArgs,
    #[arg(long)]
    pub json: bool,
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
    #[arg(long)]
    pub clean: bool,
    #[arg(long = "work-root")]
    pub work_root: Option<PathBuf>,
    #[arg(long)]
    pub memory: bool,
    #[arg(long = "no-memory")]
    pub no_memory: bool,
    #[cfg(feature = "tui")]
    #[arg(long = "no-tui")]
    pub no_tui: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct BenchmarkCheckArgs {
    #[command(flatten)]
    pub common: BenchmarkCommonArgs,
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum PluginsCommands {
    /// Generate ABI bindings for every package in crepuscularity-plugins.toml (equilibrium-ffi)
    Bindgen {
        #[arg(long, help = "Path to crepuscularity-plugins.toml")]
        manifest: Option<PathBuf>,
        #[arg(long, help = "Override ABI header (default: contract.abi_header)")]
        abi_header: Option<PathBuf>,
        #[arg(long, help = "Base output dir (default: <repo>/plugins)")]
        out_dir: Option<PathBuf>,
    },
    /// Run each package `test` command from the manifest
    Test {
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
