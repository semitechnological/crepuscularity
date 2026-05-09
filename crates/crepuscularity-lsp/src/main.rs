use std::collections::HashMap;
use std::sync::Arc;

use crepuscularity_lsp::{completion_items, crepus_diagnostics_to_lsp, hover_for};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    docs: Arc<RwLock<HashMap<Url, String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "crepus-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    // `<`, ` `, and `{` cover the three contexts our completion
                    // logic recognizes (JSX tag, class slot, expression slot).
                    trigger_characters: Some(vec![
                        "<".to_string(),
                        " ".to_string(),
                        "{".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        self.update_doc(uri, text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(last) = params.content_changes.into_iter().last() {
            self.update_doc(uri, last.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut g = self.docs.write().await;
        g.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let docs = self.docs.read().await;
        let Some(text) = docs.get(&uri).cloned() else {
            return Ok(None);
        };
        drop(docs);
        let items = completion_items(&text, position);
        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let docs = self.docs.read().await;
        let Some(text) = docs.get(&uri).cloned() else {
            return Ok(None);
        };
        drop(docs);
        Ok(hover_for(&text, position))
    }
}

impl Backend {
    async fn update_doc(&self, uri: Url, text: String) {
        self.docs.write().await.insert(uri.clone(), text.clone());
        let diagnostics = crepus_diagnostics_to_lsp(&text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if !args.iter().any(|a| a == "--stdio") {
        eprintln!("crepus-lsp: pass --stdio (VS Code / Zed language client)");
        std::process::exit(2);
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        docs: Arc::new(RwLock::new(HashMap::new())),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
