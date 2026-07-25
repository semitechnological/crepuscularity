# /tmp/moonshine-smoke

React + Vite app scaffolded by `crepus moonshine new`.

Moonshine is a separate product: https://github.com/tschk/moonshine

## Setup

Local moonshine detected at `/Users/undivisible/projects/moonshine` — `package.json` uses `file:` deps.

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
