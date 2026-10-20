# Optimised Swift grammar for Tree-sitter

This fork provides a Swift grammar with fast Tree-sitter query compilation,
regression tests, and a reproducible compilation benchmark. It is useful for
Swift/iOS code transformations, including
[Piranha](https://github.com/uber/piranha).

## Choose the right branch

| Branch | Grammar base | Rust API |
| --- | --- | --- |
| `main` | 0.7.4, retaining current upstream Swift support | Tree-sitter 0.23, `LANGUAGE` |
| `optimized-query-performance` | 0.3.6, validated with Piranha's existing queries | Tree-sitter 0.20, `language()` |

The legacy Piranha branch is pinned at
`6d6ebdba4d1a90690e4d2c2e513a4aa60a1b90c9`. Merging it into this repository's
`main` does **not** change that commit or Piranha's dependency pin. `main` retains
its newer upstream grammar, bindings, and toolchain; it is not a drop-in Rust API
replacement for the Tree-sitter 0.20 integration.

The original grammar is by
[Alex Pinkus and contributors](https://github.com/alex-pinkus/tree-sitter-swift).
The Piranha-compatible branch also preserves the history and fixes from
[`satyam1749/tree-sitter-swift`](https://github.com/satyam1749/tree-sitter-swift).
This is an independently maintained fork, not an official upstream release.

## What is optimised?

Earlier grammar versions contained an entirely hidden recursive path:

```text
_expression → _primary_expression → _playground_literal
    → argument list → _expression → …
```

Tree-sitter flattens hidden nodes. Query analysis could therefore enumerate
millions of states through nested playground-literal arguments, even for queries
that never mentioned a playground literal.

A visible, named **`playground_literal`** stops that expensive sibling expansion.
The Piranha-compatible branch adds this boundary to its older grammar. Modern
upstream already has the boundary; `main` preserves it and adds regression tests
so it is not accidentally hidden again. The merge does not downgrade modern
Swift support to the older Piranha grammar.

## Performance of the Piranha-compatible branch

Piranha's earlier Swift slowdown was primarily in **query compilation
(`Query::new`)**, not matching an already-parsed file.

These measurements compare the original pinned **0.3.6 grammar** with the
optimised Piranha-compatible branch, not `main` against upstream 0.7.4. They were
recorded on October 20, 2026, using an Apple M3 Max, release builds, Tree-sitter
0.20.10, and three samples per query:

| Cold query compilation | Original 0.3.6 grammar | Optimised branch | Speedup |
| --- | ---: | ---: | ---: |
| 105 unchanged Piranha rule/filter/scope queries, sum of medians | 143.891 s | 2.897 s | **49.7×** |
| Conjunction cleanup query | 2.313 s | 21.2 ms | 109× |
| Literal equality cleanup query | 23.242 s | 15.3 ms | 1,521× |
| Function scope query | 13.524 s | 593.6 ms | 22.8× |

These are workload- and machine-specific results, not latency guarantees.
Capture names, node kinds, ranges, and ordering agreed across both legacy
grammars on the benchmark fixture. No shipped Piranha queries were changed.

## Playground-literal compatibility

The intentional change in the legacy optimisation is limited to:

- `#imageLiteral(...)`
- `#fileLiteral(...)`
- `#colorLiteral(...)`

These produce a single `playground_literal` expression rather than flattening
argument labels and values into the enclosing node. This is also the shape
retained on modern `main`:

```scheme
(property_declaration
  value: (playground_literal
    (simple_identifier)
    (line_string_literal)))
```

Queries relying on the older flattened arguments should descend into
`playground_literal`. Queries that find an argument anywhere in a subtree still
work. The boundary also prevents literal-expression queries from mistaking
playground arguments for sibling operands. Other modern-upstream grammar changes
are independent of this optimisation; consult the upstream grammar when migrating
from 0.3.6 to 0.7.4.

## Rust usage: modern `main`

```toml
[dependencies]
tree-sitter = "0.23"
tree-sitter-swift = { git = "https://github.com/norhh/tree-sitter-swift", branch = "main" }
```

Replace `branch` with `rev = "<commit SHA>"` for a reproducible application build.

```rust
let mut parser = tree_sitter::Parser::new();
parser.set_language(&tree_sitter_swift::LANGUAGE.into()).unwrap();
let tree = parser.parse("let enabled = true", None).unwrap();
assert!(!tree.root_node().has_error());
```

For the existing Piranha integration, retain the legacy API and pinned version:

```toml
[dependencies]
tree-sitter = "0.20.10"
tree-sitter-swift = { git = "https://github.com/norhh/tree-sitter-swift", rev = "6d6ebdba4d1a90690e4d2c2e513a4aa60a1b90c9" }
```

## Tests and a small compilation benchmark

```sh
cargo test
cargo run --release --example query_compile
```

Modern `main` retains all **299 upstream grammar corpus cases**, including
Swift macros and the recent contextual-identifier regressions. The corpus also
runs through `cargo test`, without requiring the CLI. Additional tests cover
nested playground literals and literal-query behavior.

The benchmark measures three representative queries independently of parsing
and matching. It is a small reproduction, not the full 105-query Piranha workload.
It uses the API appropriate to its branch.

The legacy branch retains its original 178-case corpus. It was also validated
with all 11 Piranha Swift rewrite tests, including eight formerly ignored tests.

## Regenerating modern parser sources

Generated sources are checked in in this fork, so normal Rust Git-dependency
builds need a C compiler but not Node or the Tree-sitter CLI. Upstream's policy
on generated source commits is different; do not treat this fork as an upstream
release artifact.

To reproduce `main`'s generated sources:

```sh
cargo install tree-sitter-cli --version 0.23.0 --locked
tree-sitter generate --no-bindings --abi 14
tree-sitter test
```

CLI 0.23.0 matches the existing npm lockfile. The legacy branch uses CLI 0.20.8
instead. Existing modern Node, Python, Go,
Swift, and C bindings and upstream tooling are retained on `main`. The package
metadata still uses the original package name; this repository does not claim
to publish or replace the upstream npm or crates.io package.

## License

MIT. See [LICENSE](LICENSE). Original attribution is preserved.
