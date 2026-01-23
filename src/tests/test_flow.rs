#![cfg(test)]
#![allow(clippy::pedantic)]

// Flow Sequences

mk_test!(
    empty flow sequence;
    r"[   ]" => seq!()
);

mk_test!(
    one element flow sequence;
    r"[   element]" => seq!("element")
);

mk_test!(
    simple flow sequence no quotes;
    r"[      a,      b     , c  ,d , e       ]" => seq!("a", "b", "c", "d", "e")
);

mk_test!(
    simple flow sequence mixed quotes;
    r#"[ "a", 'b' , "  c ", d, ' e  ' ]"# => seq!("a", "b", "  c ", "d", " e  ")
);

mk_test!(
    multiple flow sequence no quotes;
    r"[[ a, b, c ,d, e   ] , [ f, g, h, i , j ]]" =>
        seq!(
            seq!("a", "b","c", "d", "e"),
            seq!("f", "g", "h", "i", "j")
        )
);

mk_test!(
    mixed kind flow sequence no quotes;
    r"[[ a, b, c], el]" => seq!(seq!("a", "b", "c"), "el")
);

mk_test!(
    mixed kind flow sequence quotes;
    r#"[" elem " , [ a, 'b ' , "   c "]]"# => seq!(" elem ", seq!("a", "b ", "   c "))
);

// Multi-line flow sequences

mk_test!(
    multi line flow sequence simple;
    r#"[
  a,
  b,
  c
]"# => seq!("a", "b", "c")
);

mk_test!(
    multi line flow sequence with comments;
    r#"[
  a,  # first item
  b,  # second item
  c   # third item
]"# => seq!("a", "b", "c")
);

mk_test!(
    multi line flow sequence nested;
    r#"[
  [1, 2, 3],
  [4, 5, 6]
]"# => seq!(
        seq!(crate::Yaml::Int(1), crate::Yaml::Int(2), crate::Yaml::Int(3)),
        seq!(crate::Yaml::Int(4), crate::Yaml::Int(5), crate::Yaml::Int(6))
    )
);

// Flow mappings

mk_test!(
    simple flow mapping;
    r"{ k : v }" => map!{ "k" : "v" }
);

mk_test!(
    multiple entry flow mapping;
    r"{ k1 : v1 ,   k2 :     v2    }" => map!{ "k1" : "v1", "k2":"v2" }
);

mk_test!(
    seq value flow mapping;
    r"{ k1 : [ a , b, c] }" => map! {
        "k1" => seq!("a", "b", "c")
    }
);

mk_test!(
    seq key flow mapping;
    r"{ [ a, map, as a key ] : val }" => map! {
        seq!("a", "map", "as a key") => "val"
    }
);

mk_test!(
    seq entry flow mapping;
    r"{ [ a, seq, as a key ] : [  a, seq, as a value ]  }" => map! {
        seq!("a", "seq", "as a key") => seq!("a", "seq", "as a value")
    }
);

mk_test!(
    map key flow mapping;
    r"{ { a map : as a key} : value }" => map! {
        map! { "a map" : "as a key" } => "value"
    }
);

mk_test!(
    map entry flow mapping;
    r"{ { a   map : as a key} : { 'a map ': as a value }   }" => map! {
        map! { "a   map" : "as a key" } => map! { "a map " : "as a value" }
    }
);

// Multi-line flow mappings

mk_test!(
    multi line flow mapping simple;
    r#"{
  k1: v1,
  k2: v2
}"# => map!{ "k1" : "v1", "k2" : "v2" }
);

mk_test!(
    multi line flow mapping with comments;
    r#"{
  name: John,  # user name
  city: NYC    # user city
}"# => map!{ "name" : "John", "city" : "NYC" }
);

mk_test!(
    multi line flow mapping nested;
    r#"{
  outer: {
    inner: value
  }
}"# => map!{ "outer" => map!{ "inner" : "value" } }
);

mk_test!(
    multi line flow mapping with sequence;
    r#"{
  items: [
    a,
    b,
    c
  ]
}"# => map!{ "items" => seq!("a", "b", "c") }
);
