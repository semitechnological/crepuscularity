import { spawnSync } from "node:child_process"
import { isAbsolute, basename } from "node:path"

export interface ViewIr {
  version: number
  root: Array<Record<string, unknown>>
}

export function crepusBin(): string {
  const bin = process.env.CREPUS_BIN ?? "crepus"
  if (!isAbsolute(bin) && basename(bin) !== bin) {
    throw new Error(`Security Error: CREPUS_BIN path must be absolute or a valid binary name, got: ${bin}`)
  }
  return bin
}

const BIND_BLOCKLIST = new Set<string>(["baseDir", "_"]) // ponytail: block security-sensitive keys only

export async function renderIr(path: string, context: Record<string, unknown> = {}): Promise<ViewIr> {
  const template = await Bun.file(path).text()
  const proc = spawnSync(crepusBin(), ["native", "ir", "--stdin-json"], {
    input: JSON.stringify({ template, context }),
    encoding: "utf8"
  })
  if (proc.status !== 0) {
    throw new Error(proc.stderr)
  }
  return JSON.parse(proc.stdout) as ViewIr
}

export async function renderHtml(path: string, context: Record<string, unknown> = {}): Promise<string> {
  const ir = await renderIr(path, context)
  return ir.root.map(renderNode).join("")
}

export type EventPayload = {
  handler: string
  payload?: unknown
}

export type EventHandler = (event: EventPayload, session: CrepusViewSession) => void | Promise<void>

export class CrepusViewSession {
  readonly path: string
  context: Record<string, unknown>
  private handlers = new Map<string, EventHandler>()

  constructor(path: string, context: Record<string, unknown> = {}) {
    this.path = path
    this.context = { ...context }
  }

  on(handler: string, callback: EventHandler): this {
    this.handlers.set(handler, callback)
    return this
  }

  async renderIr(): Promise<ViewIr> {
    return renderIr(this.path, this.context)
  }

  async renderHtml(): Promise<string> {
    const ir = await this.renderIr()
    return ir.root.map(renderNode).join("")
  }

  async dispatch(event: string | EventPayload): Promise<ViewIr> {
    const parsed = typeof event === "string" ? { handler: event } : event
    if (parsed.handler.startsWith("bind:")) {
      const [, key, ...rest] = parsed.handler.split(":")
      if (!BIND_BLOCKLIST.has(key)) {
        this.context[key] = rest.join(":")
      }
    }
    const callback = this.handlers.get(parsed.handler)
    if (callback) {
      await callback(parsed, this)
    }
    return this.renderIr()
  }
}

function renderNode(node: Record<string, unknown>): string {
  switch (node.kind) {
    case "text":
      return escapeHtml(String(node.content ?? ""))
    case "stack":
    case "scroll":
      return `<div data-crepus-kind="${escapeAttr(String(node.kind))}" data-axis="${escapeAttr(String(node.axis ?? "column"))}">${children(node)}</div>`
    case "button": {
      const label = escapeHtml(String(node.label ?? ""))
      const onClick = node.onClick ? ` data-onclick="${escapeAttr(String(node.onClick))}"` : ""
      return `<button${onClick}>${label}</button>`
    }
    case "image":
      return `<img src="${escapeAttr(String(node.src ?? ""))}" alt="${escapeAttr(String(node.alt ?? ""))}">`
    case "slotRotate":
      return `<span data-crepus-kind="slotRotate">${escapeHtml(String((node.phrases as unknown[] | undefined)?.[0] ?? ""))}</span>`
    case "input":
      return node.multiline
        ? `<textarea data-bind="${escapeAttr(String(node.bind ?? ""))}" placeholder="${escapeAttr(String(node.placeholder ?? ""))}"></textarea>`
        : `<input data-bind="${escapeAttr(String(node.bind ?? ""))}" placeholder="${escapeAttr(String(node.placeholder ?? ""))}">`
    case "picker":
      return `<select data-bind="${escapeAttr(String(node.bind ?? ""))}">${(node.options as Array<Record<string, unknown>> | undefined ?? []).map((opt) => `<option value="${escapeAttr(String(opt.value ?? ""))}">${escapeHtml(String(opt.label ?? ""))}</option>`).join("")}</select>`
    default:
      return ""
  }
}

function children(node: Record<string, unknown>): string {
  return (node.children as Array<Record<string, unknown>> | undefined ?? []).map(renderNode).join("")
}

function escapeHtml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;")
}

function escapeAttr(value: string): string {
  return escapeHtml(value).replaceAll('"', "&quot;")
}
