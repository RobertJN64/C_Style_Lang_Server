use crate::lang_types;
use tower_lsp::lsp_types::*;

#[allow(dead_code)]
pub fn capabilities() -> FoldingRangeProviderCapability {
    FoldingRangeProviderCapability::Simple(true)
}

pub fn get_folding_ranges(scope: &lang_types::Scope) -> Vec<FoldingRange> {
    let mut frs = vec![];

    for scope in scope.scopes.iter() {
        frs.push(FoldingRange {
            start_line: scope.0,
            start_character: None,
            end_line: scope.1,
            end_character: None,
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: None,
        });

        frs.extend(get_folding_ranges(&scope.2));
    }

    return frs;
}
