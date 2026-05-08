"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
function resolveServerExecutable(folder, configured) {
    if (configured.trim()) {
        return configured;
    }
    if (folder) {
        return path.join(folder.uri.fsPath, "target", "debug", "crepus-lsp");
    }
    return "crepus-lsp";
}
async function startClient() {
    const folder = vscode.workspace.workspaceFolders?.[0];
    const cfg = vscode.workspace.getConfiguration("crepus");
    const command = resolveServerExecutable(folder, cfg.get("languageServerPath", ""));
    const serverOptions = {
        run: { command, args: ["--stdio"], transport: node_1.TransportKind.stdio },
        debug: { command, args: ["--stdio"], transport: node_1.TransportKind.stdio },
    };
    const clientOptions = {
        documentSelector: [{ scheme: "file", language: "crepus" }],
    };
    client = new node_1.LanguageClient("crepus-lsp", "Crepus Language Server", serverOptions, clientOptions);
    await client.start();
}
function activate(context) {
    void startClient();
    context.subscriptions.push(vscode.commands.registerCommand("crepus.restartLanguageServer", async () => {
        if (client) {
            await client.stop();
            client = undefined;
        }
        await startClient();
    }), new vscode.Disposable(() => {
        void client?.stop();
    }));
}
async function deactivate() {
    if (client) {
        await client.stop();
    }
}
