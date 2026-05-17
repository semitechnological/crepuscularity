#!/usr/bin/env bun
/**
 * Generate crepuscularity-core Tailwind v4 default palette (hex) from theme.css.
 *
 * Usage:
 *   bun scripts/gen-tailwind-colors.mjs [--theme path] [out.rs]
 */

import { readFileSync, writeFileSync } from "fs";
import { formatHex, parse } from "culori";

const THEME_URL =
  "https://raw.githubusercontent.com/tailwindlabs/tailwindcss/v4.3.0/packages/tailwindcss/theme.css";
const DEFAULT_OUT = "crates/crepuscularity-core/src/tailwind/colors.rs";

const args = process.argv.slice(2);
let themePath = null;
let outPath = DEFAULT_OUT;
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--theme" && args[i + 1]) {
    themePath = args[++i];
  } else if (!args[i].startsWith("-")) {
    outPath = args[i];
  }
}

const css = themePath
  ? readFileSync(themePath, "utf8")
  : await fetch(THEME_URL).then((r) => r.text());

/** @type {Array<[string, string]>} */
const entries = [];

for (const m of css.matchAll(/--color-([a-z]+)-(\d+):\s*oklch\(([^)]+)\);/g)) {
  const [, family, shade, inner] = m;
  const [l, c, h] = inner.split(/\s+/);
  const lNum = l.endsWith("%") ? parseFloat(l) / 100 : parseFloat(l);
  const hex = formatHex(parse(`oklch(${lNum} ${c} ${h})`));
  if (hex) entries.push([`${family}-${shade}`, hex]);
}

for (const m of css.matchAll(/--color-([a-z]+):\s*(#[0-9a-fA-F]+);/g)) {
  entries.push([m[1], m[2].toLowerCase()]);
}

entries.sort((a, b) => a[0].localeCompare(b[0]));

const lines = [
  "//! Full Tailwind CSS v4 default palette (OKLCH → sRGB hex).",
  "//!",
  "//! Generated from tailwindcss v4.3.0 `theme.css` via `scripts/gen-tailwind-colors.mjs`.",
  "//! Class names (`bg-blue-500`, etc.) are unchanged; resolved hex values follow v4.",
  "",
  '/// Look up a Tailwind named color like `"slate-500"` or `"red-100"`.',
  "pub fn lookup_named_color(name: &str) -> Option<&'static str> {",
  "    Some(match name {",
];
for (const [key, hex] of entries) {
  lines.push(`        "${key}" => "${hex}",`);
}
lines.push("        _ => return None,", "    })", "}", "");

writeFileSync(outPath, lines.join("\n"));
console.log(`wrote ${entries.length} colors to ${outPath}`);
