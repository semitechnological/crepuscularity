import {
  ir_version,
  parse_crepus_json,
  parse_template_json,
} from "./wasm/crepuscularity_wasm.js";
import type { ViewIr } from "./types.js";

export type {
  PickerOption,
  StackAxis,
  TabItem,
  ViewIr,
  ViewNode,
  ViewNodeKind,
  ViewStyle,
} from "./types.js";

/// Schema version of the View IR this build of the parser emits.
export const IR_VERSION: number = ir_version();

export type CrepusContext = Record<string, string | number | boolean>;

/**
 * Parse `.crepus` source into View IR using the same Rust parser that backs the
 * `crepus` CLI.
 */
export function parseCrepus(source: string, context?: CrepusContext): ViewIr {
  const json = parse_crepus_json(
    source,
    context ? JSON.stringify(context) : undefined,
  );
  return JSON.parse(json) as ViewIr;
}

/**
 * Parse any supported template syntax into View IR. The frontend is chosen from
 * `filename`'s extension, so `.crepus`, `.jsx`, `.tsx`, `.svelte` and `.vue`
 * each compile through their own parser into the same IR.
 */
export function parseTemplate(
  source: string,
  filename?: string,
  context?: CrepusContext,
): ViewIr {
  const json = parse_template_json(
    source,
    filename,
    context ? JSON.stringify(context) : undefined,
  );
  return JSON.parse(json) as ViewIr;
}
