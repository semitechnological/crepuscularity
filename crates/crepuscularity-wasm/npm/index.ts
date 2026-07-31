import {
  ir_version,
  parse_crepus_json,
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
