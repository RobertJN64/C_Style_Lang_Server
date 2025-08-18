use crate::{lang_types, lsp_util};
use tower_lsp::lsp_types::*;

pub fn definition_capabilities() -> OneOf<bool, DefinitionOptions> {
    return OneOf::Left(true);
}

pub fn goto_definition(
    sps: &lang_types::ScopedParseState,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let word = lsp_util::extract_word_at(&sps.text, position);

    if let Some(lf) = sps.functions.get(&word) {
        match &lf.declaration_position {
            Some(loc) => return Some(GotoDefinitionResponse::Scalar(loc.clone())),
            None => return None,
        }
    }
    if let Some(lv) = sps.vars.get(&word) {
        match &lv.declaration_position {
            Some(loc) => return Some(GotoDefinitionResponse::Scalar(loc.clone())),
            None => return None,
        }
    }
    if let Some(lt) = sps.types.get(&word) {
        match &lt.declaration_position {
            Some(loc) => return Some(GotoDefinitionResponse::Scalar(loc.clone())),
            None => return None,
        }
    }
    if let Some(ld) = sps.defines.get(&word) {
        match &ld.declaration_position {
            Some(loc) => return Some(GotoDefinitionResponse::Scalar(loc.clone())),
            None => return None,
        }
    }
    return None;
}

pub fn type_definition_capabilities() -> TypeDefinitionProviderCapability {
    return TypeDefinitionProviderCapability::Simple(true);
}

pub fn goto_type_definition(
    sps: &lang_types::ScopedParseState,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let word = lsp_util::extract_word_at(&sps.text, position);

    if let Some(lv) = sps.vars.get(&word) {
        if let Some(lt) = sps.types.get(&lv.primary_type) {
            match &lt.declaration_position {
                Some(loc) => return Some(GotoDefinitionResponse::Scalar(loc.clone())),
                None => return None,
            }
        }
    }
    return None;
}
