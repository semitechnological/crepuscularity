import { spawnSync } from "node:child_process"

export interface ViewIr {
  version: number
  root: Array<Record<string, unknown>>
}

function crepusBin(): string {
  return process.env.CREPUS_BIN ?? "crepus"
}

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
