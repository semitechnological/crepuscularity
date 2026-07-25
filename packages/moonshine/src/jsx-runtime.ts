/**
 * Thin JSX runtime re-export.
 *
 * Moonshine apps use the React JSX transform. Point TypeScript `jsxImportSource`
 * at `moonshine` (or import from `moonshine/jsx-runtime`) when you want a single
 * framework entry — the runtime itself is React's.
 *
 * tsconfig:
 * ```json
 * { "compilerOptions": { "jsx": "react-jsx", "jsxImportSource": "moonshine" } }
 * ```
 */
export { jsx, jsxs, Fragment } from "react/jsx-runtime";
export type { JSX } from "react/jsx-runtime";
