#[cfg(test)]
mod tests {
    use crate::lang_types::*;
    use crate::lsp_util;
    use crate::parser;
    use crate::prov_completions;
    use crate::prov_folding;
    use crate::prov_hover;
    use crate::prov_semantic_tokens;
    use std::collections::HashMap;
    use std::env;
    use std::fs::File;
    use std::io::BufReader;
    use tower_lsp::lsp_types::*;

    fn location_of(src: &str, item: &str, uri: &Url) -> Location {
        for (line_idx, line) in src.lines().enumerate() {
            if let Some(col) = line.find(item) {
                return Location {
                    uri: uri.to_owned(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (col + item.len()) as u32,
                        },
                    },
                };
            }
        }
        panic!("Couldn't find {}", item);
    }

    fn shared_sample_code() -> (LangDB, &'static str, Url) {
        let empty_lang_db = LangDB {
            types: HashMap::new(),
            functions: HashMap::new(),
            defines: HashMap::new(),
            control: vec![],
            constants: vec![],
            preprocessor: vec![],
            builtin_vars: HashMap::new(),
        };

        let sample_code = r#"

        #define myRep 10

        const double global_var;

        struct MyStruct {
            vec3 myField;
            double arrayField[2];
        };

        void main(vec2 param_var) {
            const float init_var = 4;
            vec4 prim_var;
            MyStruct cust_var;
            float array_var[2][3];
        }
        "#;

        let sample_uri = Url::parse("https://sample.com").unwrap();
        return (empty_lang_db, sample_code, sample_uri);
    }

    #[test]
    fn validate_parsing() {
        let (empty_lang_db, sample_code, sample_uri) = shared_sample_code();
        let result = parser::parse(sample_code.to_owned(), &sample_uri, &empty_lang_db);

        let param_var = (
            "param_var".to_string(),
            LangVar {
                primary_type: "vec2".to_string(),
                type_qualifier_list: vec![],
                declaration_position: Some(location_of(sample_code, "param_var", &sample_uri)),
                unused: true,
            },
        );

        let mut expected_types = HashMap::new();
        expected_types.insert(
            "MyStruct".to_owned(),
            LangType {
                builtin: false,
                desc: "struct".to_owned(),
                fields: HashMap::from([
                    (
                        "myField".to_string(),
                        LangVar {
                            primary_type: "vec3".to_string(),
                            type_qualifier_list: vec![],
                            declaration_position: Some(location_of(
                                sample_code,
                                "myField",
                                &sample_uri,
                            )),
                            unused: true,
                        },
                    ),
                    (
                        "arrayField".to_string(),
                        LangVar {
                            primary_type: "double".to_string(),
                            type_qualifier_list: vec!["[]".to_string()],
                            declaration_position: Some(location_of(
                                sample_code,
                                "arrayField",
                                &sample_uri,
                            )),
                            unused: true,
                        },
                    ),
                ]),
                declaration_position: Some(location_of(sample_code, "MyStruct", &sample_uri)),
            },
        );

        let mut expected_functions = HashMap::new();
        expected_functions.insert(
            "main".to_owned(),
            LangFunc {
                params: vec![param_var.clone()],
                return_type: "void".to_owned(),
                declaration_position: Some(location_of(sample_code, "main", &sample_uri)),
                references: vec![],
                desc: "".to_owned(),
            },
        );

        let main_func_start = 11;
        let main_func_end = 16;
        let expected_global_scope = Scope {
            vars: HashMap::from([(
                "global_var".to_string(),
                LangVar {
                    primary_type: "double".to_string(),
                    type_qualifier_list: vec![],
                    declaration_position: Some(location_of(sample_code, "global_var", &sample_uri)),
                    unused: true,
                },
            )]),
            scopes: vec![(
                main_func_start,
                main_func_end,
                Scope {
                    vars: HashMap::from([
                        (
                            "init_var".to_string(),
                            LangVar {
                                primary_type: "float".to_string(),
                                type_qualifier_list: vec![],
                                declaration_position: Some(location_of(
                                    sample_code,
                                    "init_var",
                                    &sample_uri,
                                )),
                                unused: true,
                            },
                        ),
                        (
                            "prim_var".to_string(),
                            LangVar {
                                primary_type: "vec4".to_string(),
                                type_qualifier_list: vec![],
                                declaration_position: Some(location_of(
                                    sample_code,
                                    "prim_var",
                                    &sample_uri,
                                )),
                                unused: true,
                            },
                        ),
                        (
                            "cust_var".to_string(),
                            LangVar {
                                primary_type: "MyStruct".to_string(),
                                type_qualifier_list: vec![],
                                declaration_position: Some(location_of(
                                    sample_code,
                                    "cust_var",
                                    &sample_uri,
                                )),
                                unused: true,
                            },
                        ),
                        (
                            "array_var".to_string(),
                            LangVar {
                                primary_type: "float".to_string(),
                                type_qualifier_list: vec!["[]".to_string(), "[]".to_string()],
                                declaration_position: Some(location_of(
                                    sample_code,
                                    "array_var",
                                    &sample_uri,
                                )),
                                unused: true,
                            },
                        ),
                        param_var,
                    ]),
                    scopes: vec![],
                },
            )],
        };

        let mut expected_defines = HashMap::new();
        expected_defines.insert(
            "myRep".to_owned(),
            LangDefine {
                insert_text: "10".to_owned(),
                declaration_position: Some(location_of(sample_code, "myRep", &sample_uri)),
            },
        );

        assert_eq!(result.defines, expected_defines);
        assert_eq!(result.functions, expected_functions);
        assert_eq!(result.global_scope, expected_global_scope);
        assert_eq!(result.keywords, vec![]);
        assert_eq!(result.types, expected_types);
    }

    #[test]
    fn validate_lang_json() {
        let json_path = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("lang_db.json");

        let file = File::open(json_path).unwrap();

        let reader = BufReader::new(file);

        // Parse the JSON
        let _: LangDB = serde_json::from_reader(reader).unwrap();
    }

    #[test]
    fn validate_semantic_tokens() {
        let (empty_lang_db, sample_code, sample_uri) = shared_sample_code();
        let result = parser::parse(sample_code.to_owned(), &sample_uri, &empty_lang_db);

        let correct_types = HashMap::from([
            (
                "myRep",
                prov_semantic_tokens::LangSemanticToken::MACRO as u32,
            ),
            (
                "MyStruct",
                prov_semantic_tokens::LangSemanticToken::STRUCT as u32,
            ),
            (
                "main",
                prov_semantic_tokens::LangSemanticToken::FUNCTION as u32,
            ),
            (
                "param_var",
                prov_semantic_tokens::LangSemanticToken::PARAMETER as u32,
            ),
        ]);

        let sm = prov_semantic_tokens::get_sm_tokens(&result);
        let mut found_strs = vec![];
        let mut row = 0;
        let mut col = 0;

        for token in sm {
            row += token.delta_line;
            if token.delta_line == 0 {
                col += token.delta_start;
            } else {
                col = token.delta_start;
            }
            let col_end = col + token.length;

            let token_str =
                &sample_code.lines().nth(row as usize).unwrap()[col as usize..col_end as usize];

            match token_str.parse::<f64>() {
                Ok(_) => continue,
                Err(_) => {}
            };

            match correct_types.get(token_str) {
                Some(token_type) => assert!(
                    *token_type == token.token_type,
                    "{} was wrong type!",
                    token_str
                ),
                None => panic!("Unexpected sm token for {}", token_str),
            };

            found_strs.push(token_str);
        }

        for lang_str in correct_types.keys() {
            assert!(
                found_strs.contains(lang_str),
                "never provided sm token for {}",
                lang_str
            );
        }
    }

    #[test]
    fn validate_ident_seq_extract() {
        let tests = vec![
            ("myObj.", vec!["myObj"]),
            ("  spaceBefore.textAfter", vec!["spaceBefore"]),
            ("not  spaceBefore.textAfter", vec!["spaceBefore"]),
            ("first.then.next.", vec!["next", "then", "first"]),
            ("", vec![]),
            ("a ", vec![]),
            ("a. ", vec![]),
            ("first[3].next", vec!["[]", "first"]),
            (
                "complex[data[3]].next[2][3].",
                vec!["[]", "[]", "next", "[]", "complex"],
            ),
            ("].next", vec![""]), // error case, just make sure it doesn't crash
        ];

        for (test_str, expected) in tests {
            let result = lsp_util::extract_identifier_sequence(
                test_str,
                Position {
                    line: 0,
                    character: test_str.len() as u32,
                },
            );
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn validate_hovers() {
        let json_path = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("lang_db.json");

        let file = File::open(json_path).unwrap();

        let reader = BufReader::new(file);

        // Parse the JSON
        let real_lang_db: LangDB = serde_json::from_reader(reader).unwrap();
        let (_, sample_code, sample_uri) = shared_sample_code();
        let result = parser::parse(sample_code.to_owned(), &sample_uri, &real_lang_db);

        let hovers = vec![(
            Position {
                line: 14,
                character: 8 + 5,
            },
            "MyStruct",
            "### MyStruct\n---\nuser defined struct\n\nfields:\n - arrayField\n\n - myField\n\n",
        ),
        (
            Position {
                line: 11,
                character: 8 + 5,
            },
            "main",
            "### main\n---\n\n\nparams:\n - param_var\n\n",
        ),
        (
            Position {
                line: 11,
                character: 8 + 1,
            },
            "void",
            "### void\n---\nbuiltin type\n\nfor functions that do not return a value",
        ),
        (
            Position {
                line: 14,
                character: 8 + 17,
            },
            "cust_var",
            "MyStruct  cust_var",
        )];

        for (hover_pos, hover_word, hover_txt) in hovers {
            let ext_word = lsp_util::extract_word_at(sample_code, hover_pos);
            assert_eq!(ext_word, hover_word);
            match prov_hover::get_hover(&get_scoped_parse_state(&result, hover_pos), hover_pos) {
                Some(hover_result) => match hover_result.contents {
                    HoverContents::Markup(markup_content) => {
                        assert_eq!(markup_content.value, hover_txt)
                    }
                    _ => panic!(
                        "Hover returned unexpected content type on position {:#?}",
                        hover_pos
                    ),
                },
                None => panic!("Hover failed on position {:#?}", hover_pos),
            }
        }
    }

    #[test]
    fn validate_completions() {}

    #[test]
    fn validate_folding() {
        let (empty_lang_db, sample_code, sample_uri) = shared_sample_code();
        let result = parser::parse(sample_code.to_owned(), &sample_uri, &empty_lang_db);

        let fr = prov_folding::get_folding_ranges(&result.global_scope);
        assert_eq!(
            fr,
            vec![FoldingRange {
                start_line: 11,
                start_character: None,
                end_line: 16,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None
            }]
        );
    }

    #[test]
    fn cov_capabilities() {
        // call all the capability functions
        prov_semantic_tokens::capabilities();
        prov_hover::capabilities();
        prov_completions::capabilities();
        prov_folding::capabilities();
    }
}
