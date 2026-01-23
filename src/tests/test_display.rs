#![cfg(test)]
#![allow(clippy::pedantic)]

// Round trip tests

#[test]
fn test_round_trip_basic_literal_eq() {
    let input = r#"foo: bar
baz:
  - qux
  - quux
  - corge: grault
    garply: waldo
      - fred
      - plugh
      - xyzzy
    : thud
"#;
    assert_eq!(
        crate::parse(input).unwrap().to_string(),
        String::from(input)
    )
}

#[test]
fn test_round_trip_basic_structural_eq() {
    let input = r#"
key:
  the: value
  is:
    nested: mappings
    wow:
      - with a block seq
      - too
and: done
"#;
    assert_eq!(
        crate::parse(&crate::parse(input).unwrap().to_string()).unwrap(),
        map! {
            "key" => map! {
                "the" => "value";
                "is" => map! {
                    "nested" => "mappings";
                    "wow" => seq!("with a block seq", "too")
                }
            };
            "and" => "done"
        }
    )
}

// print_yaml / Display tests

#[test]
fn test_print_yaml_config_file() {
    let yaml = r#"server:
  host: localhost
  port: 8080
database:
  connection:
    host: db.example.com
    port: 5432
  credentials:
    username: admin
    password: secret
"#;
    let parsed = crate::parse(yaml).unwrap();
    let printed = parsed.to_string();
    assert_eq!(printed, yaml);
}

// Regression test: parser must handle sequence items after nested sequences in mappings
#[test]
fn test_parse_sequence_after_nested_sequence_in_mapping() {
    let yaml = r#"
tasks:
  - name: Build project
    deps:
      - compile
      - lint
  - name: Run tests
"#;
    let parsed = crate::parse(yaml).unwrap();
    // Expected: tasks is a sequence with TWO mapping items
    let expected = map! {
        "tasks" => seq!(
            map! {
                "name" => "Build project";
                "deps" => seq!("compile", "lint")
            },
            map! {
                "name" => "Run tests"
            }
        )
    };
    assert_eq!(parsed, expected);
}

#[test]
fn test_print_yaml_task_list() {
    let yaml = r#"tasks:
  - name: Build project
    command: cargo build
    deps:
      - compile
      - lint
  - name: Run tests
    command: cargo test
"#;
    let parsed = crate::parse(yaml).unwrap();
    let printed = parsed.to_string();
    assert_eq!(printed, yaml);
}

// __type printing as !tag tests

#[test]
fn test_print_type_as_tag_with_value() {
    // data: {__type: "str", __value: "hello"} should print as "data: !str hello"
    let yaml = map! { "data" => map! { "__type": "str", "__value": "hello" } };
    assert_eq!(yaml.to_string(), "data: !str hello\n");
}

#[test]
fn test_print_type_as_tag_with_fields() {
    // user: {__type: "person", name: "John"} should print as "user: !person\n  name: John"
    let yaml = map! { "user" => map! { "__type" => "person"; "name" => "John" } };
    assert_eq!(yaml.to_string(), "user: !person\n  name: John\n");
}

#[test]
fn test_print_yaml_with_tags_round_trip() {
    let yaml = r#"config:
  database: !connection
    host: localhost
    port: !int { __name: "port", __value: 5432 }
  cache: !redis enabled
"#;
    let expected = r#"config:
  database: !connection
    host: localhost
    port: !int
      __name: port
      __value: 5432
  cache: !redis enabled
"#;
    let parsed = crate::parse(yaml).unwrap();
    let printed = parsed.to_string();
    assert_eq!(printed, expected);
}

#[test]
fn test_tag_with_value_round_trip() {
    // Simple tag with __value should round-trip as "key: !tag value"
    let yaml = "name: !string John\n";
    let parsed = crate::parse(yaml).unwrap();
    let printed = parsed.to_string();
    assert_eq!(printed, yaml);
}

#[test]
fn test_deeply_nested_tags_round_trip() {
    // Three levels of nested tags
    let yaml = r#"root: !outer
  level1: !middle
    level2: !inner
      value: deep
"#;
    let parsed = crate::parse(yaml).unwrap();
    let printed = parsed.to_string();
    assert_eq!(printed, yaml);
}

#[test]
fn test_multiple_sibling_tags_round_trip() {
    // Multiple tagged values at the same level
    let yaml = r#"data:
  name: !string Alice
  age: !custom {}
  active: !bool true
"#;
    let parsed = crate::parse(yaml).unwrap();
    let printed = parsed.to_string();
    assert_eq!(printed, yaml);
}
