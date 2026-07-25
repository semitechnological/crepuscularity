//! `crepus moonshine` — scaffold and dependency helper for Moonshine + Crepus apps.
//!
//! Moonshine is an external product: <https://github.com/tschk/moonshine>
//! Crepuscularity compiles `.crepus` → View IR and emits apps that import
//! `@tschk/crepus-moonshine`.
//!
//! - `crepus moonshine new <name>` scaffolds a minimal app under cwd.
//! - `crepus moonshine dep` prints package.json dependency snippets.

use std::path::PathBuf;
use std::time::Instant;

use crate::cli::MoonshineCommands;
use crate::error::CrepusCliError;
use crate::scaffold;
use crate::ui;

const PACKAGE_JSON: &str = r#"{
  "name": "{{slug}}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@tschk/moonshine": "github:tschk/moonshine#path:packages/core",
    "@tschk/crepus-moonshine": "github:tschk/moonshine#path:packages/crepus-moonshine",
    "@tschk/moonshine-components": "github:tschk/moonshine#path:components"
  },
  "devDependencies": {
    "vite": "^6.0.0"
  }
}
"#;

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{{name}}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
"#;

const MAIN_TS: &str = r#"// Minimal Moonshine + Crepuscularity shell.
// Wire `@tschk/crepus-moonshine` (see `crepus moonshine dep`).
import "./app.css";

const root = document.getElementById("app");
if (root) {
  root.innerHTML = `
    <main class="shell">
      <h1>{{name}}</h1>
      <p>Load <code>index.crepus</code> via <code>@tschk/crepus-moonshine</code>.</p>
      <pre id="crepus-source"></pre>
    </main>
  `;
}

fetch("/index.crepus")
  .then((r) => r.text())
  .then((src) => {
    const el = document.getElementById("crepus-source");
    if (el) el.textContent = src;
  })
  .catch(() => {});
"#;

const APP_CSS: &str = r#":root {
  color-scheme: dark;
  font-family: ui-sans-serif, system-ui, sans-serif;
  background: #0a0a0b;
  color: #f4f4f5;
}

body {
  margin: 0;
}

.shell {
  max-width: 40rem;
  margin: 4rem auto;
  padding: 0 1.25rem;
  display: grid;
  gap: 0.75rem;
}

pre {
  overflow: auto;
  padding: 1rem;
  border: 1px solid #27272a;
  border-radius: 0.5rem;
  background: #18181b;
  font-size: 0.85rem;
}
"#;

const INDEX_CREPUS: &str = r#"div w-full min-h-screen p-8 flex flex-col gap-3
 div text-3xl font-bold
  "Hello from Moonshine + .crepus"
 div text-zinc-400
  "Scaffolded by crepus moonshine new."
"#;

const VITE_CONFIG: &str = r#"import { defineConfig } from "vite";

export default defineConfig({
  publicDir: "public",
  server: { port: 5173 },
});
"#;

const README: &str = r#"# {{name}}

Minimal Moonshine + Crepuscularity app scaffolded by `crepus moonshine new`.

Moonshine is a separate product: https://github.com/tschk/moonshine

```bash
bun install
bun run dev
```

Dependency snippets (kept in sync with the CLI):

```bash
crepus moonshine dep
```

Emit a View IR app entry:

```bash
crepus web build --emit moonshine --site .
```
"#;

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
    r#"# package.json dependencies for Moonshine + Crepuscularity
#
# Moonshine lives at https://github.com/tschk/moonshine (separate product).
# Bun/npm git+path form (when the monorepo uses package.json workspaces):

{
  "dependencies": {
    "@tschk/moonshine": "github:tschk/moonshine#path:packages/core",
    "@tschk/crepus-moonshine": "github:tschk/moonshine#path:packages/crepus-moonshine",
    "@tschk/moonshine-components": "github:tschk/moonshine#path:components"
  }
}

# If bun cannot resolve nested #path:… workspaces, clone the repo and use file: /
# workspace protocol instead:
#
# {
#   "dependencies": {
#     "@tschk/moonshine": "file:../moonshine/packages/core",
#     "@tschk/crepus-moonshine": "file:../moonshine/packages/crepus-moonshine",
#     "@tschk/moonshine-components": "file:../moonshine/components"
#   }
# }
#
# Or depend on the whole git repo and import via its workspace packages:
#   "moonshine": "github:tschk/moonshine"
"#
    .to_string()
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

    scaffold::ensure_dir(&base.join("src"))
        .unwrap_or_else(|e| ui::error(&format!("create src: {e}")));
    scaffold::ensure_dir(&base.join("public"))
        .unwrap_or_else(|e| ui::error(&format!("create public: {e}")));

    scaffold::write_template(&base.join("package.json"), PACKAGE_JSON, &[("{{slug}}", &slug)])
        .unwrap_or_else(|e| ui::error(&format!("write package.json: {e}")));
    scaffold::write_template(&base.join("index.html"), INDEX_HTML, &[("{{name}}", name)])
        .unwrap_or_else(|e| ui::error(&format!("write index.html: {e}")));
    scaffold::write_template(&base.join("src/main.ts"), MAIN_TS, &[("{{name}}", name)])
        .unwrap_or_else(|e| ui::error(&format!("write src/main.ts: {e}")));
    scaffold::write_file(&base.join("src/app.css"), APP_CSS)
        .unwrap_or_else(|e| ui::error(&format!("write src/app.css: {e}")));
    scaffold::write_file(&base.join("public/index.crepus"), INDEX_CREPUS)
        .unwrap_or_else(|e| ui::error(&format!("write public/index.crepus: {e}")));
    // Also keep a root index.crepus for `crepus web build --emit moonshine`.
    scaffold::write_file(&base.join("index.crepus"), INDEX_CREPUS)
        .unwrap_or_else(|e| ui::error(&format!("write index.crepus: {e}")));
    scaffold::write_file(&base.join("vite.config.ts"), VITE_CONFIG)
        .unwrap_or_else(|e| ui::error(&format!("write vite.config.ts: {e}")));
    scaffold::write_template(&base.join("README.md"), README, &[("{{name}}", name)])
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
    }
}
