use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::lsp_types::*;
use tree_sitter::Tree;

// holds information about typed objects (parameters, variables, fields, etc.)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangVar {
    pub type_list: Vec<String>, // types and modifiers applied to this var
}

// holds information about builtin types and user defined structs
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangType {
    pub builtin: bool,                    // True if not a user created struct
    pub desc: String,                     // human readable desc of type, for hovers
    pub fields: HashMap<String, LangVar>, // fields that objects of this type can take, for completions
}

// holds information about functions
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangFunc {
    pub builtin: bool, // True if not a user created functions
    pub desc: String,  // human readable desc of function, for hovers
}

// holds information about language regardless of document state
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LangDB {
    pub types: HashMap<String, LangType>,
    pub functions: HashMap<String, LangFunc>,

    // these get merged into keywords
    pub control: Vec<String>,
    pub constants: Vec<String>,
    pub preprocessor: Vec<String>,

    pub builtin_vars: HashMap<String, HashMap<String, LangVar>>,
}

// holds information about objects that only exist within scopes
#[derive(Debug, PartialEq)]
pub struct Scope {
    pub vars: HashMap<String, LangVar>,
    pub scopes: Vec<(u32, u32, Scope)>, // start line, end line, scope
}

// holds information about the document state after parsing, based on the document text and the LangDB
#[derive(Debug)]
pub struct ParseState {
    pub text: String,
    pub tree: Option<Tree>,

    pub types: HashMap<String, LangType>,
    pub functions: HashMap<String, LangFunc>,
    pub defines: Vec<String>,
    pub keywords: Vec<(CompletionItemKind, String)>,

    pub global_scope: Scope,
}

// holds information about the document after resolving the active scope
#[derive(Debug)]
#[allow(dead_code)]
pub struct ScopedParseState<'src> {
    pub text: &'src String,
    pub tree: &'src Option<Tree>,

    pub types: &'src HashMap<String, LangType>,
    pub functions: &'src HashMap<String, LangFunc>,
    pub defines: &'src Vec<String>,
    pub keywords: &'src Vec<(CompletionItemKind, String)>,

    pub vars: HashMap<String, LangVar>,
}

fn add_scoped_vars_recursive(
    active_scope: &Scope,
    loc: Position,
    vars: &mut HashMap<String, LangVar>,
) {
    vars.extend(active_scope.vars.clone());
    for scope in active_scope.scopes.iter() {
        if loc.line >= scope.0 && loc.line <= scope.1 {
            add_scoped_vars_recursive(&scope.2, loc, vars);
        }
    }
}

pub fn get_scoped_parse_state(ps: &ParseState, loc: Position) -> ScopedParseState {
    let mut vars = HashMap::new();
    add_scoped_vars_recursive(&ps.global_scope, loc, &mut vars);

    let sps = ScopedParseState {
        text: &ps.text,
        tree: &ps.tree,
        types: &ps.types,
        functions: &ps.functions,
        defines: &ps.defines,
        keywords: &ps.keywords,
        vars,
    };
    return sps;
}
