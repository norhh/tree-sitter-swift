// The upstream corpus is MIT-licensed; see ../LICENSE.
// Run it with cargo test as well as `tree-sitter test`, without requiring the CLI.

use regex::Regex;
use std::{fs, path::Path};
use tree_sitter::Parser;

#[test]
fn upstream_corpus() {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_swift::language()).unwrap();
    // Tree-sitter's corpus format omits field names, unlike Node::to_sexp().
    let fields = Regex::new(r"\b[a-z_]+: ").unwrap();
    let compact = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let mut cases = 0;
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("test/corpus");
    for file in fs::read_dir(corpus).unwrap() {
        let file = file.unwrap().path();
        let text = fs::read_to_string(&file).unwrap();
        let mut sections = text
            .split(
                "================================================================================",
            )
            .skip(1);
        while let Some(name) = sections.next() {
            let body = sections.next().expect("missing corpus test body");
            let (source, expected) = body
        .split_once(
          "--------------------------------------------------------------------------------",
        )
        .expect("missing expected tree");
            let tree = parser.parse(source.trim_start_matches('\n'), None).unwrap();
            let actual = tree.root_node().to_sexp();
            assert_eq!(
                compact(&fields.replace_all(&actual, "")),
                compact(expected),
                "{}: {}",
                file.display(),
                name.trim()
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 178, "all upstream corpus cases must run");
}
