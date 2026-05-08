import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

function resolveServerExecutable(
  folder: vscode.WorkspaceFolder | undefined,
  configured: string
): string {
  if (configured.trim()) {
    return configured;
  }
  if (folder) {
    return path.join(folder.uri.fsPath, "target", "debug", "crepus-lsp");
  }
  return "crepus-lsp";
}

async function startClient(): Promise<void> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  const cfg = vscode.workspace.getConfiguration("crepus");
  const command = resolveServerExecutable(folder, cfg.get("languageServerPath", ""));
  const serverOptions: ServerOptions = {
    run: { command, args: ["--stdio"], transport: TransportKind.stdio },
    debug: { command, args: ["--stdio"], transport: TransportKind.stdio },
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "crepus" }],
  };
  client = new LanguageClient(
    "crepus-lsp",
    "Crepus Language Server",
    serverOptions,
    clientOptions
  );
  await client.start();
}

export function activate(context: vscode.ExtensionContext) {
  void startClient();
  context.subscriptions.push(
    vscode.commands.registerCommand("crepus.restartLanguageServer", async () => {
      if (client) {
        await client.stop();
        client = undefined;
      }
      await startClient();
    }),
    new vscode.Disposable(() => {
      void client?.stop();
    })
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
  }
}
