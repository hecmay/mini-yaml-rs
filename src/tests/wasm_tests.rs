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
