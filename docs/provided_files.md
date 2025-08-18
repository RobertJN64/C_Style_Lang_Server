# Provided Files

All files included in this project, along with notes about language specific changes.
Search for "TODO lang specific" to find needed changes.
Search for "c-style" and "cstyle" to customize names.

## Language Configuration Files

`lang_db.json` - single source of truth for language configuration, contains information about builtins and keywords

`syntaxes/cstyle.tmLanguage.json` - provides basic syntax highlighting, partially generated using `gen_syntax.py` from the `lang_db.json` file

`language-configuration.json` - provides features like comment and string detection, bracket highlighting, and autoclosing

## VSCode Extension Files

`.vscode/launch.json` - configures the extension development mode, such as the default folder that will open

`client/extension.ts` - the VSCode extension, launches the language server and provides formatting with the included `clang-format.exe`

`package.json` - the VSCode extension information, **must be customized to your language and for publishing**, tells VSCode what file extension to use (defaults to `.cstyle`)

`tsconfig.json` - configures typescript compiler

## Language Server Files

`src/main.rs` the core langu

`lang_types.rs` - provides the language objects (variables, functions, etc.) that the rest of the langauge server uses, as well as utils for handling scopes

`src/parser.rs` - uses tree-sitter to convert document text into the objects described in `lang_types.rs`, may need to be modified with new tree-sitter grammar and corresponding parsing changes


`src/diagnostics.rs` - provides diagnostics, will need to be customized for your language


`src/prov_*.rs` - the language server feature providers, shouldn't need to be modified unless modifications are made to `lang_types.rs`

`src/lsp_util.rs` - util functions for extracting words from document

`Cargo.toml` describes rust dependencies