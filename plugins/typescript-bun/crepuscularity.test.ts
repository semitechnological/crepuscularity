import { expect, test } from "bun:test"
import { renderIr } from "./crepuscularity"

test("renderIr decodes ViewIr", async () => {
  const fixture = new URL("../fixtures/hello.crepus", import.meta.url).pathname
  const ir = await renderIr(fixture, { name: "Ada" })
  expect(ir.version).toBe(2)
  expect(ir.root.length).toBe(1)
})
