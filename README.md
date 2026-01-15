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
+setup[Magix RTE Settings](magix://prelude/settings):
    title: Magix RTE Settings
    authors[]:
      - !mod +@[Me](shawnx)
      - !mod +@[Boby at Unix Group](boby:unix.org)

    signature:
        pubkey: |
            MCowBQYDK2VwAyEAGb9F2CMlxqLDB3rrzBVwC7aB...

    created: 2024-06-01T12:00:00Z
    modified: 2024-06-01T12:00:00Z
"#).unwrap();

// Returns Yaml<'_> enum - use to_json() for serde_json::Value
let json = yaml.to_json();
println!("{}", json);
```

Output:
```json
{
  "+setup[Magix RTE Settings](magix://prelude/settings)": {
    "title": "Magix RTE Settings",
    "authors": [
      {
        "__type": "mod",
        "+@[Me](shawnx)"
      },
      {
        "__type": "mod",
        "+@[Boby at Unix Group](boby:unix.org)"
      }
    ],
    "signature": {
      "pubkey": "MCowBQYDK2VwAyEAGb9F2CMlxqLDB3rrzBVwC7aB..."
    },
    "created": "2024-06-01T12:00:00Z",
    "modified": "2024-06-01T12:00:00Z"
  }
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
