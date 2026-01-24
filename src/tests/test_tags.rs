#![cfg(test)]
#![allow(clippy::pedantic)]

// Tag tests

mk_test!(
    tag with quoted scalar;
    r#"!str "hello world""# => map!{ "__type" : "str", "__value" : "hello world" }
);

mk_test!(
    tag with flow sequence;
    r"!list [a, b, c]" => map!{ "__type" => "list"; "__value" => seq!("a", "b", "c") }
);

mk_test!(
    tag with flow mapping;
    r"!person {name: John, age: 30}" => map!{ "__type" => "person"; "name" => "John"; "age" => crate::Yaml::Int(30) }
);

mk_test!(
    tag in flow mapping value;
    r"{key: !tagged value}" => map!{ "key" => map!{ "__type" : "tagged", "__value" : "value" } }
);

mk_test!(
    tag with hyphen in name;
    r"!my-custom-tag value" => map!{ "__type" : "my-custom-tag", "__value" : "value" }
);

mk_test!(
    tag with underscore in name;
    r"!my_tag value" => map!{ "__type" : "my_tag", "__value" : "value" }
);

mk_test!(
    empty tag name;
    r"! value" => fail
);

// Tag tests - all tags create __type mappings

#[test]
fn test_int_tag_creates_type_mapping() {
    let parsed = crate::parse("!int 42").unwrap();
    if let crate::Yaml::Mapping(entries) = parsed {
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, crate::Yaml::Scalar("__type"));
        assert_eq!(entries[0].value, crate::Yaml::Scalar("int"));
        assert_eq!(entries[1].key, crate::Yaml::Scalar("__value"));
        assert_eq!(entries[1].value, crate::Yaml::Int(42));
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_int_tag_with_non_numeric() {
    // Now accepts any value - no validation
    let parsed = crate::parse("!int abc").unwrap();
    if let crate::Yaml::Mapping(entries) = parsed {
        assert_eq!(entries[0].value, crate::Yaml::Scalar("int"));
        assert_eq!(entries[1].value, crate::Yaml::Scalar("abc"));
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_float_tag_creates_type_mapping() {
    let parsed = crate::parse("!float 3.14").unwrap();
    if let crate::Yaml::Mapping(entries) = parsed {
        assert_eq!(entries[0].value, crate::Yaml::Scalar("float"));
        assert_eq!(entries[1].value, crate::Yaml::Float(3.14));
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_bool_tag_creates_type_mapping() {
    let parsed = crate::parse("!bool true").unwrap();
    if let crate::Yaml::Mapping(entries) = parsed {
        assert_eq!(entries[0].value, crate::Yaml::Scalar("bool"));
        assert_eq!(entries[1].value, crate::Yaml::Bool(true));
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_typed_values_in_mapping() {
    // Test automatic type inference (no explicit tags needed)
    let yaml = r#"
count: 42
price: 19.99
enabled: true
"#;
    let parsed = crate::parse(yaml).unwrap();
    if let crate::Yaml::Mapping(entries) = parsed {
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value, crate::Yaml::Int(42));
        assert_eq!(entries[1].value, crate::Yaml::Float(19.99));
        assert_eq!(entries[2].value, crate::Yaml::Bool(true));
    } else {
        panic!("Expected mapping");
    }
}

// Automatic type inference tests

#[test]
fn test_auto_int_inference() {
    assert_eq!(crate::parse("42").unwrap(), crate::Yaml::Int(42));
    assert_eq!(crate::parse("-123").unwrap(), crate::Yaml::Int(-123));
    assert_eq!(crate::parse("0").unwrap(), crate::Yaml::Int(0));
}

#[test]
fn test_auto_float_inference() {
    assert_eq!(crate::parse("3.14").unwrap(), crate::Yaml::Float(3.14));
    assert_eq!(crate::parse("-2.5").unwrap(), crate::Yaml::Float(-2.5));
    assert_eq!(crate::parse("1.0e10").unwrap(), crate::Yaml::Float(1.0e10));
}

#[test]
fn test_auto_bool_inference() {
    assert_eq!(crate::parse("true").unwrap(), crate::Yaml::Bool(true));
    assert_eq!(crate::parse("false").unwrap(), crate::Yaml::Bool(false));
    assert_eq!(crate::parse("yes").unwrap(), crate::Yaml::Bool(true));
    assert_eq!(crate::parse("no").unwrap(), crate::Yaml::Bool(false));
    assert_eq!(crate::parse("on").unwrap(), crate::Yaml::Bool(true));
    assert_eq!(crate::parse("off").unwrap(), crate::Yaml::Bool(false));
}

#[test]
fn test_quoted_strings_not_converted() {
    // Quoted values should remain as strings (quotes stripped)
    assert_eq!(crate::parse(r#""42""#).unwrap(), crate::Yaml::Scalar("42"));
    assert_eq!(
        crate::parse(r#"'true'"#).unwrap(),
        crate::Yaml::Scalar("true")
    );
    assert_eq!(
        crate::parse(r#""3.14""#).unwrap(),
        crate::Yaml::Scalar("3.14")
    );
}

#[test]
fn test_non_numeric_strings_not_converted() {
    assert_eq!(crate::parse("hello").unwrap(), crate::Yaml::Scalar("hello"));
    assert_eq!(
        crate::parse("foo123").unwrap(),
        crate::Yaml::Scalar("foo123")
    );
}

// Composite/generic type tag tests

#[test]
fn test_generic_tag_seq() {
    // Test !seq<T> style generic tags
    let yaml = r#"
tags: !seq<string> [rust, yaml, parser]
"#;
    let parsed = crate::parse(yaml).unwrap();
    if let crate::Yaml::Mapping(entries) = &parsed {
        if let crate::Yaml::Mapping(inner) = &entries[0].value {
            assert_eq!(inner[0].value, crate::Yaml::Scalar("seq<string>"));
            assert_eq!(
                inner[1].value,
                crate::Yaml::Sequence(vec![
                    crate::Yaml::Scalar("rust"),
                    crate::Yaml::Scalar("yaml"),
                    crate::Yaml::Scalar("parser")
                ])
            );
        } else {
            panic!("Expected inner mapping");
        }
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_generic_tag_option() {
    // Test !option<T> for nullable values
    let yaml = r#"
user:
  name: Alice
  nickname: !option<string> "Ali"
  age: !option<int> 30
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();
    let user = json.get("user").unwrap().as_object().unwrap();

    let nickname = user.get("nickname").unwrap().as_object().unwrap();
    assert_eq!(nickname.get("__type").unwrap(), "option<string>");
    assert_eq!(nickname.get("__value").unwrap(), "Ali");

    let age = user.get("age").unwrap().as_object().unwrap();
    assert_eq!(age.get("__type").unwrap(), "option<int>");
    assert_eq!(age.get("__value").unwrap(), 30);
}

#[test]
fn test_generic_tag_map() {
    // Test !map<K,V> for typed dictionaries
    let yaml = r#"
settings: !map<string,int>
  timeout: 30
  retries: 3
  max_connections: 100
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();
    let settings = json.get("settings").unwrap().as_object().unwrap();

    assert_eq!(settings.get("__type").unwrap(), "map<string,int>");
    assert_eq!(settings.get("timeout").unwrap(), 30);
    assert_eq!(settings.get("retries").unwrap(), 3);
    assert_eq!(settings.get("max_connections").unwrap(), 100);
}

#[test]
fn test_generic_tag_nested() {
    // Test nested generics like !seq<option<string>>
    let yaml = r#"!seq<option<string>> [hello, world]"#;
    let parsed = crate::parse(yaml).unwrap();
    if let crate::Yaml::Mapping(entries) = &parsed {
        assert_eq!(entries[0].value, crate::Yaml::Scalar("seq<option<string>>"));
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_generic_tag_union() {
    // Test union types with pipe: !option<string|int>
    let yaml = r#"value: !option<string|int> 42"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();
    let value = json.get("value").unwrap().as_object().unwrap();
    assert_eq!(value.get("__type").unwrap(), "option<string|int>");
    assert_eq!(value.get("__value").unwrap(), 42);
}

#[test]
fn test_generic_tag_unclosed_bracket() {
    // Unclosed angle bracket should fail
    let result = crate::parse("!seq<string [a, b]");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.msg.unwrap().contains("unclosed '<'"));
}

#[test]
fn test_generic_tag_unmatched_close() {
    // Unmatched closing bracket should fail
    let result = crate::parse("!seq> [a, b]");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.msg.unwrap().contains("unmatched '>'"));
}

#[test]
fn test_generic_tag_with_spaces() {
    // Spaces inside angle brackets - tag stops at space
    // !option<string | int> becomes tag "option<string" (unclosed)
    let yaml = r#"!option<string | int> 42"#;
    let result = crate::parse(yaml);
    println!("Result with spaces: {:?}", result);
    // This should fail because space breaks the tag, leaving unclosed '<'
    assert!(result.is_err());
}
