use super::Backend;
use tower_lsp::lsp_types::*;

impl Backend {
    // TODO lang specific - implement a diagnostics provider
    #[allow(unused_variables)]
    pub async fn generate_diagnostics(&self, text: &str, uri: Url, version: i32) {
        let mut items: Vec<Diagnostic> = vec![];

        items.push(Diagnostic {
            range: Range {
                start: Position {
                    line: 0 as u32,
                    character: 0 as u32,
                },
                end: Position {
                    line: 0 as u32,
                    character: 1 as u32,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: "Diagnostics provider not implemented!".to_owned(),
            source: Some("C-Style Lang Server".to_owned()), // TODO lang specific - set name here
            ..Default::default()
        });

        self.client
            .publish_diagnostics(uri, items, Some(version))
            .await;
    }
}
