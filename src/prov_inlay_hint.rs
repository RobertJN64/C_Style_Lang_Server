use crate::{lang_types, lsp_util};
use tower_lsp::lsp_types::*;

pub fn capabilities() -> OneOf<bool, InlayHintServerCapabilities> {
    return OneOf::Left(true);
}
