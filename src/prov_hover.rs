use crate::{lang_types, lsp_util};
use tower_lsp::lsp_types::*;

pub fn capabilities() -> HoverProviderCapability {
    HoverProviderCapability::Simple(true)
}

pub fn get_hover(sps: &lang_types::ScopedParseState, position: Position) -> Option<Hover> {
    let word = lsp_util::extract_word_at(&sps.text, position);

    if let Some(lt) = sps.types.get(&word) {
        let mut desc = "### ".to_owned() + &word + "\n---\n";
        if lt.builtin {
            desc += &("builtin type\n\n".to_owned() + &lt.desc);
        } else {
            desc += "user defined struct"
        }

        let mut fields: Vec<&String> = lt.fields.keys().collect();
        if fields.len() > 0 {
            desc += "\n\nfields:\n";
            fields.sort(); // makes output deterministic
            for field in fields {
                desc += " - ";
                desc += field;
                desc += "\n\n";
            }
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
        let mut desc = "### ".to_owned() + &word + "\n---\n" + &lf.desc;

        if lf.params.len() > 0 {
            desc += "\n\nparams:\n";
            for (param_name, _) in lf.params.iter() {
                desc += " - ";
                desc += param_name;
                desc += "\n\n";
            }
        }

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: desc,
            }),
            range: None,
        });
    }

    if let Some(lv) = sps.vars.get(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: lv.primary_type.to_owned()
                    + " "
                    + &lv.type_qualifier_list.join(" ")
                    + " "
                    + &word,
            }),
            range: None,
        });
    }

    return None;
}
