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

// Returns Yaml<'_> enum - use to_json() for serde_json::Value
let json = yaml.to_json();
println!("{}", json);
```

Output:
```json
{
  "name": "John",
  "items": ["apple", "banana"]
}
```

### Tag Support

Tags are converted to `__type` fields:

```yaml
!person {name: John}    # → {__type: "person", name: "John"}
!custom_tag [1, 2, 3]   # → {__type: "custom_tag", __value: [1, 2, 3]}
```

### Native Type Conversion

Unquoted scalar values are automatically converted to native types:

```yaml
42            # → 42 (as integer)
-123          # → -123 (as integer)
3.14          # → 3.14 (as float)
1.0e10        # → 1.0e10 (as float)
true          # → true (as boolean)
false         # → false (as boolean)
yes/no        # → true/false (as boolean)
on/off        # → true/false (as boolean)

# Keep these as strings by quoting:
"42"          # → "42" (string, quotes stripped)
'true'        # → "true" (string, quotes stripped)
```


## License

Apache-2.0. See [LICENSE](LICENSE).

## Acknowledgments

Based on [minimal-yaml](https://github.com/nathanwhit/minimal-yaml) by Nathan Whitaker.
