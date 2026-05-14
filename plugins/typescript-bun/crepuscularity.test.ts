import { expect, test } from "bun:test"
import { CrepusViewSession, renderHtml, renderIr } from "./crepuscularity"

test("renderIr decodes ViewIr", async () => {
  const fixture = new URL("../fixtures/hello.crepus", import.meta.url).pathname
  const ir = await renderIr(fixture, { name: "Ada" })
  expect(ir.version).toBe(3)
  expect(ir.root.length).toBe(1)
  expect(await renderHtml(fixture, { name: "Ada" })).toBe('<div data-crepus-kind="stack" data-axis="column">Hello Ada</div>')
})

test("CrepusViewSession dispatches bind events and rerenders", async () => {
  const fixture = new URL("../fixtures/interactive.crepus", import.meta.url).pathname
  const session = new CrepusViewSession(fixture, { count: "1" })
  expect(await session.renderHtml()).toContain("Count 1")
  const ir = await session.dispatch("bind:count:2")
  expect(JSON.stringify(ir)).toContain("Count 2")
  expect(await session.renderHtml()).toContain("Count 2")
})
