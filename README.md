# C Style Lang Server

A [language server](https://microsoft.github.io/language-server-protocol/) for "c-style" languages.

Language servers provide features such as syntax highlighting, completions, hovers, and diagnostics inside an editor (such as VS Code).

"C-Style" languages have variables with types that indicate what fields or functions are available on that object. Types can be provided by the language spec, or defined by the user (structs / classes). Functions are defined and called with parameters. For example, C, C++, GLSL, Go, Java and many more languages follow this general structure. Languages with very different syntax (such as assembly or FP languages) or more complex type systems (such as lifetimes in rust) will not be supported well.

## Features

 - Completions
   - Generic completions for keywords and language builtins
   - Smart completions based on variable type
 - Hovers
   - Hover for more information on functions, types, defines, and variables
 - Semantic Highlighting

## Syntax Representation
### Language Objects
#### LangVar
A typed object (variables, parameters, fields, etc.)

 - `type_list: Vec<String>`

types and modifiers applied to this var

for example, `int my2dArr[5][5] -> ['int', '[]', '[]']`

used for providing completions

 - `declaration_position: Position`

declaration location within the document

used for providing unusued variable warnings

 - `unused: bool`

set to true by default, set to false if used outside of its declaration

used for providing unusued variable warnings

#### LangType
A builtin type or user defined struct

 - `fields: HashMap<String, LangVar>`

fields that objects of this type have

used for providing completions

 - `desc: String`

human readable desc of type as a markdown string

used for hovers

 - `enable_semantic_highlighting: bool`

set to false if from LangDB, set to true if user created type

#### LangFunc
A builtin or user defined function

 - `params: Vec<(String, LangVar)>`

Ordered list of function parameters

Note - parameters are also added as variables to the scope that the function creates

The variables in the scope are used for handling unused parameter warnings

 - `desc: String`

human readable desc of type as a markdown string

used for hovers

 - `enable_semantic_highlighting: bool`

set to false if from LangDB, set to true if user created function

#### LangDefine
A `#define` replacement macro

 - `insert_text: String`

text that will be inserted when preprocessor runs

### LangDB
#### LangDB
Holds information about language syntax and builtins, does not depend on document contents

 - `builtin_vars: HashMap<String, LangVar>`

Builtin variables

 - `types: HashMap<String, LangType>`

Builtin types

 - `functions: HashMap<String, LangFunc>`

Builtin functions

 - `defines: HashMap<String, LangDefine>`

Builtin defines

 - `control: Vec<String>`

Control keywords (if/else/while) - merged into ParseState keywords

 - `constants: Vec<String>`

Constants (true/false) - merged into ParseState keywords

 - `preprocessor: Vec<String>`

Preprocessor macros (#define) - merged into ParseState keywords

### Parse Outputs
#### Scope
Holds Language Objects that are tied to a specific scope (such as a function or loop)

Currently only LangVars, but LangFunc could be moved into here if needed for your language

Scopes can be nested

 - `vars: HashMap<String, LangVar>`

Maps var name to LangVar object

 - `scopes: Vec<(u32, u32, Scope)>`

Nested scopes (start line, end line, scope)

#### ParseState
Holds information about the document state after parsing, based on the document text and the LangDB

 - `text: String`

The raw text of the source file

 - `tree: Option<Tree>`

The tree sitter tree (if parsing succeeds)

 - `types: HashMap<String, LangType>`

LangTypes (builtin and user defined)

 - `functions: HashMap<String, LangFunc>`

Functions (builtin and user defined)

 - `defines: HashMap<String, LangDefine>`

LangDefines (builtin and user defined)

 - `keywords: Vec<(CompletionItemKind, String)>`

Keywords (with completion item kind, only for completions)

 - `global_scope: Scope`

All scope specific objects are stored in nested scopes accessible from the global scope

builtin vars are placed in the global scope

#### ScopedParseState
Holds information about the document after resolving the active scope

 - `text: &'src String`

The raw text of the source file

 - `tree: &'src Option<Tree>`

The tree sitter tree (if parsing succeeds)

 - `types: &'src HashMap<String, LangType>`

LangTypes (builtin and user defined)

 - `functions: &'src HashMap<String, LangFunc>`

Functions (builtin and user defined)

 - `defines: &'src HashMap<String, LangDefine>`

LangDefines (builtin and user defined)

 - `keywords: &'src Vec<(CompletionItemKind, String)>`

Keywords (with completion item kind, only for completions)

 - `vars: HashMap<String, LangVar>`

LangVars available in the active scope

