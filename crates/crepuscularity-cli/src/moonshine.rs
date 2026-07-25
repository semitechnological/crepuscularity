//! `crepus moonshine` — scaffold and dependency helper for Moonshine + Crepus apps.
//!
//! Moonshine is an external product: <https://github.com/tschk/moonshine>
//! Crepuscularity compiles `.crepus` → View IR and emits apps that import
//! `@tschk/crepus-moonshine`.
//!
//! - `crepus moonshine new <name>` scaffolds a React + Vite app under cwd.
//! - `crepus moonshine dep` prints package.json dependency snippets.

use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::cli::MoonshineCommands;
use crate::error::CrepusCliError;
use crate::scaffold;
use crate::ui;

/// Resolve a local tschk/moonshine checkout, if present.
///
/// Order: `MOONSHINE_PATH` → `../moonshine` relative to cwd → `~/projects/moonshine`.
fn resolve_moonshine_root() -> Option<PathBuf> {
    if let Ok(raw) = env::var("MOONSHINE_PATH") {
        let p = PathBuf::from(raw);
        if looks_like_moonshine_checkout(&p) {
            return Some(canonicalize_or_self(&p));
        }
    }

    let cwd_sibling = env::current_dir().ok().map(|cwd| cwd.join("..").join("moonshine"));
    if let Some(p) = cwd_sibling {
        if looks_like_moonshine_checkout(&p) {
            return Some(canonicalize_or_self(&p));
        }
    }

    if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
        let p = PathBuf::from(home).join("projects").join("moonshine");
        if looks_like_moonshine_checkout(&p) {
            return Some(canonicalize_or_self(&p));
        }
    }

    None
}

fn looks_like_moonshine_checkout(root: &Path) -> bool {
    root.join("packages/core/package.json").is_file()
        && root.join("packages/crepus-moonshine/package.json").is_file()
        && root.join("components/package.json").is_file()
}

fn canonicalize_or_self(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// `file:` dependency paths for the three @tschk packages.
///
/// Always uses absolute `file:` URLs so scaffolds outside the moonshine tree
/// (e.g. `/tmp/app`) don't get broken `../../Users/...` relatives.
fn file_dep_paths(moonshine_root: &Path, _from: Option<&Path>) -> (String, String, String) {
    let core = moonshine_root.join("packages/core");
    let crepus = moonshine_root.join("packages/crepus-moonshine");
    let components = moonshine_root.join("components");

    let fmt = |target: &Path| -> String {
        format!("file:{}", canonicalize_or_self(target).display())
    };

    (fmt(&core), fmt(&crepus), fmt(&components))
}

fn package_json_template(core: &str, crepus: &str, components: &str) -> String {
    format!(
        r#"{{
  "name": "{{{{slug}}}}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "@tschk/moonshine": "{core}",
    "@tschk/crepus-moonshine": "{crepus}",
    "@tschk/moonshine-components": "{components}",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }},
  "devDependencies": {{
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.5.0",
    "typescript": "^5.8.0",
    "vite": "^6.0.0"
  }}
}}
"#
    )
}

const PLACEHOLDER_CORE: &str = "file:../moonshine/packages/core";
const PLACEHOLDER_CREPUS: &str = "file:../moonshine/packages/crepus-moonshine";
const PLACEHOLDER_COMPONENTS: &str = "file:../moonshine/components";

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{{name}}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#;

const MAIN_TSX: &str = r##"import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { renderCrepusIr } from "@tschk/crepus-moonshine";
import type { CrepusIr } from "@tschk/crepus-moonshine";
// When @tschk/crepus-moonshine exports `ViewIr`, prefer: import type { ViewIr } from "@tschk/crepus-moonshine";
import { Sparkline } from "@tschk/moonshine-components";
import "./app.css";

const sampleIr = {
  version: 1,
  root: [
    {
      kind: "stack",
      axis: "column",
      gap: 16,
      children: [
        {
          kind: "text",
          content: "{{name}}",
          style: { fontSize: 28, fontWeight: 700 },
        },
        {
          kind: "text",
          content: "Moonshine + Crepuscularity View IR",
          style: { opacity: 0.7 },
        },
        {
          kind: "badge",
          label: "renderCrepusIr",
          tone: "accent",
        },
        {
          kind: "button",
          label: "Ping",
          onClick: "ping",
        },
      ],
    },
  ],
} as const satisfies CrepusIr;

function App() {
  return (
    <main className="shell">
      {renderCrepusIr(sampleIr, {
        onAction: (handler) => console.log("action:", handler),
      })}
      <section className="spark">
        <h2>Sparkline</h2>
        <Sparkline values={[2, 4, 3, 7, 5, 9, 6, 8, 10, 7]} color="blue" height={56} />
      </section>
    </main>
  );
}

const el = document.getElementById("app");
if (!el) throw new Error("#app missing");
createRoot(el).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
"##;

const APP_CSS: &str = r#":root {
  color-scheme: light dark;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
  background:
    radial-gradient(1200px 600px at 10% -10%, #1e3a5f55, transparent),
    radial-gradient(900px 500px at 100% 0%, #0f766e33, transparent),
    #0c1117;
  color: #e8eef6;
}

body {
  margin: 0;
  min-height: 100vh;
}

.shell {
  max-width: 36rem;
  margin: 3.5rem auto;
  padding: 0 1.25rem;
  display: grid;
  gap: 1.5rem;
}

.spark h2 {
  margin: 0 0 0.5rem;
  font-size: 0.85rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  opacity: 0.65;
  font-weight: 600;
}
"#;

const INDEX_CREPUS: &str = r#"div w-full min-h-screen p-8 flex flex-col gap-3
 div text-3xl font-bold
  "Hello from Moonshine + .crepus"
 div text-zinc-400
  "Scaffolded by crepus moonshine new."
"#;

const VITE_CONFIG: &str = r#"import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  publicDir: "public",
  server: { port: 5173 },
});
"#;

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "jsx": "react-jsx",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true,
    "isolatedModules": true,
    "resolveJsonModule": true
  },
  "include": ["src"]
}
"#;

fn readme_template(local_hint: &str) -> String {
    format!(
        r#"# {{{{name}}}}

React + Vite app scaffolded by `crepus moonshine new`.

Moonshine is a separate product: https://github.com/tschk/moonshine

## Setup

{local_hint}

```bash
bun install
bun run dev
```

Emit a View IR app entry from `.crepus`:

```bash
crepus web build --emit moonshine --site .
```

Dependency snippets:

```bash
crepus moonshine dep
```
"#
    )
}

pub fn execute(cmd: MoonshineCommands) -> Result<(), CrepusCliError> {
    match cmd {
        MoonshineCommands::New { name } => {
            scaffold_app(&name);
            Ok(())
        }
        MoonshineCommands::Dep => {
            print!("{}", dependency_snippets());
            Ok(())
        }
    }
}

fn dependency_snippets() -> String {
    let mut out = String::from(
        r#"# package.json dependencies for Moonshine + Crepuscularity
#
# Moonshine: https://github.com/tschk/moonshine
# Do NOT use bun's github-repo + nested path form — it 404s. Use file: paths instead.
"#,
    );

    if let Some(root) = resolve_moonshine_root() {
        let (core, crepus, components) = file_dep_paths(&root, env::current_dir().ok().as_deref());
        out.push_str(&format!(
            r#"
# A) Local checkout detected at {}
#    Prefer file: paths (relative to cwd when possible):

{{
  "dependencies": {{
    "@tschk/moonshine": "{core}",
    "@tschk/crepus-moonshine": "{crepus}",
    "@tschk/moonshine-components": "{components}",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }}
}}
"#,
            root.display()
        ));
    } else {
        out.push_str(
            r#"
# A) No local moonshine checkout found.
#    Set MOONSHINE_PATH, place a checkout at ../moonshine, or ~/projects/moonshine.
#    Placeholder file: paths (clone first — see B):

{
  "dependencies": {
    "@tschk/moonshine": "file:../moonshine/packages/core",
    "@tschk/crepus-moonshine": "file:../moonshine/packages/crepus-moonshine",
    "@tschk/moonshine-components": "file:../moonshine/components",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }
}
"#,
        );
    }

    out.push_str(
        r#"
# B) Clone tschk/moonshine, then point file: deps at that tree:

git clone https://github.com/tschk/moonshine.git ../moonshine
# or: git clone https://github.com/tschk/moonshine.git ~/projects/moonshine

# Then either re-run `crepus moonshine dep` / `crepus moonshine new`, or set:
#   export MOONSHINE_PATH=/absolute/path/to/moonshine
"#,
    );

    out
}

fn scaffold_app(name: &str) {
    let t0 = Instant::now();
    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let base = PathBuf::from(&slug);
    if base.exists() {
        ui::error(&format!("directory already exists: {slug}"));
    }

    let app_abs = env::current_dir()
        .map(|cwd| cwd.join(&base))
        .unwrap_or_else(|_| base.clone());

    let (core, crepus, components, local_hint) = match resolve_moonshine_root() {
        Some(root) => {
            let (c, cr, co) = file_dep_paths(&root, Some(&app_abs));
            let hint = format!(
                "Local moonshine detected at `{}` — `package.json` uses `file:` deps.",
                root.display()
            );
            eprintln!("{} {hint}", ui::ok());
            (c, cr, co, hint)
        }
        None => {
            eprintln!(
                "{} no local moonshine checkout found (MOONSHINE_PATH, ../moonshine, or ~/projects/moonshine)",
                ui::warn()
            );
            eprintln!(
                "  {} scaffolding with placeholder file:../moonshine/… — clone tschk/moonshine first (see README)",
                ui::dim("→")
            );
            (
                PLACEHOLDER_CORE.to_string(),
                PLACEHOLDER_CREPUS.to_string(),
                PLACEHOLDER_COMPONENTS.to_string(),
                r#"No local moonshine checkout was found. Clone it next to this app (or set `MOONSHINE_PATH`):

```bash
git clone https://github.com/tschk/moonshine.git ../moonshine
# packages: packages/core, packages/crepus-moonshine, components
```

`package.json` already uses placeholder `file:../moonshine/…` paths."#
                    .to_string(),
            )
        }
    };

    scaffold::ensure_dir(&base.join("src"))
        .unwrap_or_else(|e| ui::error(&format!("create src: {e}")));
    scaffold::ensure_dir(&base.join("public"))
        .unwrap_or_else(|e| ui::error(&format!("create public: {e}")));

    let pkg = package_json_template(&core, &crepus, &components);
    scaffold::write_template(&base.join("package.json"), &pkg, &[("{{slug}}", &slug)])
        .unwrap_or_else(|e| ui::error(&format!("write package.json: {e}")));
    scaffold::write_template(&base.join("index.html"), INDEX_HTML, &[("{{name}}", name)])
        .unwrap_or_else(|e| ui::error(&format!("write index.html: {e}")));
    scaffold::write_template(&base.join("src/main.tsx"), MAIN_TSX, &[("{{name}}", name)])
        .unwrap_or_else(|e| ui::error(&format!("write src/main.tsx: {e}")));
    scaffold::write_file(&base.join("src/app.css"), APP_CSS)
        .unwrap_or_else(|e| ui::error(&format!("write src/app.css: {e}")));
    scaffold::write_file(&base.join("public/index.crepus"), INDEX_CREPUS)
        .unwrap_or_else(|e| ui::error(&format!("write public/index.crepus: {e}")));
    // Root index.crepus for `crepus web build --emit moonshine`.
    scaffold::write_file(&base.join("index.crepus"), INDEX_CREPUS)
        .unwrap_or_else(|e| ui::error(&format!("write index.crepus: {e}")));
    scaffold::write_file(&base.join("vite.config.ts"), VITE_CONFIG)
        .unwrap_or_else(|e| ui::error(&format!("write vite.config.ts: {e}")));
    scaffold::write_file(&base.join("tsconfig.json"), TSCONFIG)
        .unwrap_or_else(|e| ui::error(&format!("write tsconfig.json: {e}")));
    let readme = readme_template(&local_hint);
    scaffold::write_template(&base.join("README.md"), &readme, &[("{{name}}", name)])
        .unwrap_or_else(|e| ui::error(&format!("write README.md: {e}")));

    scaffold::scaffold_success(
        &slug,
        &base,
        &[
            &format!("cd {slug}"),
            "bun install",
            "bun run dev",
            "crepus moonshine dep   # dependency snippets",
            "crepus web build --emit moonshine --site .",
        ],
    );
    ui::done_in(t0.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dep_mentions_moonshine_packages() {
        let snip = dependency_snippets();
        assert!(snip.contains("\"@tschk/moonshine\""));
        assert!(snip.contains("\"@tschk/crepus-moonshine\""));
        assert!(snip.contains("\"@tschk/moonshine-components\""));
        assert!(snip.contains("file:"));
        assert!(snip.contains("git clone https://github.com/tschk/moonshine"));
        assert!(!snip.contains("#path:packages/"));
    }

    #[test]
    fn package_json_includes_react() {
        let pkg = package_json_template(PLACEHOLDER_CORE, PLACEHOLDER_CREPUS, PLACEHOLDER_COMPONENTS);
        assert!(pkg.contains("\"react\""));
        assert!(pkg.contains("\"react-dom\""));
        assert!(pkg.contains("@vitejs/plugin-react"));
        assert!(pkg.contains("file:../moonshine/packages/core"));
    }

    #[test]
    fn looks_like_moonshine_rejects_empty() {
        assert!(!looks_like_moonshine_checkout(Path::new("/tmp/nope-moonshine-xyz")));
    }
}
