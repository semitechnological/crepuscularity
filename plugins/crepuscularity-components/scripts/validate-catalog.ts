#!/usr/bin/env bun
/**
 * Ensure every catalog entry has a matching spec file, every spec is listed,
 * and every declared theme has a JSON file under catalog/themes/.
 */
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const catalogPath = join(root, "catalog/components.json");
const specsDir = join(root, "specs");
const themesDir = join(root, "catalog/themes");

type Entry = { id: string; spec: string; themes?: string[]; platforms?: string[] };

const catalog = JSON.parse(readFileSync(catalogPath, "utf8")) as {
  themes?: string[];
  platforms?: string[];
  components: Entry[];
};

const errors: string[] = [];
const catalogIds = new Set<string>();
const knownPlatforms = new Set(catalog.platforms ?? []);
const knownThemes = new Set(catalog.themes ?? []);

const themeFiles = new Set(
  readdirSync(themesDir)
    .filter((f) => f.endsWith(".json"))
    .map((f) => f.replace(/\.json$/, "")),
);

for (const name of knownThemes) {
  if (!themeFiles.has(name)) {
    errors.push(`catalog theme missing file: catalog/themes/${name}.json`);
  }
}
for (const name of themeFiles) {
  if (!knownThemes.has(name)) {
    errors.push(`orphan theme not in catalog.themes: ${name}.json`);
  }
}

for (const entry of catalog.components) {
  if (catalogIds.has(entry.id)) {
    errors.push(`duplicate catalog id: ${entry.id}`);
  }
  catalogIds.add(entry.id);
  const abs = join(root, entry.spec);
  if (!existsSync(abs)) {
    errors.push(`missing spec for ${entry.id}: ${entry.spec}`);
    continue;
  }
  const spec = JSON.parse(readFileSync(abs, "utf8")) as {
    id?: string;
    themes?: string[];
    platforms?: string[];
  };
  if (spec.id !== entry.id) {
    errors.push(`spec id mismatch for ${entry.id}: got ${spec.id}`);
  }
  for (const t of entry.themes ?? []) {
    if (!knownThemes.has(t)) {
      errors.push(`component ${entry.id}: unknown theme "${t}"`);
    }
  }
  for (const p of entry.platforms ?? []) {
    if (knownPlatforms.size && !knownPlatforms.has(p)) {
      errors.push(`component ${entry.id}: unknown platform "${p}"`);
    }
  }
  for (const t of spec.themes ?? []) {
    if (!knownThemes.has(t)) {
      errors.push(`spec ${entry.id}: unknown theme "${t}"`);
    }
  }
}

const specFiles = readdirSync(specsDir).filter((f) => f.endsWith(".json"));
for (const file of specFiles) {
  const id = file.replace(/\.json$/, "");
  if (!catalogIds.has(id)) {
    errors.push(`orphan spec not in catalog: ${file}`);
  }
}

if (errors.length) {
  console.error("validate-catalog FAILED:");
  for (const e of errors) console.error(" -", e);
  process.exit(1);
}

console.log(
  `validate-catalog OK: ${catalog.components.length} components, ${specFiles.length} specs, ${knownThemes.size} themes`,
);
