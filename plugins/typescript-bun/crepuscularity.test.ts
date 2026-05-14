import { expect, test } from "bun:test"
import { renderHtml, renderIr } from "./crepuscularity"

test("renderIr decodes ViewIr", async () => {
  const fixture = new URL("../fixtures/hello.crepus", import.meta.url).pathname
  const ir = await renderIr(fixture, { name: "Ada" })
  expect(ir.version).toBe(3)
  expect(ir.root.length).toBe(1)
  expect(await renderHtml(fixture, { name: "Ada" })).toBe('<div data-crepus-kind="stack" data-axis="column">Hello Ada</div>')
})
