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
