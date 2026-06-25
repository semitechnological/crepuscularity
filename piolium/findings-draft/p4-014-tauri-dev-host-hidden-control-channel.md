---
id: p4-014
phase: L3
slug: tauri-dev-host-hidden-control-channel
severity: medium
category: hidden-control-channel
cwe: CWE-200
status: rejected-fp
rejection_reason: example/benchmark code, not shipping to production; fails Pre-FP Gate check 5
---

# Tauri Vite Dev Config Exposes `TAURI_DEV_HOST` — Hidden Control Channel

## Summary

The Tauri benchmark app's Vite configuration reads `process.env.TAURI_DEV_HOST` and uses it to set the dev server's `host` and HMR configuration. This environment variable controls which network interface the dev server binds to and is used in the HMR WebSocket connection URL. An attacker who controls this env var on the developer's machine can redirect HMR connections to their server, enabling live code injection during development.

## Vulnerable Code

`examples/benchmarks/apps/tauri/vite.config.ts:4-22`

```typescript
const host = process.env.TAURI_DEV_HOST;

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig(async () => ({
  // ...
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,       // <-- attacker-controlled host binding
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

## Impact

1. **Dev server bound to public interface**: Setting `TAURI_DEV_HOST=0.0.0.0` exposes the dev server to the local network
2. **HMR redirect**: Setting `TAURI_DEV_HOST=attacker.com` causes the HMR WebSocket to connect to the attacker's server
3. **Live code injection**: An attacker controlling HMR can inject arbitrary JavaScript into the developer's application

## Attacker Control

The `TAURI_DEV_HOST` environment variable must be set in the developer's shell or `.env` file. Attack scenarios:

1. **Shared development environment**: Docker container with inherited env vars
2. **`.env` file committed to repo**: If `.env` is accidentally committed, all developers are affected
3. **CI/CD pipeline**: Build server with attacker-controlled environment

## Recommended Fix

1. Validate `TAURI_DEV_HOST` against an allowlist (reject anything that's not a local IP, `localhost`, or `127.0.0.1`)
2. Log a warning when `TAURI_DEV_HOST` is set to a non-loopback address
3. Document the security implications in the template
