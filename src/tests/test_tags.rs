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
