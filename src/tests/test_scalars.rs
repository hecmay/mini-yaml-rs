#![cfg(test)]
#![allow(clippy::pedantic)]

// Scalars

mk_test!(
    double quote scalar whitespace;
    r#""a scalar value with whitespace""# => "a scalar value with whitespace"
);

mk_test!(
    double quote scalar no whitespace;
    r#""a_scalarvaluewithout_whitespace""# => "a_scalarvaluewithout_whitespace"
);

mk_test!(
    single quote scalar whitespace;
    r#"'a scalar value with whitespace'"# => "a scalar value with whitespace"
);

mk_test!(
    single quote scalar no whitespace;
    r#"'ascalarvalue_without_whitespace'"# => "ascalarvalue_without_whitespace"
);

mk_test!(
    no quote scalar whitespace;
    "an unquoted scalar value with whitespace" => "an unquoted scalar value with whitespace"
);

mk_test!(
    no quote scalar no whitespace;
    "anunquoted_scalar_value_withoutwhitespace" => "anunquoted_scalar_value_withoutwhitespace"
);

// Literal block scalar tests

#[test]
fn test_simple_literal_block_scalar() {
    let yaml = r#"key: |
    line1
    line2
"#;
    let result = crate::parse(yaml).unwrap();
    if let crate::Yaml::Mapping(entries) = result {
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, crate::Yaml::Scalar("key"));
        if let crate::Yaml::String(s) = &entries[0].value {
            assert_eq!(s, "line1\nline2\n");
        } else {
            panic!("Expected String, got {:?}", entries[0].value);
        }
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_literal_block_scalar_strip() {
    let yaml = r#"key: |-
    line1
    line2
"#;
    let result = crate::parse(yaml).unwrap();
    if let crate::Yaml::Mapping(entries) = result {
        if let crate::Yaml::String(s) = &entries[0].value {
            assert_eq!(s, "line1\nline2");
        } else {
            panic!("Expected String");
        }
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_literal_block_in_complex_yaml() {
    let yaml = r#"
+setup[Magix RTE Settings](test://prelude/settings):
    title: Magix RTE Settings
    authors[]:
      - !mod +@[Me](shawnx)
      - !mod +@[Boby at Unix Group](boby:unix.org)

    signature:
        pubkey: |
            MCowBQYDK2VwAyEAGb9F2CMlxqLDB3rrzBVwC7aB...

    created: 2024-06-01T12:00:00Z
    modified: 2024-06-01T12:00:00Z
"#;
    // This should now parse successfully
    let result = crate::parse(yaml);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
