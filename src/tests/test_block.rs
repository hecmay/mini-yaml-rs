#![cfg(test)]
#![allow(clippy::pedantic)]

// Block Sequence

mk_test!(
simple block sequence;
r#"
- a
- sequence
- of
- yaml
-   nodes
- "in"
- 'block'
- ' form '"# => seq!("a", "sequence", "of", "yaml", "nodes", "in", "block", " form ")
);

mk_test!(
block sequence flow seq;
r#"
- a
- sequence
- with
-       [ a, sequence, "as ", 'a', node  ]
"# => seq!("a", "sequence", "with", seq!("a", "sequence", "as ", "a", "node"))
);

mk_test!(
block sequence flow map;
r#"
- a
- block
- sequence
- '  "with" '
- { a : "flow", mapping : ' as ', a : " 'node' "}
"# => seq!("a", "block", "sequence", "  \"with\" ", map!{ "a" : "flow", "mapping" : " as ", "a" : " 'node' "})
);

mk_test!(
block sequence nested;
r#"
-
  - " a "
  - ' nested'
  - ' " block  " '
  - sequence
-
  - with
  - two
  - "'e l e m e n t s'"
"# => seq!(seq!(" a ", " nested", " \" block  \" ", "sequence"), seq!("with", "two", "'e l e m e n t s'"))
);

mk_test!(
super simple block sequence nested;
r#"
-
  - " a "
  - ' nested'
"# => seq!(seq!(" a ", " nested"))
);

mk_test!(
block sequence multiple nested;
r##"
-
    - "a"
    - "nested"
    - block
    -
        - sequence
        - with
    - lots
    -
        - of
        - different
-
    - indent
    - levels
    -
        - [with, a, flow, sequence for good measure]
- "' the '"
- end
"## =>
    seq!(
        seq!(
            "a",
            "nested",
            "block",
            seq!(
                "sequence",
                "with"
            ),
            "lots",
            seq!(
                "of",
                "different"
            )
        ),
        seq!(
            "indent",
            "levels",
            seq!(
                seq!(
                    "with",
                    "a",
                    "flow",
                    "sequence for good measure"
                )
            )
        ),
        "' the '",
        "end"
    )
);

// Block mappings

mk_test!(
super simple;
r#"
key: value
key2: value2
"# => map! { "key" : "value", "key2" : "value2"}
);

mk_test!(
block mapping simple;
r#"
key: value
key2: value2
and: another
now with: "some quotes"
'and': "a 'few' more"
"# => map!{ "key" : "value", "key2" : "value2", "and" : "another", "now with" : "some quotes", "and" : "a 'few' more"}
);

mk_test!(
block mapping flow;
r#"
key: {the: " value ", 'i s': a, flow: mapping}
mind: blown
wait: [it, works, with, flow, sequences too]
[now, how, about, one, with, the, flow, mapping, as]: a key
"# => map!{
    "key" => map!{ "the" : " value ", "i s": "a", "flow":"mapping"};
    "mind" => "blown";
    "wait" => seq!("it", "works", "with", "flow", "sequences too");
    seq!("now", "how", "about", "one", "with", "the", "flow", "mapping", "as") => "a key"
}
);

mk_test!(
block mapping nested blocks;
r#"
key:
  the: value
  is:
    nested: mappings
    wow:
      - with a block seq
      - too
and: done
"# => map!{
    "key" => map! {
        "the" => "value";
        "is" => map! {
            "nested" => "mappings";
            "wow" => seq!("with a block seq", "too")
        }
    };
    "and" => "done"
}
);
