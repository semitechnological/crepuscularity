# tree-sitter-crepus

A [tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar for the Crepus template language.

Crepus is an indentation-based template DSL used by the Crepuscularity project. This grammar provides incremental parsing, syntax highlighting queries, and Rust bindings.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
tree-sitter-crepus = "0.0.1"
```

```rust
use tree_sitter::Parser;
use tree_sitter_crepus;

let mut parser = Parser::new();
parser.set_language(&tree_sitter_crepus::LANGUAGE.into()).unwrap();
let tree = parser.parse("div p-4\n  text-lg\n    Hello", None).unwrap();
```

## License

MPL-2.0
