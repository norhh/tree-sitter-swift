# Optimized Swift grammar for Tree-sitter

**This is a query-optimized Swift grammar**, designed to make Tree-sitter query
compilation dramatically faster while keeping existing queries compatible.
It is particularly useful for Swift/iOS code transformations with
[Piranha](https://github.com/uber/piranha).

This fork preserves the history and fixes of
[`satyam1749/tree-sitter-swift`](https://github.com/satyam1749/tree-sitter-swift),
based on commit `08a28993599f1968bc81631a89690503e1db7704` (version 0.3.6).
The original grammar is by
[Alex Pinkus and contributors](https://github.com/alex-pinkus/tree-sitter-swift).
This is an independently maintained fork, not an official upstream release.

## Performance

Piranha's Swift slowdown was primarily in **query compilation (`Query::new`)**,
not in finding matches in an already-parsed file.

Local measurements on October 20, 2026, using an Apple M3 Max, release builds,
Tree-sitter 0.20.10, and three samples per query:

| Cold query compilation | Original grammar | Optimized grammar | Speedup |
| --- | ---: | ---: | ---: |
| 105 unchanged Piranha rule/filter/scope queries, sum of medians | 143.891 s | 2.897 s | **49.7×** |
| Conjunction cleanup query | 2.313 s | 21.2 ms | 109× |
| Literal equality cleanup query | 23.242 s | 15.3 ms | 1,521× |
| Function scope query | 13.524 s | 593.6 ms | 22.8× |

These are workload- and machine-specific results, not latency guarantees.
Capture names, node kinds, ranges, and ordering agreed across both grammars on
the benchmark fixture. No shipped Piranha queries were changed.

## What is optimized?

The old grammar contained an entirely hidden recursive path:

```text
_expression → _primary_expression → _playground_literal
    → argument list → _expression → …
```

Tree-sitter flattens hidden nodes. Consequently, query analysis could enumerate
millions of states through nested playground-literal arguments, even for queries
that never mentioned a playground literal.

This fork makes **`playground_literal` a visible, named node**. That boundary
stops the expensive sibling expansion. The LR parser still has **6,567 states**,
the same as before; this fix does not require a parser-state reduction, query
serialization, or changes to the Tree-sitter runtime.

The scanner, grammar precedence, conflicts, and optional-chain rules are
unchanged.

## Query compatibility

All existing public node kinds and field names are retained. Ordinary
expressions, optional chains, calls, types, function parameters, and literals
keep their existing tree shapes.

The only intentional tree-shape change is for:

- `#imageLiteral(...)`
- `#fileLiteral(...)`
- `#colorLiteral(...)`

These now produce a single `playground_literal` expression rather than
flattening their argument labels and values into the enclosing node:

```scheme
; Before
(property_declaration
  (simple_identifier)
  (line_string_literal))

; Optimized grammar
(property_declaration
  value: (playground_literal
    (simple_identifier)
    (line_string_literal)))
```

Queries that specifically rely on flattened playground arguments should descend
into `playground_literal`. Queries that find an argument anywhere in a subtree
still work. The new boundary also prevents literal-expression queries from
mistaking playground arguments for sibling operands.

## Rust usage

Use a pinned commit from this repository for reproducible builds:

```toml
[dependencies]
tree-sitter = "0.20.10"
tree-sitter-swift = { git = "https://github.com/norhh/tree-sitter-swift", branch = "optimized-query-performance" }
```

Replace `branch` with `rev = "<commit SHA>"` when integrating into an application.

```rust
let mut parser = tree_sitter::Parser::new();
parser.set_language(tree_sitter_swift::language()).unwrap();
let tree = parser.parse("let enabled = true", None).unwrap();
assert!(!tree.root_node().has_error());
```

This fork retains the Tree-sitter 0.20 Rust API and ABI-14 grammar.

## Tests and a small compilation benchmark

```sh
cargo test
cargo run --release --example query_compile
```

The example measures three representative queries independently of parsing and
matching. It is a small reproduction, not the full 105-query Piranha workload.

The original **178-case grammar corpus** is retained. Only the expected
playground-literal tree is updated. The corpus also runs through `cargo test`,
without requiring the Tree-sitter CLI. Additional tests cover nested playground
literals and literal-query behavior.

The grammar was also validated against all 11 Piranha Swift rewrite tests, which
completed in about 33 seconds in a debug build, including eight formerly ignored
tests.

## Regenerating the parser

Generated sources are checked in, so normal Rust builds need a C compiler but
do not need Node or the Tree-sitter CLI.

To reproduce this fork's generated sources:

```sh
cargo install tree-sitter-cli --version 0.20.8 --locked
tree-sitter generate --no-bindings
tree-sitter test
```

The existing Node bindings and upstream tooling are retained. Their package
metadata still uses the original package name; this repository does not claim
to publish or replace the upstream npm package.

## License

MIT. See [LICENSE](LICENSE). Original attribution is preserved.
