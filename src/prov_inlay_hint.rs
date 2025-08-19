use crate::lang_types;
use crate::lsp_util::point_to_position;
use tower_lsp::lsp_types::*;
use tree_sitter::Node;

pub fn capabilities() -> OneOf<bool, InlayHintServerCapabilities> {
    return OneOf::Left(true);
}

fn process_call_expression(
    src: &str,
    node: Node,
    parse_state: &lang_types::ParseState,
    inlay_hints: &mut Vec<InlayHint>,
) -> Result<(), &'static str> {
    let function_node = node
        .child_by_field_name("function")
        .ok_or("missing call_expression function")?;
    let arguments_node = node
        .child_by_field_name("arguments")
        .ok_or("missing call_expression arguments")?;

    let function_name = function_node.utf8_text(src.as_bytes()).unwrap().to_string();
    if let Some(lf) = parse_state.functions.get(&function_name) {
        let mut param_counter = 0;
        for node in arguments_node.children(&mut node.walk()) {
            match node.kind() {
                "(" | "," | "ERROR" | ")" => (),
                _ => {
                    param_counter += 1;
                }
            }
        }

        // don't bother for single parameter hints, but still show the ? if the user provided more than 2 params
        if lf.params.len() < 2 && param_counter < 2 {
            return Ok(());
        }

        let mut param_counter = 0;
        for node in arguments_node.children(&mut node.walk()) {
            match node.kind() {
                "(" | "," | "ERROR" => (),
                ")" => {
                    for (param_name, _) in lf.params.iter().skip(param_counter) {
                        inlay_hints.push(InlayHint {
                            position: point_to_position(node.start_position()),
                            label: InlayHintLabel::String(param_name.to_owned() + ":"),
                            kind: Some(InlayHintKind::PARAMETER),
                            text_edits: None,
                            tooltip: None,
                            padding_left: Some(true),
                            padding_right: Some(true),
                            data: None,
                        });
                    }
                }
                _ => {
                    let label = match lf.params.iter().nth(param_counter) {
                        Some((param_name, _)) => param_name,
                        None => "?",
                    };

                    // don't render hint if the current argument is the hint label
                    if label != node.utf8_text(src.as_bytes()).unwrap().to_string() {
                        inlay_hints.push(InlayHint {
                            position: point_to_position(node.start_position()),
                            label: InlayHintLabel::String(label.to_owned() + ":"),
                            kind: Some(InlayHintKind::PARAMETER),
                            text_edits: None,
                            tooltip: None,
                            padding_left: Some(false),
                            padding_right: Some(true),
                            data: None,
                        });
                    }
                    param_counter += 1;
                }
            }
        }
    }

    return Ok(());
}

fn extract_inlay_hints_recursively(
    src: &str,
    node: Node,
    parse_state: &lang_types::ParseState,
    inlay_hints: &mut Vec<InlayHint>,
) {
    if node.kind() == "call_expression" {
        let _ = process_call_expression(src, node, parse_state, inlay_hints);
    }
    for child in node.children(&mut node.walk()) {
        extract_inlay_hints_recursively(src, child, parse_state, inlay_hints);
    }
}

pub fn get_inlay_hints(parse_state: &lang_types::ParseState) -> Vec<InlayHint> {
    let mut inlay_hints = vec![];
    if let Some(tree) = &parse_state.tree {
        extract_inlay_hints_recursively(
            &parse_state.text,
            tree.root_node(),
            parse_state,
            &mut inlay_hints,
        );
    }
    return inlay_hints;
}
