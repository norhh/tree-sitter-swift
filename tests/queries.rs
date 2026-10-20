use tree_sitter::{Parser, Query, QueryCursor};

#[test]
fn playground_literals_are_single_expression_values() {
    let language = tree_sitter_swift::language();
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    for literal in [
        r#"#imageLiteral(resourceName: "heart")"#,
        r#"#fileLiteral(resourceName: "data")"#,
        "#colorLiteral(red: 1, green: 0, blue: 0, alpha: 1)",
        "#colorLiteral(red: #colorLiteral(red: 1), green: 0)",
    ] {
        let source = format!("let value = {literal}");
        let tree = parser.parse(&source, None).unwrap();
        assert!(!tree.root_node().has_error(), "{}", source);
        let declaration = tree.root_node().named_child(0).unwrap();
        let value = declaration.child_by_field_name("value").unwrap();
        assert_eq!(value.kind(), "playground_literal");
        assert_eq!(value.utf8_text(source.as_bytes()).unwrap(), literal);
    }
}

#[test]
fn literal_queries_do_not_match_flattened_playground_arguments() {
    let source = "let a = 1 == 2\nlet b = #colorLiteral(red: 1, green: 2) == 2";
    let language = tree_sitter_swift::language();
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(!tree.root_node().has_error());
    let query = Query::new(
        language,
        "(equality_expression lhs: (integer_literal) @left rhs: (integer_literal) @right) @expr",
    )
    .unwrap();
    let mut cursor = QueryCursor::new();
    let matched_expressions = cursor
        .matches(&query, tree.root_node(), source.as_bytes())
        .map(|m| {
            let expr = m
                .captures
                .iter()
                .find(|c| query.capture_names()[c.index as usize] == "expr")
                .unwrap();
            expr.node.utf8_text(source.as_bytes()).unwrap().to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(matched_expressions, vec!["1 == 2"]);
    assert!(!cursor.did_exceed_match_limit());
}
