use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::lsp_types::{CompletionItemKind, Location, Position};
use tree_sitter::Tree;

//// Language Objects

// A typed object (variables, parameters, fields, etc.)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangVar {
    // type of the variable
    // for example, `const int my2dArr[5][5] -> 'int'
    // used for providing completions
    pub primary_type: String,

    // types qualifiers applied to this var
    // for example, `const int my2dArr[5][5] -> ['const', '[]', '[]']`
    // used for providing completions
    #[serde(default)] // default type_qualifier_list to [] if not provided in lang db
    pub type_qualifier_list: Vec<String>,

    // declaration location within the document
    // used for providing unusued variable warnings and goto definition
    pub declaration_position: Option<Location>,

    // set to true by default, set to false if used outside of its declaration
    // used for providing unusued variable warnings
    #[serde(default)] // default unused to false if not provided in lang db
    pub unused: bool,
}

fn serde_default_true() -> bool {
    true
}

// A builtin type or user defined struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangType {
    // fields that objects of this type have
    // used for providing completions
    #[serde(default)] // default fields to [] if not provided in lang db (for basic types)
    pub fields: HashMap<String, LangVar>,

    // declaration location within any document
    // used for providing goto definition
    pub declaration_position: Option<Location>,

    // human readable desc of type as a markdown string
    // used for hovers
    pub desc: String,

    // changes hover and semantic highlights
    #[serde(default = "serde_default_true")]
    pub builtin: bool,
}

// A builtin or user defined function
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangFunc {
    // Ordered list of function parameters
    // Note - parameters are also added as variables to the scope that the function creates
    // The variables in the scope are used for handling unused parameter warnings
    pub params: Vec<(String, LangVar)>,

    // declaration location within any document
    // used for providing goto definition
    pub declaration_position: Option<Location>,

    // human readable desc of type as a markdown string
    // used for hovers
    pub desc: String,
}

// A `#define` replacement macro
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangDefine {
    // text that will be inserted when preprocessor runs
    pub insert_text: String,

    // declaration location within any document
    // used for providing goto definition
    pub declaration_position: Option<Location>,
}

//// LangDB

// Holds information about language syntax and builtins, does not depend on document contents
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LangDB {
    // Builtin variables
    pub builtin_vars: HashMap<String, LangVar>,

    // Builtin types
    pub types: HashMap<String, LangType>,

    // Builtin functions
    pub functions: HashMap<String, LangFunc>,

    // Builtin defines
    pub defines: HashMap<String, LangDefine>,

    // Control keywords (if/else/while) - merged into ParseState keywords
    pub control: Vec<String>,

    // Constants (true/false) - merged into ParseState keywords
    pub constants: Vec<String>,

    // Preprocessor macros (#define) - merged into ParseState keywords
    pub preprocessor: Vec<String>,
}

//// Parse Outputs

// Holds Language Objects that are tied to a specific scope (such as a function or loop)
// Currently only LangVars, but LangFunc could be moved into here if needed for your language
// Scopes can be nested
#[derive(Debug, PartialEq)]
pub struct Scope {
    // Maps var name to LangVar object
    pub vars: HashMap<String, LangVar>,

    // Nested scopes (start line, end line, scope)
    pub scopes: Vec<(u32, u32, Scope)>,
}

// Holds information about the document state after parsing, based on the document text and the LangDB
#[derive(Debug)]
pub struct ParseState {
    // The raw text of the source file
    pub text: String,

    // The tree sitter tree (if parsing succeeds)
    pub tree: Option<Tree>,

    // LangTypes (builtin and user defined)
    pub types: HashMap<String, LangType>,

    // Functions (builtin and user defined)
    pub functions: HashMap<String, LangFunc>,

    // LangDefines (builtin and user defined)
    pub defines: HashMap<String, LangDefine>,

    // Keywords (with completion item kind, only for completions)
    pub keywords: Vec<(CompletionItemKind, String)>,

    // All scope specific objects are stored in nested scopes accessible from the global scope
    // builtin vars are placed in the global scope
    pub global_scope: Scope,
}

// Holds information about the document after resolving the active scope
#[derive(Debug)]
pub struct ScopedParseState<'src> {
    // The raw text of the source file
    pub text: &'src String,

    // The tree sitter tree (if parsing succeeds)
    pub tree: &'src Option<Tree>,

    // LangTypes (builtin and user defined)
    pub types: &'src HashMap<String, LangType>,

    // Functions (builtin and user defined)
    pub functions: &'src HashMap<String, LangFunc>,

    // LangDefines (builtin and user defined)
    pub defines: &'src HashMap<String, LangDefine>,

    // Keywords (with completion item kind, only for completions)
    pub keywords: &'src Vec<(CompletionItemKind, String)>,

    // LangVars available in the active scope
    pub vars: HashMap<String, LangVar>,
}

//// SPS Functions

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
