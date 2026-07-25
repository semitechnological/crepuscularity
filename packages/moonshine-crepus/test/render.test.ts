import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { renderCrepusIr } from "../src/render";

describe("renderCrepusIr", () => {
  test("renders text / stack / button / sparkline", () => {
    const el = renderCrepusIr({
      version: 1,
      root: [
        {
          kind: "stack",
          axis: "column",
          children: [
            { kind: "text", content: "Hello" },
            { kind: "button", label: "Click", onClick: "doThing" },
            { kind: "sparkline", values: [1, 2, 3], width: 40, height: 10 },
          ],
        },
      ],
    });

    const html = renderToStaticMarkup(el);
    expect(html).toContain("data-crepus-root");
    expect(html).toContain("Hello");
    expect(html).toContain("Click");
    expect(html).toContain('data-onclick="doThing"');
    expect(html).toContain("data-crepus-kind=\"sparkline\"");
    expect(html).toContain("<path");
  });

  test("unknown kinds render a stub", () => {
    const html = renderToStaticMarkup(
      renderCrepusIr({
        root: [{ kind: "picker", options: [] }],
      }),
    );
    expect(html).toContain('data-crepus-kind="picker"');
    expect(html).toContain("data-crepus-unknown");
  });
});
