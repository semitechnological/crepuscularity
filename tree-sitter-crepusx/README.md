# tree-sitter-crepusx

A [tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar for the CrepusX (.csx) JSX/React Native template language.

CrepusX is the JSX-based syntax used by the Crepuscularity project for mobile templates. This grammar provides incremental parsing, syntax highlighting queries, and Rust bindings.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
tree-sitter-crepusx = "0.0.1"
```

```rust
use tree_sitter::Parser;
use tree_sitter_crepusx;

let mut parser = Parser::new();
parser.set_language(&tree_sitter_crepusx::LANGUAGE.into()).unwrap();
let tree = parser.parse("<View class=\"flex-1\">\n  <Text>Hello</Text>\n</View>", None).unwrap();
```

## License

ISC
