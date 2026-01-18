#![cfg(all(test, feature = "wasm"))]

use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse_yaml_returns_plain_object() {
    let yaml = r#"
name: test
value: 123
"#;
    let result = crate::wasm::parse_yaml_to_json(yaml).unwrap();

    // Verify it's a plain Object, not a Map
    assert!(result.is_object());
    assert!(!result.has_type::<js_sys::Map>());

    // Verify we can access it as a plain JS object
    let obj = result.dyn_ref::<js_sys::Object>().unwrap();
    let keys = js_sys::Object::keys(obj);
    assert_eq!(keys.length(), 2);
}

#[wasm_bindgen_test]
fn test_parse_yaml_to_mx_returns_plain_object() {
    let yaml = r#"
+database[Order History](db://localhost):
  header:
    - name: id
    - name: date
"#;
    let result = crate::wasm::parse_yaml_to_mx(yaml).unwrap();

    // Verify it's a plain Object, not a Map
    assert!(result.is_object());
    assert!(!result.has_type::<js_sys::Map>());

    // Verify we can access it as a plain JS object
    let obj = result.dyn_ref::<js_sys::Object>().unwrap();
    let keys = js_sys::Object::keys(obj);
    assert!(keys.length() > 0);
}

#[wasm_bindgen_test]
fn test_nested_objects_are_plain() {
    let yaml = r#"
outer:
  inner:
    key: value
"#;
    let result = crate::wasm::parse_yaml_to_json(yaml).unwrap();

    // Get nested object and verify it's also a plain object
    let obj = result.dyn_ref::<js_sys::Object>().unwrap();
    let outer = js_sys::Reflect::get(obj, &"outer".into()).unwrap();
    assert!(outer.is_object());
    assert!(!outer.has_type::<js_sys::Map>());

    let inner = js_sys::Reflect::get(&outer, &"inner".into()).unwrap();
    assert!(inner.is_object());
    assert!(!inner.has_type::<js_sys::Map>());
}

#[wasm_bindgen_test]
fn test_parse_yaml_preserves_field_order() {
    // Test that field order is preserved when returning JS object
    let yaml = r#"
zebra: 1
alpha: 2
mike: 3
beta: 4
"#;
    let result = crate::wasm::parse_yaml_to_json(yaml).unwrap();
    let obj = result.dyn_ref::<js_sys::Object>().unwrap();
    let keys = js_sys::Object::keys(obj);

    // Verify keys are in original YAML order, not alphabetically sorted
    assert_eq!(keys.length(), 4);
    assert_eq!(keys.get(0).as_string().unwrap(), "zebra");
    assert_eq!(keys.get(1).as_string().unwrap(), "alpha");
    assert_eq!(keys.get(2).as_string().unwrap(), "mike");
    assert_eq!(keys.get(3).as_string().unwrap(), "beta");
}

#[wasm_bindgen_test]
fn test_parse_yaml_to_mx_preserves_field_order() {
    // Test that field order is preserved in to_mx WASM binding
    let yaml = r#"
+form[Test]:
  zzz: first
  aaa: second
  mmm: third
"#;
    let result = crate::wasm::parse_yaml_to_mx(yaml).unwrap();
    let obj = result.dyn_ref::<js_sys::Object>().unwrap();

    // Get the +form object
    let form = js_sys::Reflect::get(obj, &"+form".into()).unwrap();
    let form_obj = form.dyn_ref::<js_sys::Object>().unwrap();
    let keys = js_sys::Object::keys(form_obj);

    // Collect keys into a Vec for easier assertion
    let key_vec: Vec<String> = (0..keys.length())
        .map(|i| keys.get(i).as_string().unwrap())
        .filter(|k| !k.starts_with("__")) // exclude metadata fields
        .collect();

    // Verify original YAML field order is preserved
    assert_eq!(key_vec, vec!["zzz", "aaa", "mmm"]);
}

#[wasm_bindgen_test]
fn test_parse_yaml_utf8_chinese_preserved() {
    // Test that Chinese characters are correctly preserved through WASM binding
    let yaml = r#"info: |
  你好世界
  测试中文
"#;
    let result = crate::wasm::parse_yaml_to_json(yaml).unwrap();
    let obj = result.dyn_ref::<js_sys::Object>().unwrap();

    let info = js_sys::Reflect::get(obj, &"info".into()).unwrap();
    let info_str = info.as_string().unwrap();

    assert!(info_str.contains("你好世界"), "Chinese not preserved: {}", info_str);
    assert!(info_str.contains("测试中文"), "Chinese not preserved: {}", info_str);
}
