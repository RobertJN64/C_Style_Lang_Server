use crate::lang_types;
use tower_lsp::lsp_types::*;

pub fn capabilities() -> CodeLensOptions {
    return CodeLensOptions {
        resolve_provider: Some(false), // we send the full code action right away
    };
}

pub fn get_code_lenses(
    parse_state: &lang_types::ParseState,
    text_document: TextDocumentIdentifier,
) -> Vec<CodeLens> {
    let mut code_lenses = vec![];

    for (func_name, func) in parse_state.functions.iter() {
        if let Some(dec_pos) = &func.declaration_position {
            if dec_pos.uri != text_document.uri {
                continue;
            }
            if func_name == "main" {
                // TODO lang specific - setup a reasonable command runner in the extension.ts file
                code_lenses.push(CodeLens {
                    range: dec_pos.range,
                    command: Some(Command {
                        title: "▶ Run".to_string(),
                        command: "cstyle-lang-server.runMain".to_string(),
                        arguments: None,
                    }),
                    data: None,
                });
            } else {
                code_lenses.push(CodeLens {
                    range: dec_pos.range,
                    command: Some(Command {
                        title: func.references.len().to_string() + " references",
                        command: "cstyle-lang-server.showReferences".to_string(), // need a custom function to convert the types in TS
                        arguments: Some(vec![
                            serde_json::to_value(text_document.uri.clone()).unwrap(),
                            serde_json::to_value(dec_pos.range.start).unwrap(),
                            serde_json::to_value(func.references.clone()).unwrap(),
                        ]),
                    }),

                    data: None,
                });
            }
        }
    }

    //log::debug!("{:#?}", code_lenses);
    return code_lenses;
}
