//! Route parsed `clap` commands to module handlers.

use std::path::PathBuf;
use std::time::Instant;

use crate::cli::{
    BenchmarkCheckArgs, BenchmarkCommands, BenchmarkRunArgs, BrowserArg, Cli, Commands,
    IosCommands, MobileCommands, MobilePlatformArg, NativeCommands, NativeIosTargetArg,
    TuiCommands, WebCommands, WebextCommands,
};
use crate::target_build::ManifestBuildArgs;
use crate::ui;

pub fn run(cli: Cli) {
    let Some(command) = cli.command else {
        print_top_level_help();
        std::process::exit(0);
    };
    match command {
        Commands::New { name } => crate::new::run(&name),
        Commands::Init { kind, name } => run_init(&kind, &name),
        #[cfg(feature = "desktop")]
        Commands::Dev {
            bin,
            emit_events,
            build,
        } => {
            let options = build.into_options_or_exit();
            crate::dev::run(bin, options, emit_events);
        }
        Commands::Build {
            build,
            target_id,
            manifest,
            all,
            selector,
        } => run_top_level_build(build, target_id, manifest, all, selector),
        #[cfg(feature = "desktop")]
        Commands::Preview { file } => crate::preview::run_preview(file),
        Commands::Render {
            file,
            ctx,
            vars,
            component,
        } => crate::render::execute(file, ctx, vars, component),
        Commands::Web { command } => crate::web::execute(command),
        Commands::Webext { command } => crate::webext::execute(command),
        Commands::Ios { command } => crate::ios::execute(command),
        Commands::Tui { command } => crate::tui::execute(command),
        Commands::Native { command } => crate::native::execute(command),
        Commands::Mobile { command } => crate::mobile::execute(command),
        #[cfg(feature = "aurora")]
        Commands::Aurora { aurorality_args } => crate::aurora::run_forwarded(&aurorality_args),
        #[cfg(not(feature = "aurora"))]
        Commands::Aurora { .. } => {
            ui::error("Aurora not compiled in. Rebuild crepus with --features aurora.");
        }
        Commands::Embedded { command } => crate::embedded::execute(command),
        Commands::Benchmark { command, flat } => match command {
            Some(BenchmarkCommands::Check { args }) => {
                crate::benchmark::execute_check(benchmark_check_options(args));
            }
            Some(BenchmarkCommands::All { args }) | Some(BenchmarkCommands::Run { args }) => {
                crate::benchmark::execute_run(benchmark_run_options(args));
            }
            None => crate::benchmark::execute_run(benchmark_run_options(flat)),
        },
    }
}

fn run_init(kind: &str, name: &str) {
    match kind {
        "web" => crate::web::execute(WebCommands::New {
            name: name.to_string(),
        }),
        "webext" | "extension" | "browser-extension" => {
            crate::webext::execute(WebextCommands::New {
                name: name.to_string(),
            })
        }
        "tui" => crate::tui::execute(TuiCommands::New {
            name: name.to_string(),
        }),
        "native" => crate::native::execute(NativeCommands::New {
            name: name.to_string(),
        }),
        "mobile" => crate::mobile::execute(MobileCommands::New {
            name: name.to_string(),
        }),
        "ios" => crate::ios::execute(IosCommands::New {
            name: name.to_string(),
        }),
        #[cfg(feature = "desktop")]
        "gpui" | "app" | "desktop" => crate::new::run(name),
        #[cfg(not(feature = "desktop"))]
        "gpui" | "app" | "desktop" => {
            ui::error("GPUI scaffold not compiled in. Rebuild crepus with --features desktop.");
        }
        #[cfg(feature = "aurora")]
        "aurora" => crate::aurora::run_forwarded(&["new".to_string(), name.to_string()]),
        other => {
            let mut kinds = "web, webext, tui, native, ios, mobile".to_string();
            #[cfg(feature = "desktop")]
            {
                kinds.push_str(", gpui");
            }
            #[cfg(not(feature = "desktop"))]
            {
                kinds.push_str(", app");
            }
            #[cfg(feature = "aurora")]
            {
                kinds.push_str(", aurora");
            }
            ui::error(&format!("unknown init kind {other:?}; expected {kinds}"));
        }
    }
}

fn run_top_level_build(
    build: crate::build_options::BuildOptionsArgs,
    target_id: Option<String>,
    manifest: Option<PathBuf>,
    all: bool,
    selector: Option<String>,
) {
    let options = build.into_options_or_exit();
    let use_manifest = target_id.is_some()
        || manifest.is_some()
        || all
        || selector.is_some()
        || crate::target_build::has_manifest_targets(manifest.clone());
    if use_manifest {
        crate::target_build::execute(ManifestBuildArgs {
            options,
            target_id,
            selector,
            manifest,
            all,
        });
        return;
    }
    let t0 = Instant::now();
    let cwd = std::env::current_dir().unwrap_or_else(|e| {
        ui::error(&format!("cannot determine current directory: {e}"));
    });
    let sp = ui::spinner(if options.release() {
        "cargo build --release"
    } else {
        "cargo build"
    });
    #[cfg(feature = "desktop")]
    let outcome = crate::builder::cargo_build(&cwd, options, None);
    #[cfg(not(feature = "desktop"))]
    let outcome = crate::preview::cargo_build_fallback(&cwd, options);
    if outcome.success {
        ui::spinner_ok(&sp, "build succeeded");
        ui::done_in(t0.elapsed());
    } else {
        sp.finish_and_clear();
        ui::error("build failed");
    }
}

fn print_top_level_help() {
    use clap::CommandFactory;
    let _ = Cli::command().print_help();
}

pub fn browser_target(b: Option<BrowserArg>) -> Option<crepuscularity_webext::BrowserTarget> {
    b.map(|v| match v {
        BrowserArg::Chromium => crepuscularity_webext::BrowserTarget::Chromium,
        BrowserArg::Firefox => crepuscularity_webext::BrowserTarget::Firefox,
    })
}

pub fn benchmark_run_options(args: BenchmarkRunArgs) -> crate::benchmark::RunOptions {
    let verbose = if args.json {
        false
    } else if args.quiet {
        false
    } else if args.verbose {
        true
    } else {
        false
    };
    let measure_memory = !args.no_memory;
    crate::benchmark::RunOptions {
        config_path: args.common.config,
        suite_filter: args.common.suite,
        only: args.common.only.map(|s| {
            s.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        }),
        json_out: args.json,
        dry_run: args.dry_run,
        clean: args.clean,
        repo_override: args.common.repo,
        work_override: args.work_root,
        verbose,
        measure_memory,
        #[cfg(feature = "tui")]
        no_tui: args.no_tui,
    }
}

pub fn benchmark_check_options(args: BenchmarkCheckArgs) -> crate::benchmark::CheckOptions {
    crate::benchmark::CheckOptions {
        config_path: args.common.config,
        suite_filter: args.common.suite,
        only: args.common.only.map(|s| {
            s.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        }),
        json_out: args.json,
        repo_override: args.common.repo,
    }
}

pub fn mobile_platform(p: MobilePlatformArg) -> crate::mobile::MobilePlatform {
    match p {
        MobilePlatformArg::Ios => crate::mobile::MobilePlatform::Ios,
        MobilePlatformArg::Android => crate::mobile::MobilePlatform::Android,
        MobilePlatformArg::All => crate::mobile::MobilePlatform::All,
    }
}

pub fn native_ios_target(t: NativeIosTargetArg) -> crate::native::IosBuildTarget {
    match t {
        NativeIosTargetArg::Simulator => crate::native::IosBuildTarget::Simulator,
        NativeIosTargetArg::Device => crate::native::IosBuildTarget::Device,
    }
}

pub fn native_ios_target_from_rest(rest: &[String]) -> NativeIosTargetArg {
    match crate::native::ios_target_from_cli_args(rest) {
        crate::native::IosBuildTarget::Simulator => NativeIosTargetArg::Simulator,
        crate::native::IosBuildTarget::Device => NativeIosTargetArg::Device,
    }
}
