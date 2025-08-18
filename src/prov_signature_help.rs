use crate::{lang_types, lsp_util};
use tower_lsp::lsp_types::*;

pub fn capabilities() -> SignatureHelpOptions {
    SignatureHelpOptions {
        trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
        retrigger_characters: Some(vec![")".to_string()]),
        work_done_progress_options: WorkDoneProgressOptions {
            work_done_progress: None,
        },
    }
}
