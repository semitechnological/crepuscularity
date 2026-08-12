# Plasmo Benchmark App — Vulnerability Audit

## Summary

The Plasmo framework (`plasmo@0.90.5`) is the **latest and final release**. The
project is unmaintained — no newer version exists on npm. Plasmo 0.90.5 depends
on **Parcel 2.9.3** (`@parcel/core`, `@parcel/fs`, `@parcel/package-manager`,
etc.) and a large tree of `@plasmohq/parcel-*` packages, all of which pull in
vulnerable transitive dependencies.

## Audit Results (npm audit)

```
73 vulnerabilities (1 low, 5 moderate, 67 high)
```

### Key Vulnerable Packages

| Package | Severity | Advisory |
|---------|----------|----------|
| `@parcel/reporter-dev-server` | high | Origin Validation Error (GHSA-qm9p-f9j5-w83w) |
| `content-security-policy-parser` | high | Prototype Pollution → RCE (GHSA-w2cq-g8g3-gm83) |
| `msgpackr` (via `lmdb` → `@parcel/cache`) | high | Infinite recursion (GHSA-7hpj-7hhx-2fgx) |
| `esbuild` (via `tsup`) | moderate | Dev server cross-origin requests (GHSA-67mh-4wv8-2f99) |
| `js-yaml` | moderate | Quadratic-complexity DoS (GHSA-h67p-54hq-rp68) |
| `@babel/core` | moderate | Arbitrary File Read via sourceMappingURL (GHSA-4x5r-pxfx-6jf8) |

### Root Cause

All vulnerabilities trace back to Parcel 2.9.3 and the `@plasmohq/parcel-*`
ecosystem. `npm audit fix --force` would downgrade to `plasmo@0.50.1`, which is
a major breaking change and would likely not build the extension correctly.

## Resolution

**These vulnerabilities cannot be resolved by upgrading Plasmo** — 0.90.5 is
the last published version and the framework is unmaintained.

### Options

1. **Replace the benchmark app** with a simpler browser-extension setup that
   does not depend on Parcel (e.g., a plain Vite + webextension polyfill
   project, or WXT). This would eliminate all 73 vulnerabilities.
2. **Keep as-is** — this is a benchmark/example app, not a production
   application. The vulnerabilities are in build-time dev dependencies, not
   runtime code shipped to users. The risk is limited to developer machines
   running `plasmo dev`.

### Recommendation

Since this is a benchmark app for comparing framework overhead, the Plasmo
benchmark can remain as-is for now with the understanding that the
vulnerabilities are in build-time tooling only. If this app were to be used
in production, it should be replaced with a WXT-based or plain Vite setup.
