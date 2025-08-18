use std::collections::HashMap;

use crate::{lang_types, lsp_util};
use tower_lsp::lsp_types::*;

pub fn capabilities() -> CompletionOptions {
    CompletionOptions {
        trigger_characters: Some(vec![".".to_owned()]),
        ..Default::default()
    }
}

fn add_basic_completions(sps: &lang_types::ScopedParseState, items: &mut Vec<CompletionItem>) {
    // TODO - preprocessor might not work reliably with # starter

    for (cik, label) in sps.keywords.iter() {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(cik.to_owned()),
            ..Default::default()
        });
    }

    for label in sps.functions.keys() {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            ..Default::default()
        });
    }

    for label in sps.types.keys() {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    for (label, _) in sps.defines.iter() {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            ..Default::default()
        });
    }

    for label in sps.vars.keys() {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            ..Default::default()
        });
    }
}

fn add_field_completions(
    items: &mut Vec<CompletionItem>,
    fields: &HashMap<String, lang_types::LangVar>,
) {
    for component in fields.keys() {
        items.push(CompletionItem {
            label: component.to_owned(),
            kind: Some(CompletionItemKind::FIELD),
            ..Default::default()
        });
    }
}

fn add_ident_completions(
    items: &mut Vec<CompletionItem>,
    active_ident: &lang_types::LangVar,
    remaining_fields: &mut Vec<String>,
    sps: &lang_types::ScopedParseState,
) {
    let mut array_count = 0;

    while let Some("[]") = remaining_fields.last().map(|s| s.as_str()) {
        array_count += 1;
        remaining_fields.pop();
    }

    let expected_array_count = active_ident
        .type_list
        .iter()
        .filter(|ident| *ident == "[]")
        .count();

    if array_count > expected_array_count {
        return; // too many array accesses
    }
    if array_count < expected_array_count {
        // we have an array
        items.push(CompletionItem {
            label: "length()".to_string(),
            kind: Some(CompletionItemKind::FIELD),
            ..Default::default()
        });
        return;
    }

    for var_type in active_ident.type_list.iter() {
        if let Some(lt) = sps.types.get(var_type) {
            if remaining_fields.len() == 0 {
                add_field_completions(items, &lt.fields);
            } else {
                if let Some(field) = lt.fields.get(&remaining_fields.pop().unwrap()) {
                    add_ident_completions(items, field, remaining_fields, sps);
                }
            }
        }
    }
}

pub fn get_completions(
    sps: &lang_types::ScopedParseState,
    position: Position,
) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = vec![];

    let mut idents = lsp_util::extract_identifier_sequence(&sps.text, position);
    match idents.pop() {
        Some(base_ident) => {
            if let Some(active_ident) = sps.vars.get(&base_ident) {
                add_ident_completions(&mut items, active_ident, &mut idents, &sps);
            }
        }
        None => add_basic_completions(sps, &mut items),
    }

    return items;
}
