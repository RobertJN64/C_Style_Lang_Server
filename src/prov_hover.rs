use crate::{lang_types, lsp_util};
use tower_lsp::lsp_types::*;

pub fn capabilities() -> HoverProviderCapability {
    HoverProviderCapability::Simple(true)
}

pub fn get_hover(sps: &lang_types::ScopedParseState, position: Position) -> Option<Hover> {
    let word = lsp_util::extract_word_at(&sps.text, position);

    if let Some(lt) = sps.types.get(&word) {
        let mut desc = "### ".to_owned() + &word + "\n---\ntype with the following fields:\n\n";

        let mut fields: Vec<&String> = lt.fields.keys().collect();
        fields.sort(); // makes output deterministic
        for field in fields {
            desc += " - ";
            desc += field;
            desc += "\n\n";
        }
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: desc,
            }),
            range: None,
        });
    }

    if let Some(lf) = sps.functions.get(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "### ".to_owned() + &word + "\n---\n" + &lf.desc,
            }),
            range: None,
        });
    }

    if let Some(lv) = sps.vars.get(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: lv.type_list.join(" ") + " " + &word,
            }),
            range: None,
        });
    }

    return None;
}
