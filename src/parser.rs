use crate::lang_types::*;
use std::collections::HashMap;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Parser, Point};
use tree_sitter_c;

fn point_to_position(point: Point) -> Position {
    return Position {
        line: point.row as u32,
        character: point.column as u32,
    };
}

fn node_to_location(node: Node, uri: &Url) -> Location {
    return Location {
        uri: uri.to_owned(),
        range: Range {
            start: point_to_position(node.start_position()),
            end: point_to_position(node.end_position()),
        },
    };
}

fn process_struct(src: &str, node: Node, uri: &Url) -> Result<(String, LangType), &'static str> {
    let struct_name_node = node
        .child_by_field_name("name")
        .ok_or("Missing Struct Name")?;
    let struct_name = struct_name_node.utf8_text(src.as_bytes()).unwrap();
    let body_node = node
        .child_by_field_name("body")
        .ok_or("Missing Struct Body")?;

    let mut fields = HashMap::new();
    for field_declaration in body_node.children(&mut body_node.walk()) {
        if let Ok((k, v)) = process_declaration(src, field_declaration, uri) {
            fields.insert(k, v);
        }
    }

    return Ok((
        struct_name.to_string(),
        LangType {
            fields,
            declaration_position: Some(node_to_location(struct_name_node, uri)),
            desc: "struct".to_string(),
            builtin: false,
        },
    ));
}

fn process_declarator(
    src: &str,
    node: Node,
    uri: &Url,
    type_list: &mut Vec<String>,
) -> Result<(String, Location), &'static str> {
    let identifier;

    match node.kind() {
        "identifier" | "field_identifier" => {
            identifier = (
                node.utf8_text(src.as_bytes()).unwrap().to_string(),
                node_to_location(node, uri),
            )
        }
        "array_declarator" => {
            let declarator_node = node
                .child_by_field_name("declarator")
                .ok_or("missing array_declarator declarator")?;
            identifier = process_declarator(src, declarator_node, uri, type_list)?;
            type_list.push("[]".to_owned());
        }
        "init_declarator" => {
            let declarator_node = node
                .child_by_field_name("declarator")
                .ok_or("missing array_declarator declarator")?;
            identifier = process_declarator(src, declarator_node, uri, type_list)?;
        }
        _ => return Err("unexpected node kind"),
    }

    return Ok(identifier);
}

// TODO - multiple declarations?
// TODO - type qualifiers?
// can process an input like "double x = 5; vec3 x; vec4 x[2]; in body, array, or function header"
fn process_declaration(
    src: &str,
    node: Node,
    uri: &Url,
) -> Result<(String, LangVar), &'static str> {
    let declarator_node = node
        .child_by_field_name("declarator")
        .ok_or("missing declaration declarator")?;
    let type_node = node
        .child_by_field_name("type")
        .ok_or("missing declaration type")?;

    let primary_type = type_node.utf8_text(src.as_bytes()).unwrap().to_string();
    let mut type_qualifier_list = vec![];
    let (identifier, location) =
        process_declarator(src, declarator_node, uri, &mut type_qualifier_list)?;

    return Ok((
        identifier.to_string(),
        LangVar {
            primary_type,
            type_qualifier_list,
            declaration_position: Some(location),
            unused: true,
        },
    ));
}

fn extract_recursively(
    src: &str,
    node: Node,
    uri: &Url,
    types: &mut HashMap<String, LangType>,
    functions: &mut HashMap<String, LangFunc>,
    defines: &mut HashMap<String, LangDefine>,
    active_scope: &mut Scope,
) {
    if node.kind() == "declaration" || node.kind() == "parameter_declaration" {
        if let Ok((name, lv)) = process_declaration(src, node, uri) {
            active_scope.vars.insert(name, lv);
        }
    } else if node.kind() == "struct_specifier" {
        if let Ok((name, lt)) = process_struct(src, node, uri) {
            types.insert(name, lt);
        }
    }

    for child in node.children(&mut node.walk()) {
        if child.kind() == "identifier" {
            if node.kind() == "function_declarator" {
                let func_name = child.utf8_text(src.as_bytes()).unwrap();
                functions.insert(
                    func_name.to_owned(),
                    LangFunc {
                        params: vec![], // TODO - params
                        declaration_position: Some(node_to_location(child, uri)),
                        desc: "".to_owned(), // TODO - grab surrounding comments for desc
                    },
                );
            } else if node.kind() == "preproc_def" {
                let define_name = child.utf8_text(src.as_bytes()).unwrap();
                defines.insert(
                    define_name.to_owned(),
                    LangDefine {
                        insert_text: "".to_owned(), // TODO - grab the replacement text
                        declaration_position: Some(node_to_location(child, uri)),
                    },
                );
            } else {
                // log::debug!(
                //     "{} {}",
                //     node.kind(),
                //     child.utf8_text(src.as_bytes()).unwrap()
                // );
            }
        }

        if child.kind() == "function_definition" {
            let mut sub_scope = Scope {
                vars: HashMap::new(),
                scopes: vec![],
            };
            extract_recursively(src, child, uri, types, functions, defines, &mut sub_scope);
            active_scope.scopes.push((
                child.start_position().row as u32,
                child.end_position().row as u32,
                sub_scope,
            ));
        } else {
            extract_recursively(src, child, uri, types, functions, defines, active_scope);
        }
    }
}

pub fn parse(text: String, uri: &Url, lang_db: &LangDB) -> ParseState {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .expect("Failed to load grammar");
    let tree = parser.parse(&text, None);

    let mut types = lang_db.types.clone(); // TODO (perf) - this clone is not needed
    let mut functions = lang_db.functions.clone();
    let mut defines = lang_db.defines.clone();
    let mut keywords = vec![];
    let mut global_scope = Scope {
        vars: lang_db.builtin_vars.clone(),
        scopes: vec![],
    };

    // TODO (perf) - move these into one time startup
    for label in lang_db.constants.iter() {
        keywords.push((CompletionItemKind::CONSTANT, label.to_owned()));
    }
    for label in lang_db.preprocessor.iter() {
        keywords.push((CompletionItemKind::KEYWORD, label.to_owned()));
    }
    for label in lang_db.control.iter() {
        keywords.push((CompletionItemKind::KEYWORD, label.to_owned()));
    }

    if let Some(tree) = &tree {
        //log::debug!("{:#?}", tree.root_node().to_sexp());
        extract_recursively(
            &text,
            tree.root_node(),
            uri,
            &mut types,
            &mut functions,
            &mut defines,
            &mut global_scope,
        );
    }

    let ps = ParseState {
        text,
        tree,
        types,
        functions,
        defines,
        keywords,
        global_scope,
    };

    //log::debug!("{:#?}", ps);
    return ps;
}

#[allow(dead_code)]
pub fn print_tree(src: &str, node: Node, depth: usize, field_name: Option<&str>) {
    if !node.kind().chars().any(|c| c.is_alphanumeric()) {
        return; // skips over nodes like ';'
    }

    let fmt_contents = {
        let l: Vec<_> = node.utf8_text(src.as_bytes()).unwrap().lines().collect();
        match l.len() {
            0 => "".to_string(),
            1 => l[0].to_string(),
            2 => format!("{}\n{}", l[0], l[1]),
            _ => format!("{}...{}", l[0], l[l.len() - 1].trim()),
        }
    };

    match field_name {
        Some(field_name) => {
            println!(
                "{}{:#?} {} {:#?}",
                std::iter::repeat(" ").take(depth).collect::<String>(),
                field_name,
                node.kind(),
                fmt_contents
            );
        }
        None => {
            println!(
                "{}{} {:#?}",
                std::iter::repeat(" ").take(depth).collect::<String>(),
                node.kind(),
                fmt_contents
            );
        }
    }

    for (ind, child) in node.children(&mut node.walk()).enumerate() {
        print_tree(src, child, depth + 2, node.field_name_for_child(ind as u32));
    }
}
