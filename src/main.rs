use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;

use log::debug;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod diagnostics;
mod lang_types;
mod lsp_util;
mod parser;
mod prov_completions;
mod prov_folding;
mod prov_goto;
mod prov_hover;
mod prov_semantic_tokens;

struct Backend {
    client: Client,
    lang_db: lang_types::LangDB,
    documents: RwLock<HashMap<Url, lang_types::ParseState>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                completion_provider: Some(prov_completions::capabilities()),
                hover_provider: Some(prov_hover::capabilities()),
                semantic_tokens_provider: Some(prov_semantic_tokens::capabilities()),
                definition_provider: Some(prov_goto::definition_capabilities()),
                type_definition_provider: Some(prov_goto::type_definition_capabilities()),

                //folding_range_provider: Some(prov_folding::capabilities()), // the default indentation based is better
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        debug!("initialized!");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let rw_guard = self.documents.read().await;
        let parse_state = rw_guard.get(&params.text_document_position.text_document.uri);
        match parse_state {
            Some(parse_state) => Ok(Some(CompletionResponse::Array(
                prov_completions::get_completions(
                    &lang_types::get_scoped_parse_state(
                        parse_state,
                        params.text_document_position.position,
                    ),
                    params.text_document_position.position,
                ),
            ))),
            None => Ok(None),
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("file opened");
        self.documents.write().await.insert(
            params.text_document.uri.clone(),
            parser::parse(
                params.text_document.text.clone(),
                &params.text_document.uri,
                &self.lang_db,
            ),
        );

        self.generate_diagnostics(
            &params.text_document.text,
            params.text_document.uri,
            params.text_document.version,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.documents.write().await.insert(
            params.text_document.uri.clone(),
            parser::parse(
                params.content_changes[0].text.clone(),
                &params.text_document.uri,
                &self.lang_db,
            ),
        );

        self.generate_diagnostics(
            &params.content_changes[0].text, // 0 b/c TextDocumentSyncKind::FULL
            params.text_document.uri,
            params.text_document.version,
        )
        .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.generate_diagnostics("", params.text_document.uri, 0)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let rw_guard = self.documents.read().await;
        let parse_state = rw_guard.get(&params.text_document_position_params.text_document.uri);
        match parse_state {
            Some(parse_state) => Ok(prov_hover::get_hover(
                &lang_types::get_scoped_parse_state(
                    parse_state,
                    params.text_document_position_params.position,
                ),
                params.text_document_position_params.position,
            )),
            None => Ok(None),
        }
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let rw_guard = self.documents.read().await;
        let parse_state = rw_guard.get(&params.text_document.uri);

        match parse_state {
            Some(parse_state) => Ok(Some(prov_folding::get_folding_ranges(
                &parse_state.global_scope,
            ))),
            None => Ok(None),
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let rw_guard = self.documents.read().await;
        let parse_state = rw_guard.get(&params.text_document.uri);

        match parse_state {
            Some(parse_state) => Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: prov_semantic_tokens::get_sm_tokens(parse_state),
            }))),
            None => Ok(None),
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let rw_guard = self.documents.read().await;
        let parse_state = rw_guard.get(&params.text_document_position_params.text_document.uri);

        match parse_state {
            Some(parse_state) => Ok(prov_goto::goto_definition(
                &lang_types::get_scoped_parse_state(
                    parse_state,
                    params.text_document_position_params.position,
                ),
                params.text_document_position_params.position,
            )),
            None => Ok(None),
        }
    }

    async fn goto_type_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let rw_guard = self.documents.read().await;
        let parse_state = rw_guard.get(&params.text_document_position_params.text_document.uri);

        match parse_state {
            Some(parse_state) => Ok(prov_goto::goto_type_definition(
                &lang_types::get_scoped_parse_state(
                    parse_state,
                    params.text_document_position_params.position,
                ),
                params.text_document_position_params.position,
            )),
            None => Ok(None),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // TODO - fix this path
    let json_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lang_db.json");

    let file = File::open(json_path).unwrap();
    let reader = BufReader::new(file);

    // Parse the JSON
    let lang_db: lang_types::LangDB = serde_json::from_reader(reader).unwrap();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        lang_db,
        documents: RwLock::new(HashMap::new()),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
