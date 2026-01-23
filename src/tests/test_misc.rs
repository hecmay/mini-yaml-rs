#![cfg(test)]
#![allow(clippy::pedantic)]

use crate::YamlParseError;

impl<'a> From<&'a str> for crate::Yaml<'a> {
    fn from(other: &'a str) -> Self {
        crate::Yaml::Scalar(other)
    }
}

// Misc tests

mk_test!(
input with comments;
r#"
key: #comment 1
   - value line 1
   #comment 2
   - value line 2
   #comment 3
   - value line 3
"# => map!{
    "key" => seq!(
        "value line 1",
        "value line 2",
        "value line 3"
    )
}
);

mk_test!(
scalar with pound in middle;
r#"
- foo#bar
- "baz#bax"
- 'quux#xyzzy'
"# => seq!(
        "foo",
        "baz#bax",
        "quux#xyzzy"
    )
);

mk_test!(
input with error;
r#"
{key: value, missing : }
"# => err YamlParseError{ line: 2, col: 25, msg: Some(String::from(r#"unexpected symbol '}'"#)), source: None }
);

mk_test!(
error msg;
r#"
{key: value, missing : }
"# => err msg r#"error occurred parsing the input at line 2, column 25 : unexpected symbol '}'"#
);

mk_test!(
nested seq in complex mapping with empty line;
r#"
+test.settings.states[test-system-states-demo]():
  opened_apps:
    - Magix-Introduction.md
    - Ex1-Personal-Productivity.md
    - Ex2-Teaching-Aids.md

  pinned_apps:
    - name: Magix Docs
      icon: book
      command: |
        { open } = import('test');
        open("Magix-Introduction.md");
"# => map! {
    "+test.settings.states[test-system-states-demo]()" => map! {
        "opened_apps" => seq!(
            "Magix-Introduction.md",
            "Ex1-Personal-Productivity.md",
            "Ex2-Teaching-Aids.md"
        );
        "pinned_apps" => seq!(
            map! {
                "name" => "Magix Docs";
                "icon" => "book";
                "command" => crate::Yaml::String("{ open } = import('test');\nopen(\"Magix-Introduction.md\");\n".to_string())
            }
        )
    }
}
);

mk_test!(
input with doc start;
r"
---
- this
- is
- a
-
  sequence: of
  values: in
  a: yaml file
" => seq!(
    "this", "is", "a",
    map! {
        "sequence" : "of", "values" : "in", "a" : "yaml file"
    }
)
);

mk_test!(
input with seq and dash;
r"
---
- this
- is
- a
- valid
- minimal-yaml
- sequence
" => seq!("this", "is", "a", "valid", "minimal-yaml", "sequence")
);

mk_test!(
odd structure;
r"
this is:
 - totally
 - valid
 - input: to the parser
" => map!{ "this is" => seq!("totally", "valid", map!{ "input": "to the parser"})  }
);

mk_test!(
readme example;
r"
[this, is]:
 -
  - totally
  - valid
 - input
 - {to: the parser}
 " => map!{ seq!("this", "is") => seq!( seq!("totally","valid"), "input", map!{"to":"the parser"})}
);

mk_test!(
block mapping missing value;
r"
a: block
mapping: missing
a value for this key:

" => err YamlParseError { line: 5, col: 1, msg: Some("unexpected end of input".into()), source: None}
);

mk_test!(
input with indicators;
r"
stuff:
    - this::thing::with::colons::and::all-these-other-indicator-characters-:used:-in--an:unquoted:::::::string

" => map! { "stuff" => seq!("this::thing::with::colons::and::all-these-other-indicator-characters-:used:-in--an:unquoted:::::::string")}
);

// Regression tests

mk_test!(issue_13a;
r"
foo:
- baz
bar: bax
" => map! { "foo" => seq!("baz"); "bar" => "bax"}
);

mk_test!(issue_13b;
r"
value: {x: -0}
" => map! { "value" => map! { "x" => crate::Yaml::Int(0) }}
);

mk_test!(malformed seq;
r"
- a
-b
" => fail
);

mk_test!(issue_14;
r"a: -1" => map! { "a" => crate::Yaml::Int(-1) }
);

mk_test!(issue_15a;
r"a: foo[0]" => map! { "a": "foo[0]" }
);

mk_test!(issue_15b;
r"a: a - a" => map! { "a": "a - a"}
);

// Regression test: colons inside brackets/parens should not break scalar key parsing
#[test]
fn test_colon_inside_brackets_in_key() {
    // The colon after "Magix" should NOT be treated as a key-value separator
    // because it's inside the square brackets
    let yaml = r#"
+test.banner[Magix: Supercharge LLMs](http://example.com/bg.jpg):
  offset: 100
"#;
    let parsed = crate::parse(yaml).unwrap();
    let expected = map! {
        "+test.banner[Magix: Supercharge LLMs](http://example.com/bg.jpg)" => map! {
            "offset" => crate::Yaml::Int(100)
        }
    };
    assert_eq!(parsed, expected);
}

#[test]
fn test_colon_inside_brackets_multiple() {
    // Multiple colons inside brackets
    let yaml = r#"
+test.images[Like building furniture: not 3D-printing: use pre-made parts.]():
  images:
    - http://example.com/img0.png
"#;
    let parsed = crate::parse(yaml).unwrap();
    let expected = map! {
        "+test.images[Like building furniture: not 3D-printing: use pre-made parts.]()" => map! {
            "images" => seq!("http://example.com/img0.png")
        }
    };
    assert_eq!(parsed, expected);
}

// UTF-8 and field order tests

#[test]
fn test_utf8_chinese_in_block_scalar() {
    // Test that Chinese characters in literal block scalars are preserved correctly
    let yaml = r#"info: |
  请输入简短的描述
  最多200字
"#;
    let parsed = crate::parse(yaml).unwrap();
    if let crate::Yaml::Mapping(entries) = parsed {
        assert_eq!(entries.len(), 1);
        if let crate::Yaml::String(s) = &entries[0].value {
            assert!(s.contains("请输入简短的描述"), "Chinese text not preserved: {}", s);
            assert!(s.contains("最多200字"), "Chinese text not preserved: {}", s);
        } else {
            panic!("Expected String for block scalar");
        }
    } else {
        panic!("Expected mapping");
    }
}

#[test]
fn test_utf8_chinese_in_block_scalar_to_json() {
    // Test that Chinese characters survive YAML -> JSON conversion
    let yaml = r#"info: |
  你好世界
  这是测试
"#;
    let parsed = crate::parse(yaml).unwrap();
    let json = parsed.to_json();
    let obj = json.as_object().unwrap();
    let info = obj.get("info").unwrap().as_str().unwrap();
    assert!(info.contains("你好世界"), "Chinese not in JSON: {}", info);
    assert!(info.contains("这是测试"), "Chinese not in JSON: {}", info);
}

#[test]
fn test_utf8_mixed_content_block_scalar() {
    // Test mixed ASCII and multi-byte UTF-8 content
    let yaml = r#"description: |
  Hello 世界
  Price: ¥100
  Temperature: 25°C
"#;
    let parsed = crate::parse(yaml).unwrap();
    if let crate::Yaml::Mapping(entries) = parsed {
        if let crate::Yaml::String(s) = &entries[0].value {
            assert!(s.contains("Hello 世界"), "Mixed content not preserved: {}", s);
            assert!(s.contains("¥100"), "Yen symbol not preserved: {}", s);
            assert!(s.contains("25°C"), "Degree symbol not preserved: {}", s);
        } else {
            panic!("Expected String");
        }
    } else {
        panic!("Expected mapping");
    }
}
