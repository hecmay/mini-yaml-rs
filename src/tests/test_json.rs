#![cfg(test)]
#![allow(clippy::pedantic)]

// JSON conversion tests

#[test]
fn test_typed_values_to_json() {
    // Test automatic type inference in JSON conversion
    let yaml = r#"
count: 42
price: 19.99
enabled: true
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();
    let obj = json.as_object().unwrap();

    assert_eq!(obj.get("count").unwrap().as_i64().unwrap(), 42);
    assert_eq!(obj.get("price").unwrap().as_f64().unwrap(), 19.99);
    assert_eq!(obj.get("enabled").unwrap().as_bool().unwrap(), true);
}

#[test]
fn test_to_json_basic() {
    let yaml = r#"
name: John
age: 30
hobbies:
  - reading
  - coding
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();

    assert!(json.is_object());
    let obj = json.as_object().unwrap();
    assert_eq!(obj.get("name").unwrap(), "John");
    assert_eq!(obj.get("age").unwrap().as_i64().unwrap(), 30);

    let hobbies = obj.get("hobbies").unwrap().as_array().unwrap();
    assert_eq!(hobbies.len(), 2);
    assert_eq!(hobbies[0], "reading");
    assert_eq!(hobbies[1], "coding");
}

#[test]
fn test_to_json_nested() {
    let yaml = r#"
outer:
  inner:
    value: nested
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();

    let json_str = serde_json::to_string(&json).unwrap();
    assert!(json_str.contains("outer"));
    assert!(json_str.contains("inner"));
    assert!(json_str.contains("nested"));
}

// to_mx tests

#[test]
fn test_to_mx_basic() {
    let yaml = r#"
+myKey[Display Name](some value):
  foo: bar
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();

    let obj = json.as_object().unwrap();
    let my_key = obj.get("+myKey").unwrap().as_object().unwrap();
    assert_eq!(my_key.get("__name").unwrap(), "Display Name");
    assert_eq!(my_key.get("__value").unwrap(), "some value");
    assert_eq!(my_key.get("foo").unwrap(), "bar");
}

#[test]
fn test_to_mx_without_paren() {
    let yaml = r#"
+settings.config[My Settings]:
  enabled: true
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();

    let obj = json.as_object().unwrap();
    let settings = obj.get("+settings.config").unwrap().as_object().unwrap();
    assert_eq!(settings.get("__name").unwrap(), "My Settings");
    assert!(settings.get("__value").is_none());
    assert_eq!(settings.get("enabled").unwrap().as_bool().unwrap(), true);
}

#[test]
fn test_to_mx_with_special_chars() {
    let yaml = r#"
+app.user@domain[User Name](user://id):
  active: true
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();

    let obj = json.as_object().unwrap();
    let app = obj.get("+app.user@domain").unwrap().as_object().unwrap();
    assert_eq!(app.get("__name").unwrap(), "User Name");
    assert_eq!(app.get("__value").unwrap(), "user://id");
}

#[test]
fn test_to_mx_error_not_object() {
    let yaml = "- item1\n- item2";
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();

    let obj = json.as_object().unwrap();
    let error = obj.get("+error").unwrap().as_object().unwrap();
    assert!(error
        .get("__name")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("must be an object"));
}

#[test]
fn test_to_mx_error_invalid_key() {
    let yaml = r#"
invalid_key:
  foo: bar
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();

    let obj = json.as_object().unwrap();
    let error = obj.get("+error").unwrap().as_object().unwrap();
    assert!(error
        .get("__name")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("does not match"));
}

#[test]
fn test_to_mx_empty_mx_value() {
    // Mx key with empty parentheses, no colon - parsed as scalar
    // to_mx should recognize the pattern and convert to object instance
    let yaml = r#"+shop[Your Online Shop]()"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();

    let obj = json.as_object().unwrap();
    let shop = obj.get("+shop").unwrap().as_object().unwrap();
    assert_eq!(shop.get("__name").unwrap(), "Your Online Shop");
    assert_eq!(shop.get("__value").unwrap(), "");
}

#[test]
fn test_to_mx_colon_inside_brackets() {
    // Test that to_mx correctly extracts __name with colons
    let yaml = r#"
+test.banner[Title: Subtitle](http://example.com):
  foo: bar
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();

    let obj = json.as_object().unwrap();
    let banner = obj.get("+test.banner").unwrap().as_object().unwrap();
    assert_eq!(banner.get("__name").unwrap(), "Title: Subtitle");
    assert_eq!(banner.get("__value").unwrap(), "http://example.com");
    assert_eq!(banner.get("foo").unwrap(), "bar");
}

// Field order preservation tests

#[test]
fn test_field_order_preserved_in_json() {
    // Test that field order is preserved when converting to JSON
    // This requires serde_json's preserve_order feature
    let yaml = r#"
zebra: 1
alpha: 2
mike: 3
beta: 4
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();
    let obj = json.as_object().unwrap();

    // Check that keys are in original YAML order, not sorted
    let keys: Vec<&String> = obj.keys().collect();
    assert_eq!(keys, vec!["zebra", "alpha", "mike", "beta"]);
}

#[test]
fn test_field_order_preserved_nested() {
    // Test field order preservation in nested structures
    let yaml = r#"
outer:
  zz: first
  aa: second
  mm: third
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();
    let outer = json
        .as_object()
        .unwrap()
        .get("outer")
        .unwrap()
        .as_object()
        .unwrap();

    let keys: Vec<&String> = outer.keys().collect();
    assert_eq!(keys, vec!["zz", "aa", "mm"]);
}

#[test]
fn test_field_order_preserved_in_mx() {
    // Test that field order is preserved in to_mx() conversion
    let yaml = r#"
+form[Test Form]:
  zField: value1
  aField: value2
  mField: value3
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_mx();
    let form = json
        .as_object()
        .unwrap()
        .get("+form")
        .unwrap()
        .as_object()
        .unwrap();

    // __name is inserted but other fields should maintain order
    let keys: Vec<&String> = form.keys().collect();
    // Note: __name is inserted during to_mx transformation
    assert!(keys.contains(&&"zField".to_string()));
    assert!(keys.contains(&&"aField".to_string()));
    assert!(keys.contains(&&"mField".to_string()));

    // Find positions of the original fields (excluding __name)
    let field_keys: Vec<&String> = keys
        .iter()
        .filter(|k| !k.starts_with("__"))
        .copied()
        .collect();
    assert_eq!(field_keys, vec!["zField", "aField", "mField"]);
}
