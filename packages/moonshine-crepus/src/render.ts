import { createElement, type CSSProperties, type ReactElement, type ReactNode } from "react";

/**
 * Minimal crepuscularity → moonshine IR.
 * Enough for CLI emit to import; not a full View IR v4 decoder.
 */
export type CrepusNode =
  | {
      kind: "text";
      content?: string;
      style?: Record<string, string | number>;
    }
  | {
      kind: "stack";
      axis?: "horizontal" | "vertical" | "row" | "column";
      children?: CrepusNode[];
      style?: Record<string, string | number>;
      gap?: number | string;
    }
  | {
      kind: "button";
      label?: string;
      onClick?: string;
      style?: Record<string, string | number>;
    }
  | {
      kind: "sparkline";
      values?: number[];
      width?: number;
      height?: number;
      color?: string;
      style?: Record<string, string | number>;
    }
  | {
      kind: string;
      [key: string]: unknown;
    };

export type CrepusIr = {
  version?: number;
  root: CrepusNode[];
};

export type RenderCrepusOptions = {
  /** Invoked when a button's `onClick` handler name fires. */
  onAction?: (handler: string) => void;
  /** Optional key prefix for React reconciliation. */
  keyPrefix?: string;
};

function styleOf(node: { style?: Record<string, string | number> }): CSSProperties | undefined {
  return node.style as CSSProperties | undefined;
}

function sparklinePath(values: number[], width: number, height: number): string {
  if (values.length === 0) return "";
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const step = values.length === 1 ? 0 : width / (values.length - 1);
  return values
    .map((v, i) => {
      const x = i * step;
      const y = height - ((v - min) / span) * height;
      return `${i === 0 ? "M" : "L"}${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");
}

/** Map one IR node to a moonshine/React element. */
export function renderCrepusNode(
  node: CrepusNode,
  options: RenderCrepusOptions = {},
  key = "0",
): ReactNode {
  switch (node.kind) {
    case "text": {
      const n = node as Extract<CrepusNode, { kind: "text" }>;
      return createElement(
        "span",
        { key, "data-crepus-kind": "text", style: styleOf(n) },
        n.content ?? "",
      );
    }
    case "stack": {
      const n = node as Extract<CrepusNode, { kind: "stack" }>;
      const axis = n.axis ?? "column";
      const row = axis === "horizontal" || axis === "row";
      const style: CSSProperties = {
        display: "flex",
        flexDirection: row ? "row" : "column",
        gap: n.gap ?? 8,
        ...styleOf(n),
      };
      const children = (n.children ?? []).map((child, i) =>
        renderCrepusNode(child, options, `${key}.${i}`),
      );
      return createElement(
        "div",
        {
          key,
          "data-crepus-kind": "stack",
          "data-axis": row ? "row" : "column",
          style,
        },
        children,
      );
    }
    case "button": {
      const n = node as Extract<CrepusNode, { kind: "button" }>;
      return createElement(
        "button",
        {
          key,
          type: "button",
          "data-crepus-kind": "button",
          "data-onclick": n.onClick,
          style: styleOf(n),
          onClick: () => {
            if (n.onClick) options.onAction?.(n.onClick);
          },
        },
        n.label ?? "",
      );
    }
    case "sparkline": {
      const n = node as Extract<CrepusNode, { kind: "sparkline" }>;
      const width = n.width ?? 120;
      const height = n.height ?? 32;
      const values = n.values ?? [];
      const color = n.color ?? "currentColor";
      const d = sparklinePath(values, width, height);
      return createElement(
        "svg",
        {
          key,
          "data-crepus-kind": "sparkline",
          width,
          height,
          viewBox: `0 0 ${width} ${height}`,
          style: { display: "block", ...styleOf(n) },
          role: "img",
          "aria-label": "sparkline",
        },
        createElement("path", {
          d,
          fill: "none",
          stroke: color,
          strokeWidth: 1.5,
          strokeLinejoin: "round",
          strokeLinecap: "round",
        }),
      );
    }
    default:
      return createElement(
        "div",
        {
          key,
          "data-crepus-kind": String((node as { kind: string }).kind),
          "data-crepus-unknown": "true",
        },
        null,
      );
  }
}

/**
 * Map a minimal crepuscularity JSON IR tree to moonshine React elements.
 *
 * CLI emit imports this so `.crepus` → IR → React stays one hop:
 *
 * ```ts
 * import { renderCrepusIr, createMoonshineApp } from "@crepuscularity/moonshine";
 *
 * function App() {
 *   return renderCrepusIr({
 *     version: 1,
 *     root: [{ kind: "text", content: "hello" }],
 *   });
 * }
 * ```
 */
export function renderCrepusIr(
  ir: CrepusIr,
  options: RenderCrepusOptions = {},
): ReactElement {
  const prefix = options.keyPrefix ?? "crepus";
  const children = (ir.root ?? []).map((node, i) =>
    renderCrepusNode(node, options, `${prefix}.${i}`),
  );
  return createElement(
    "div",
    {
      "data-crepus-root": "true",
      "data-crepus-ir-version": ir.version ?? 1,
    },
    children,
  );
}
