# mini-yaml-rs

A minimalist, zero-copy YAML parser for Rust. Supports sequences, mappings, and custom tags.

## Features

- Zero-copy parsing (returns references to input)
- Sequences and mappings (flow and block styles)
- Custom tag support: `!tagname` becomes `__type: "tagname"`

## Usage

```rust
use mini_yaml_rs::parse;

let yaml = parse(r#"
name: John
items:
  - apple
  - banana
"#).unwrap();
```

### Tag Support

Tags are converted to `__type` fields:

```yaml
!person {name: John}     # → {__type: "person", name: "John"}
!int 42                  # → {__type: "int", __value: "42"}
!list [a, b]             # → {__type: "list", __value: ["a", "b"]}
```

## License

Apache-2.0. See [LICENSE](LICENSE).

## Acknowledgments

Based on [minimal-yaml](https://github.com/nathanwhit/minimal-yaml) by Nathan Whitaker.
