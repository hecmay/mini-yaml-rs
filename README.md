# mini-yaml-rs

A minimalist, zero-copy YAML parser for Rust. Supports sequences, mappings, and custom tags.

## Features

- Zero-copy parsing (returns references to input)
- Sequences and mappings (flow and block styles)
- Custom tag support: `!tagname` becomes `__type: "tagname"`
- Works in both Rust backend (Tauri) and WebAssembly

## Installation

### Rust (Cargo)

```toml
[dependencies]
mini-yaml-rs = "0.2"
```

### JavaScript/TypeScript (npm)

```bash
npm install mini-yaml-rs
```

## Building

```bash
# Build Rust library (for Tauri/backend)
make build-rust

# Build WASM for bundlers (Vite, webpack)
make build

# Build WASM for direct browser use
make build-web

# Build WASM for Node.js
make build-node
```

## Usage

### Rust

```rust
use mini_yaml_rs::parse;

let yaml = parse(r#"
+setup[Settings](db://settings):
    title: Settings
    authors[]:
      - !mod +@[Me](shawnx){}
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
  "+setup[Settings](db://settings)": {
    "title": "Settings",
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

### Magix Format

The `to_mx()` method transforms keys in `+name[label](value)` format:

```rust
use mini_yaml_rs::parse;

let yaml = parse(r#"
+setup[Settings](db://settings):
    title: Settings
    enabled: true
"#).unwrap();

let mx = yaml.to_mx();
println!("{}", mx);
```

Output:
```json
{
  "+setup": {
    "__name": "Settings",
    "__value": "db://settings",
    "title": "Settings",
    "enabled": true
  }
}
```

The key `+setup[Settings](db://settings)` becomes `+setup` with:
- `__name` = bracket content (`Settings`)
- `__value` = paren content (`db://settings`, optional)

### Tag Support

Tags are converted to `__type` fields:

```yaml
!person {name: John}    # → {__type: "person", name: "John"}
!custom_tag [1, 2, 3]   # → {__type: "custom_tag", __value: [1, 2, 3]}
```

### Type Inference

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

### JavaScript/TypeScript (WASM)

All functions return plain JavaScript objects (not `Map` objects), making them easy to use with standard JS object syntax.

```typescript
import init, { parseYaml, parseYamlToMx, printYaml } from 'mini-yaml-rs';

// Initialize WASM module
await init();

// Parse YAML → JavaScript object (no JSON.parse needed)
const obj = parseYaml(`
name: hello
value: 42
items:
  - one
  - two
`);
console.log(obj.name);   // "hello"
console.log(obj.value);  // 42
console.log(obj.items);  // ["one", "two"]

// Convert JavaScript object → YAML string
const yaml = printYaml({
  title: "My Config",
  enabled: true,
  settings: {
    theme: "dark",
    fontSize: 14
  }
});
console.log(yaml);
// title: My Config
// enabled: true
// settings:
//   theme: dark
//   fontSize: 14

// Parse with mx transformation
const mx = parseYamlToMx(`
+setup[Settings](db://settings):
  title: Settings
`);
console.log(mx["+setup"].__name);   // "Settings"
console.log(mx["+setup"].__value);  // "db://settings"
```

## License

Apache-2.0. See [LICENSE](LICENSE).

## Acknowledgments

Based on [minimal-yaml](https://github.com/nathanwhit/minimal-yaml) by Nathan Whitaker.
