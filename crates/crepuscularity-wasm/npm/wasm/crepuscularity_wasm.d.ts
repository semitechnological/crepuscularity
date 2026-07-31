/* tslint:disable */
/* eslint-disable */

/**
 * Schema version of the IR this module emits; consumers should check it.
 */
export function ir_version(): number;

/**
 * Parse `.crepus` source into View IR, serialized as JSON.
 *
 * `context_json`, when present, must be a JSON object whose values are bound
 * as template variables.
 */
export function parse_crepus_json(source: string, context_json?: string | null): string;
