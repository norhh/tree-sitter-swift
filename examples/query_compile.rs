//! A small cold-query compilation benchmark. Run with:
//! cargo run --release --example query_compile

use std::time::{Duration, Instant};
use tree_sitter::Query;

fn main() {
    let literal_types = "\
        (integer_literal) (hex_literal) (oct_literal) (bin_literal) \
        (real_literal) (boolean_literal) (line_string_literal) (regex_literal)";
    let equality = format!(
        "((equality_expression lhs: [{literal_types}] @left \
         rhs: [{literal_types}] @right) @cond (#eq? @left @right))"
    );
    let queries = [
        (
            "conjunction",
            r#"((conjunction_expression
                lhs: [(boolean_literal) @true
                      (tuple_expression value: (boolean_literal) @true)]
                rhs: (_) @rhs) @expr
                (#eq? @true "true"))"#,
        ),
        ("literal equality", equality.as_str()),
        (
            "function scope",
            r#"((function_declaration
                name: (simple_identifier) @func_name
                (parameter)* @parameters
                body: (function_body)) @func
                (#eq? @func_name "@name")
                (#eq? @parameters "@params"))"#,
        ),
    ];
    let language = tree_sitter_swift::language();
    for (name, text) in queries {
        let mut samples = [Duration::ZERO; 3];
        for sample in &mut samples {
            let start = Instant::now();
            let query = Query::new(language, text).expect("query must compile");
            *sample = start.elapsed();
            std::hint::black_box(&query);
        }
        samples.sort();
        println!("{name}: median {:?} (3 cold compilations)", samples[1]);
    }
}
