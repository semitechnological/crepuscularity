#!/usr/bin/env bun
/**
 * Ensure every catalog entry has a matching spec file, and every spec is listed.
 */
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const catalogPath = join(root, "catalog/components.json");
const specsDir = join(root, "specs");

type Entry = { id: string; spec: string };

const catalog = JSON.parse(readFileSync(catalogPath, "utf8")) as {
  components: Entry[];
};

const errors: string[] = [];
const catalogIds = new Set<string>();

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
  const spec = JSON.parse(readFileSync(abs, "utf8")) as { id?: string };
  if (spec.id !== entry.id) {
    errors.push(`spec id mismatch for ${entry.id}: got ${spec.id}`);
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
  `validate-catalog OK: ${catalog.components.length} components, ${specFiles.length} specs`,
);
