// The upstream corpus is MIT-licensed; see ../LICENSE.
// Run it with cargo test as well as `tree-sitter test`, without requiring the CLI.

use regex::Regex;
use std::{fs, path::Path};
use tree_sitter::Parser;

#[test]
fn upstream_corpus() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_swift::LANGUAGE.into())
        .unwrap();
    // Corpus cases may opt into field assertions; otherwise omit field names
    // from Node::to_sexp(), just like the Tree-sitter CLI's corpus runner.
    let fields = Regex::new(r"\b[a-z_]+:\s+").unwrap();
    let headers = Regex::new(r"(?m)^={3,}\r?$").unwrap();
    let separators = Regex::new(r"(?m)^-{3,}\r?$").unwrap();
    let compact = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let mut cases = 0;
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("test/corpus");
    for file in fs::read_dir(corpus).unwrap() {
        let file = file.unwrap().path();
        let text = fs::read_to_string(&file).unwrap();
        let mut sections = headers.split(&text).skip(1);
        while let Some(name) = sections.next() {
            let body = sections.next().expect("missing corpus test body");
            let separator = separators.find(body).expect("missing expected tree");
            let source = &body[..separator.start()];
            let expected = &body[separator.end()..];
            let tree = parser.parse(source.trim_start_matches('\n'), None).unwrap();
            let actual = tree.root_node().to_sexp();
            let actual = if fields.is_match(expected) {
                actual.as_str().into()
            } else {
                fields.replace_all(&actual, "")
            };
            assert_eq!(
                compact(&actual),
                compact(expected),
                "{}: {}",
                file.display(),
                name.trim()
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 299, "all upstream corpus cases must run");
}
