//! `crepus native` — Native mobile applications for iOS and Android.
//!
//! Scaffold and build native iOS (SwiftUI) and Android (Jetpack Compose) apps
//! that use View IR to render .crepus templates.

use std::fs;
use std::path::Path;
use std::process::Command;

use console::style;

use crate::ui;

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus native new <name>");
            });
            scaffold_native_app(name);
        }
        Some("build") => match args.get(1).map(|s| s.as_str()) {
            Some("ios") => {
                let scheme = args
                    .iter()
                    .find(|a| a.starts_with("--scheme="))
                    .map(|s| s.strip_prefix("--scheme=").unwrap())
                    .unwrap_or("NativeShell");
                build_ios_app(scheme);
            }
            Some("android") => {
                let flavor = args
                    .iter()
                    .find(|a| a.starts_with("--flavor="))
                    .map(|s| s.strip_prefix("--flavor=").unwrap())
                    .unwrap_or("debug");
                build_android_app(flavor);
            }
            _ => ui::error("Usage: crepus native build ios|android [options]"),
        },
        Some("run") => match args.get(1).map(|s| s.as_str()) {
            Some("ios") => run_ios_app(),
            Some("android") => run_android_app(),
            _ => ui::error("Usage: crepus native run ios|android"),
        },
        _ => print_native_usage(),
    }
}

fn scaffold_native_app(name: &str) {
    ui::error("Native app scaffolding not implemented yet - coming soon!");
}

fn build_ios_app(_scheme: &str) {
    ui::error("iOS build not implemented yet - use Xcode directly");
}

fn build_android_app(_flavor: &str) {
    ui::error("Android build not implemented yet - use Gradle directly");
}

fn run_ios_app() {
    ui::error("iOS run not implemented yet - use Xcode to run the app");
}

fn run_android_app() {
    ui::error("Android run not implemented yet - use Android Studio or adb");
}

fn print_native_usage() {
    eprintln!(
        "{}",
        style("crepus native - Native mobile applications")
            .cyan()
            .bold()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>              ").green(),
        style("scaffold cross-platform native app").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build ios [--scheme S]   ").green(),
        style("build iOS app with Xcode").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build android [--flavor F]").green(),
        style("build Android app with Gradle").dim()
    );
    eprintln!(
        "  {}  {}",
        style("run ios                  ").green(),
        style("run iOS app (use Xcode)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("run android              ").green(),
        style("install and run Android app").dim()
    );
    eprintln!();
    eprintln!("{}", style("EXAMPLES").dim());
    eprintln!("  crepus native new my-mobile-app");
    eprintln!("  crepus native build ios");
    eprintln!("  crepus native build android");
}
