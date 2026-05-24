use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};

use clap::{Parser, Subcommand};
use crepuscularity_lite::config::CrepusLiteConfig;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(name = "cl", about = "crepuscularity-lite CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Build {
        example: Option<String>,
    },
    Dev {
        example: Option<String>,
    },
    Serve {
        example: Option<String>,
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    New {
        name: String,
    },
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
}

enum ProjectType {
    Lite,
    Full,
}

fn repo_root() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| format!("cwd: {e}"))
}

fn sdkroot() -> String {
    std::env::var("SDKROOT")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            Command::new("xcrun")
                .arg("--show-sdk-path")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        })
}

fn detect_project_type(dir: &Path) -> Result<ProjectType, String> {
    if dir.join("crepus.toml").exists() {
        return Ok(ProjectType::Full);
    }
    if dir.join("crepus-lite.toml").exists() || dir.join("crepus-lite.example.toml").exists() {
        return Ok(ProjectType::Lite);
    }
    Err(format!(
        "not a crepuscularity-lite project: no crepus.toml or crepus-lite.toml found in {}",
        dir.display()
    ))
}

fn resolve_target_dir(target: &str) -> Result<PathBuf, String> {
    let direct = PathBuf::from(target);
    if direct.exists() {
        if direct.is_dir() {
            return Ok(direct);
        }
        return Err(format!("target is not a directory: {}", direct.display()));
    }

    let root = repo_root()?;
    let example_dir = root.join("examples").join(target);
    if example_dir.exists() && example_dir.is_dir() {
        return Ok(example_dir);
    }

    Err(format!("target not found: {target}"))
}

fn resolve_build_script(dir: &Path) -> Result<&'static str, String> {
    let package_path = dir.join("package.json");
    let raw = std::fs::read_to_string(&package_path)
        .map_err(|e| format!("read {}: {e}", package_path.display()))?;
    let pkg: Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", package_path.display()))?;
    let scripts = pkg
        .get("scripts")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing scripts object in {}", package_path.display()))?;

    if scripts.contains_key("crepus:build") {
        return Ok("crepus:build");
    }
    if scripts.contains_key("build") {
        return Ok("build");
    }
    Err(format!(
        "missing build script (expected scripts.build or scripts.\"crepus:build\") in {}",
        package_path.display()
    ))
}

fn run_bun_build(dir: &Path, install: bool, watch: bool) -> Result<(), String> {
    let script = resolve_build_script(dir)?;

    if install {
        let status = Command::new("bun")
            .arg("install")
            .current_dir(dir)
            .status()
            .map_err(|e| format!("bun install: {e}"))?;
        if !status.success() {
            return Err("bun install failed".into());
        }
    }

    let mut build = Command::new("bun");
    build.arg("run").arg(script);
    if watch {
        build.arg("--watch");
    }
    let status = build
        .current_dir(dir)
        .status()
        .map_err(|e| format!("bun run build: {e}"))?;
    if !status.success() {
        return Err("bun run build failed".into());
    }
    Ok(())
}

fn spawn_bun_watch(dir: &Path) -> Result<Child, String> {
    let script = resolve_build_script(dir)?;
    Command::new("bun")
        .arg("install")
        .current_dir(dir)
        .status()
        .map_err(|e| format!("bun install: {e}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err("bun install failed".to_string())
            }
        })?;

    Command::new("bun")
        .arg("run")
        .arg(script)
        .arg("--watch")
        .current_dir(dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("bun run build --watch: {e}"))
}

fn resolve_config_path(dir: &Path) -> Result<PathBuf, String> {
    let primary = dir.join("crepus-lite.toml");
    if primary.exists() {
        return Ok(primary);
    }
    let example = dir.join("crepus-lite.example.toml");
    if example.exists() {
        return Ok(example);
    }
    Err(format!(
        "missing crepus-lite.toml or crepus-lite.example.toml in {}",
        dir.display()
    ))
}

fn guest_entry_path(config_path: &Path, dev: bool) -> Result<PathBuf, String> {
    let cfg = CrepusLiteConfig::load_from_path(config_path)
        .map_err(|e| format!("load {}: {e}", config_path.display()))?;
    let entry = if dev {
        cfg.dev_guest_entry.or(cfg.guest_entry)
    } else {
        cfg.guest_entry
    }
    .ok_or_else(|| format!("guest_entry missing in {}", config_path.display()))?;
    let base = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(base.join(entry))
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

fn spawn_gui(config_path: &Path, verbose: bool, dev: bool) -> Result<Child, String> {
    let root = repo_root()?;
    let abs_config = if config_path.is_absolute() {
        config_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("cwd: {e}"))?
            .join(config_path)
    };
    let abs_config_canon = abs_config
        .canonicalize()
        .map_err(|e| format!("canonicalize config: {e}"))?;
    let config_base = abs_config_canon
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    eprintln!("CLI: CREPUS_LITE_CONFIG={}", abs_config_canon.display());

    Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("crepuscularity-lite")
        .current_dir(root)
        .env("SDKROOT", sdkroot())
        .env("CREPUS_LITE_CONFIG", abs_config_canon)
        .env("CREPUS_LITE_BASE", config_base)
        .env("CREPUS_LITE_MODE", if dev { "dev" } else { "serve" })
        .env("CREPUS_LITE_VERBOSE", if verbose { "1" } else { "0" })
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("cargo run: {e}"))
}

fn run_gui_once(config_path: &Path, verbose: bool) -> Result<(), String> {
    let status = spawn_gui(config_path, verbose, false)?
        .wait()
        .map_err(|e| format!("wait cargo run: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("cargo run failed".to_string())
    }
}

fn run_dev_loop(target: &str, verbose: bool) -> Result<(), String> {
    let dir = resolve_target_dir(target)?;
    detect_project_type(&dir)?;
    let config_path = resolve_config_path(&dir)?;
    let guest_path = guest_entry_path(&config_path, true)?;
    let dev_native = CrepusLiteConfig::load_from_path(&config_path)
        .map(|cfg| cfg.dev_guest_entry.is_some())
        .unwrap_or(false);

    let mut watch = if dev_native {
        None
    } else {
        Some(spawn_bun_watch(&dir)?)
    };
    let mut gui: Child;
    let mut last_seen = modified(&guest_path);

    if !dev_native {
        for _ in 0..120 {
            if guest_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(250));
        }
    }
    if !guest_path.exists() {
        return Err(format!(
            "guest entry does not exist: {}",
            guest_path.display()
        ));
    }

    gui = spawn_gui(&config_path, verbose, true)?;

    loop {
        if let Some(watch) = watch.as_mut() {
            if let Some(status) = watch.try_wait().map_err(|e| format!("watch status: {e}"))? {
                if !status.success() {
                    return Err("build watch process exited with failure".to_string());
                }
                return Ok(());
            }
        }

        let current = modified(&guest_path);
        if current.is_some() && current != last_seen {
            last_seen = current;
            let _ = gui.kill();
            let _ = gui.wait();
            gui = spawn_gui(&config_path, verbose, true)?;
        }

        if let Some(_status) = gui.try_wait().map_err(|e| format!("gui status: {e}"))? {
            gui = spawn_gui(&config_path, verbose, true)?;
        }

        thread::sleep(Duration::from_millis(300));
    }
}

fn scaffold_new(name: &str) -> Result<(), String> {
    let root = repo_root()?;
    let dir = root.join("examples").join(name);
    if dir.exists() {
        return Err(format!("example already exists: {}", dir.display()));
    }
    std::fs::create_dir_all(dir.join("src")).map_err(|e| format!("create dirs: {e}"))?;
    std::fs::write(
        dir.join("package.json"),
        format!(
            "{{\n  \"name\": \"crepus-example-{name}\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {{\n    \"build\": \"bun build.mjs\"\n  }},\n  \"devDependencies\": {{\n    \"esbuild\": \"^0.24.2\"\n  }}\n}}\n"
        ),
    )
    .map_err(|e| format!("write package.json: {e}"))?;
    std::fs::write(
        dir.join("build.mjs"),
        "import * as esbuild from \"esbuild\";\nimport { guestIifeOptions } from \"../guest-starters/scripts/guest-esbuild-defaults.mjs\";\n\nawait esbuild.build(guestIifeOptions({ entryPoints: [\"src/main.ts\"] }));\n",
    )
    .map_err(|e| format!("write build.mjs: {e}"))?;
    std::fs::write(
        dir.join("src/main.ts"),
        "type HostNode = {\n  type: string;\n  text?: string;\n  style?: Record<string, unknown>;\n  children?: HostNode[];\n};\n\nfunction invoke(plugin: string, method: string, payload: unknown) {\n  return JSON.parse(Crepus.invoke(plugin, method, JSON.stringify(payload)));\n}\n\nexport function run() {\n  Crepus.invoke(\"window\", \"setTitle\", JSON.stringify({ title: \"crepus-lite\" }));\n  const tree: HostNode = {\n    type: \"column\",\n    style: {\n      width: 720,\n      height: 480,\n      padding: 24,\n      gap: 12,\n      background: \"#09090b\",\n      color: \"#f4f4f5\",\n    },\n    children: [\n      { type: \"text\", text: \"hello crepus\", style: { fontSize: 24, fontWeight: \"bold\" } },\n      { type: \"text\", text: \"runtime-native TypeScript in dev\" },\n    ],\n  };\n  return invoke(\"host\", \"renderTree\", { tree });\n}\n",
    )
    .map_err(|e| format!("write src/main.ts: {e}"))?;
    std::fs::write(
        dir.join("crepus-lite.example.toml"),
        "guest_entry = \"dist/guest.js\"\ndev_guest_entry = \"src/main.ts\"\nwatch_guest = true\n",
    )
    .map_err(|e| format!("write toml: {e}"))?;
    println!("created {}", dir.display());
    Ok(())
}

fn detect_framework(package_json: &Value) -> &'static str {
    let has_dep = |name: &str| {
        package_json
            .get("dependencies")
            .and_then(Value::as_object)
            .and_then(|m| m.get(name))
            .is_some()
            || package_json
                .get("devDependencies")
                .and_then(Value::as_object)
                .and_then(|m| m.get(name))
                .is_some()
    };

    if has_dep("react") {
        "react"
    } else if has_dep("vue") {
        "vue"
    } else if has_dep("solid-js") {
        "solid"
    } else if has_dep("svelte") {
        "svelte"
    } else {
        "vanilla"
    }
}

fn infer_entry_file(project_dir: &Path) -> Option<&'static str> {
    const CANDIDATES: &[&str] = &[
        "src/main.tsx",
        "src/main.ts",
        "src/main.jsx",
        "src/main.js",
        "src/index.tsx",
        "src/index.ts",
        "src/index.jsx",
        "src/index.js",
    ];
    CANDIDATES
        .iter()
        .copied()
        .find(|candidate| project_dir.join(candidate).exists())
}

fn parse_entry_stem(entry: &str) -> String {
    let stem = Path::new(entry)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    stem.to_string()
}

fn supports_native_dev_entry(project_dir: &Path, entry: &str) -> bool {
    if !(entry.ends_with(".ts")
        || entry.ends_with(".tsx")
        || entry.ends_with(".js")
        || entry.ends_with(".jsx"))
    {
        return false;
    }
    let Ok(source) = std::fs::read_to_string(project_dir.join(entry)) else {
        return false;
    };
    !source.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("import ")
            || trimmed.starts_with("export *")
            || trimmed.contains(" from ")
    })
}

fn build_alias_lines() -> &'static str {
    "  alias: {\n    react: \"crepus-gpui-runtime/react\",\n    \"react-dom\": \"crepus-gpui-runtime/react-dom-client\",\n    \"react-dom/client\": \"crepus-gpui-runtime/react-dom-client\",\n    \"react/jsx-runtime\": \"crepus-gpui-runtime/jsx-runtime\",\n    \"react/jsx-dev-runtime\": \"crepus-gpui-runtime/jsx-runtime\",\n    vue: \"crepus-gpui-runtime/vue-runtime-dom\",\n    \"@vue/runtime-dom\": \"crepus-gpui-runtime/vue-runtime-dom\",\n    \"solid-js\": \"crepus-gpui-runtime/solid-web\",\n    \"solid-js/web\": \"crepus-gpui-runtime/solid-web\",\n  },"
}

fn runtime_dependency_specs() -> (String, String) {
    let root = repo_root().unwrap_or_else(|_| PathBuf::from("."));
    let crepus_lite_path = root.join("npm").join("crepus-lite").join("package.json");
    let gpui_runtime_path = root
        .join("npm")
        .join("crepus-gpui-runtime")
        .join("package.json");

    if crepus_lite_path.exists() && gpui_runtime_path.exists() {
        let crepus = format!("file:{}", root.join("npm/crepus-lite").display());
        let runtime = format!("file:{}", root.join("npm/crepus-gpui-runtime").display());
        return (crepus, runtime);
    }

    ("^0.3.0".to_string(), "^0.3.0".to_string())
}

fn update_package_json(project_dir: &Path) -> Result<(), String> {
    let package_path = project_dir.join("package.json");
    let raw = std::fs::read_to_string(&package_path)
        .map_err(|e| format!("read {}: {e}", package_path.display()))?;
    let mut pkg: Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", package_path.display()))?;

    let obj = pkg.as_object_mut().ok_or_else(|| {
        format!(
            "package.json must be a JSON object: {}",
            package_path.display()
        )
    })?;

    let scripts = obj
        .entry("scripts")
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .ok_or_else(|| "scripts must be an object".to_string())?;
    scripts.insert(
        "crepus:build".into(),
        Value::String("bun run ./crepus.build.mjs".into()),
    );
    scripts.insert("crepus:dev".into(), Value::String("cl dev .".into()));
    scripts.insert("crepus:serve".into(), Value::String("cl serve .".into()));

    let dependencies = obj
        .entry("dependencies")
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .ok_or_else(|| "dependencies must be an object".to_string())?;
    let (crepus_lite_spec, gpui_runtime_spec) = runtime_dependency_specs();
    dependencies.insert("crepus-lite".into(), Value::String(crepus_lite_spec));
    dependencies.insert(
        "crepus-gpui-runtime".into(),
        Value::String(gpui_runtime_spec),
    );

    let dev_dependencies = obj
        .entry("devDependencies")
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .ok_or_else(|| "devDependencies must be an object".to_string())?;
    dev_dependencies
        .entry("esbuild")
        .or_insert(Value::String("^0.24.2".into()));

    let mut ordered = BTreeMap::new();
    for (k, v) in obj.iter() {
        ordered.insert(k.clone(), v.clone());
    }

    let updated = serde_json::to_string_pretty(&ordered)
        .map_err(|e| format!("serialize package.json: {e}"))?;
    std::fs::write(&package_path, format!("{updated}\n"))
        .map_err(|e| format!("write {}: {e}", package_path.display()))
}

fn init_project(path: &str) -> Result<(), String> {
    let project_dir = PathBuf::from(path);
    if !project_dir.exists() || !project_dir.is_dir() {
        return Err(format!(
            "project directory not found: {}",
            project_dir.display()
        ));
    }

    let package_path = project_dir.join("package.json");
    if !package_path.exists() {
        return Err(format!("missing package.json: {}", package_path.display()));
    }

    let package_raw = std::fs::read_to_string(&package_path)
        .map_err(|e| format!("read {}: {e}", package_path.display()))?;
    let package_json: Value =
        serde_json::from_str(&package_raw).map_err(|e| format!("parse package.json: {e}"))?;
    let framework = detect_framework(&package_json);
    let entry = infer_entry_file(&project_dir).unwrap_or("src/main.ts");

    let runner_ext = if entry.ends_with(".tsx") {
        "tsx"
    } else if entry.ends_with(".jsx") {
        "jsx"
    } else if entry.ends_with(".js") {
        "js"
    } else {
        "ts"
    };

    let guest_runner = project_dir.join(format!("src/crepus-guest-entry.{runner_ext}"));
    if !guest_runner.exists() {
        let stem = parse_entry_stem(entry);
        let import_path = format!("./{stem}");
        std::fs::create_dir_all(project_dir.join("src"))
            .map_err(|e| format!("create src dir: {e}"))?;
        std::fs::write(
            &guest_runner,
            format!("import \"{import_path}\";\n\nexport function run() {{\n  return null;\n}}\n"),
        )
        .map_err(|e| format!("write {}: {e}", guest_runner.display()))?;
    }

    let build_path = project_dir.join("crepus.build.mjs");
    let runner_rel = format!("src/crepus-guest-entry.{runner_ext}");
    std::fs::write(
        &build_path,
        format!(
            "import * as esbuild from \"esbuild\";\n\nawait esbuild.build({{\n  entryPoints: [\"{runner_rel}\"],\n  bundle: true,\n  format: \"iife\",\n  globalName: \"CrepusGuest\",\n  outfile: \"dist/guest.js\",\n  platform: \"browser\",\n  target: \"es2020\",\n{}\n  logLevel: \"info\",\n}});\n"
                ,
            build_alias_lines()
        ),
    )
    .map_err(|e| format!("write {}: {e}", build_path.display()))?;

    let config_path = project_dir.join("crepus-lite.toml");
    if !config_path.exists() {
        let dev_entry = if supports_native_dev_entry(&project_dir, entry) {
            format!("dev_guest_entry = \"{entry}\"\n")
        } else {
            String::new()
        };
        std::fs::write(
            &config_path,
            format!("guest_entry = \"dist/guest.js\"\n{dev_entry}watch_guest = true\n\n[capabilities]\n"),
        )
        .map_err(|e| format!("write {}: {e}", config_path.display()))?;
    }

    update_package_json(&project_dir)?;

    println!("initialized crepus project in {}", project_dir.display());
    println!("framework detected: {framework}");
    println!("next:");
    println!("  1) bun install");
    println!("  2) cl build .");
    println!("  3) cl serve .");
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Build { example } => {
            let target = example.as_deref().unwrap_or(".");
            let dir = resolve_target_dir(target);
            if let Err(e) = &dir {
                eprintln!("cl: {e}");
                std::process::exit(1);
            }
            let dir = dir.unwrap();
            if let Err(e) = detect_project_type(&dir) {
                eprintln!("cl: {e}");
                std::process::exit(1);
            }
            run_bun_build(&dir, true, false)
        }
        Cmd::Dev { example } => {
            let target = example.as_deref().unwrap_or(".");
            let dir = resolve_target_dir(target);
            if let Err(e) = &dir {
                eprintln!("cl: {e}");
                std::process::exit(1);
            }
            let dir = dir.unwrap();
            if let Err(e) = detect_project_type(&dir) {
                eprintln!("cl: {e}");
                std::process::exit(1);
            }
            run_dev_loop(target, false)
        }
        Cmd::Serve { example, verbose } => {
            let target = example.as_deref().unwrap_or(".");
            let dir = resolve_target_dir(target);
            if let Err(e) = &dir {
                eprintln!("cl: {e}");
                std::process::exit(1);
            }
            let dir = dir.unwrap();
            match detect_project_type(&dir) {
                Ok(ProjectType::Full) => {
                    eprintln!("cl: you are working in a full crepuscularity project; use 'crepus serve' or 'cre web serve' instead");
                    std::process::exit(1);
                }
                Ok(ProjectType::Lite) => {
                    resolve_config_path(&dir).and_then(|config| run_gui_once(&config, verbose))
                }
                Err(e) => Err(e),
            }
        }
        Cmd::New { name } => scaffold_new(&name),
        Cmd::Init { path } => init_project(&path),
    };

    if let Err(err) = result {
        eprintln!("cl: {err}");
        std::process::exit(1);
    }
}
