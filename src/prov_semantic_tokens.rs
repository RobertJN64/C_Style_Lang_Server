use crate::lang_types;
use tower_lsp::lsp_types::*;
use tree_sitter::Node;

pub enum LangSemanticToken {
    FUNCTION,
    NUMBER,
    MACRO,
    PARAMETER,
    STRUCT,
}
const TOKEN_TYPES: [SemanticTokenType; 5] = [
    SemanticTokenType::FUNCTION,
    SemanticTokenType::NUMBER,
    SemanticTokenType::MACRO,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::STRUCT,
];

struct SimpleToken {
    row: usize,
    col: usize,
    len: usize,
    token_type: LangSemanticToken,
}

pub fn capabilities() -> SemanticTokensServerCapabilities {
    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
        legend: SemanticTokensLegend {
            token_types: TOKEN_TYPES.to_vec(),
            token_modifiers: vec![],
        },

        full: Some(SemanticTokensFullOptions::Bool(true)),
        ..Default::default()
    })
}

fn extract_sm_tokens_recursively(
    src: &str,
    node: Node,
    parse_state: &lang_types::ParseState,
    simple_tokens: &mut Vec<SimpleToken>,
) {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "identifier" {
            let name = child.utf8_text(src.as_bytes()).unwrap();
            if parse_state.functions.contains_key(name) {
                simple_tokens.push(SimpleToken {
                    row: child.start_position().row,
                    col: child.start_position().column,
                    len: name.len(),
                    token_type: LangSemanticToken::FUNCTION,
                });
            }
            if parse_state.defines.contains_key(name) {
                simple_tokens.push(SimpleToken {
                    row: child.start_position().row,
                    col: child.start_position().column,
                    len: name.len(),
                    token_type: LangSemanticToken::MACRO,
                });
            }
            // TODO - highlight vars within scope
        }
        if child.kind() == "number_literal" {
            let name = child.utf8_text(src.as_bytes()).unwrap();
            simple_tokens.push(SimpleToken {
                row: child.start_position().row,
                col: child.start_position().column,
                len: name.len(),
                token_type: LangSemanticToken::NUMBER,
            });
        }
        if node.kind() == "parameter_declaration" && child.kind() == "identifier" {
            let name = child.utf8_text(src.as_bytes()).unwrap();
            simple_tokens.push(SimpleToken {
                row: child.start_position().row,
                col: child.start_position().column,
                len: name.len(),
                token_type: LangSemanticToken::PARAMETER, // TODO - also highlight within scope, change color if unused
            });
        }
        if child.kind() == "type_identifier" {
            let name = child.utf8_text(src.as_bytes()).unwrap();
            if let Some(lt) = parse_state.types.get(name) {
                if !lt.builtin {
                    simple_tokens.push(SimpleToken {
                        row: child.start_position().row,
                        col: child.start_position().column,
                        len: name.len(),
                        token_type: LangSemanticToken::STRUCT,
                    });
                }
            }
        }

        extract_sm_tokens_recursively(src, child, parse_state, simple_tokens);
    }
}

pub fn get_sm_tokens(parse_state: &lang_types::ParseState) -> Vec<SemanticToken> {
    let mut simple_tokens = vec![];
    if let Some(tree) = &parse_state.tree {
        extract_sm_tokens_recursively(
            &parse_state.text,
            tree.root_node(),
            parse_state,
            &mut simple_tokens,
        );
    }

    let mut start_row = 0;
    let mut start_col = 0;
    let mut sm_tokens = vec![];

    for st in simple_tokens {
        let delta_line = st.row as u32 - start_row;
        let delta_start = if delta_line == 0 {
            st.col as u32 - start_col
        } else {
            st.col as u32
        };
        sm_tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: st.len as u32,
            token_type: st.token_type as u32,
            token_modifiers_bitset: 0,
        });
        start_row = st.row as u32;
        start_col = st.col as u32
    }

    return sm_tokens;
}
