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
!person {name: John}    # → {__type: "person", name: "John"}
!custom_tag [1, 2, 3]   # → {__type: "custom_tag", __value: [1, 2, 3]}
```

For some built-in tags, the parser automatically converts to native types:

```yaml
42            # → 42 (as integer)
true          # → true (as boolean)
3.14          # → 3.14 (as float)

# Keep these as strings:
"42"          # → "42" (as string)
'true'        # → 'true' (as string)
```

## License

Apache-2.0. See [LICENSE](LICENSE).

## Acknowledgments

Based on [minimal-yaml](https://github.com/nathanwhit/minimal-yaml) by Nathan Whitaker.
