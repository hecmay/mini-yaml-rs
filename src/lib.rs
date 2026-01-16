#![warn(clippy::all, clippy::pedantic)]
mod bytes;
mod errors;
mod parse;
mod tests;

pub use crate::errors::YamlParseError;

use wasm_bindgen::prelude::*;

pub(crate) type Result<T> = std::result::Result<T, YamlParseError>;

use parse::Parser;

use serde_json::{Map, Value};
use std::{fmt, fmt::Display};
#[cfg_attr(test, derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
/// A Yaml Element
pub enum Yaml<'a> {
    /// A literal value, losslessly interpreted as a string
    Scalar(&'a str),

    /// An owned string value, used for literal block scalars
    String(String),

    /// An integer value, parsed from `!int` tag
    Int(i64),

    /// A floating-point value, parsed from `!float` tag
    Float(f64),

    /// A boolean value, parsed from `!bool` tag
    Bool(bool),

    /// A sequence of values in flow style
    /// `[x, y, z]`
    /// or in block style
    /// ```yaml
    ///     - x
    ///     - y
    ///     - z
    /// ```
    Sequence(Vec<Yaml<'a>>),

    /// A mapping from key to value in flow style
    /// `{x: X, y: Y, z: Z}`
    /// or in block style
    /// ```yaml
    ///     x: X
    ///     y: Y
    ///     z: Z
    /// ```
    Mapping(Vec<Entry<'a>>),
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum PrintStyle {
    Block,
    #[allow(unused)]
    Flow,
}

fn print_indent(indent: usize, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:indent$}", "", indent = indent)
}

/// Check if a Yaml node is a tagged mapping (has __type as first key).
/// Returns the tag name if so.
fn get_tag_name<'a>(node: &'a Yaml<'a>) -> Option<&'a str> {
    match node {
        Yaml::Mapping(map) => match map.first() {
            Some(Entry {
                key: Yaml::Scalar("__type"),
                value: Yaml::Scalar(tag),
            }) => Some(tag),
            _ => None,
        },
        _ => None,
    }
}

/// Check if a Yaml value is a simple scalar type
fn is_scalar(node: &Yaml<'_>) -> bool {
    matches!(
        node,
        Yaml::Scalar(..) | Yaml::String(..) | Yaml::Int(..) | Yaml::Float(..) | Yaml::Bool(..)
    )
}

const INDENT_AMT: usize = 2;

/// Print a value after ":" has been written. Handles tagged mappings inline.
/// Returns true if it handled the value (used for continue in loops).
fn print_value_after_colon(
    value: &Yaml<'_>,
    indent: usize,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    // Check if value is a tagged mapping - print tag inline
    if let Some(tag) = get_tag_name(value) {
        if let Yaml::Mapping(value_map) = value {
            write!(f, " !{}", tag)?;
            // Check if it's __type only (empty tagged mapping)
            if value_map.len() == 1 {
                writeln!(f, " {{}}")?;
                return Ok(());
            }
            // Check if it's __type + __value only
            if value_map.len() == 2 {
                if let Some(second) = value_map.get(1) {
                    if let Yaml::Scalar("__value") = &second.key {
                        write!(f, " ")?;
                        print_yaml(&second.value, indent, f, PrintStyle::Block)?;
                        writeln!(f)?;
                        return Ok(());
                    }
                }
            }
            // Print remaining fields on new lines
            writeln!(f)?;
            print_mapping_entries(value_map.iter().skip(1), indent + INDENT_AMT, f)?;
            return Ok(());
        }
    }
    // Regular value handling
    if is_scalar(value) {
        write!(f, " ")?;
        print_yaml(value, indent, f, PrintStyle::Block)?;
        writeln!(f)?;
    } else {
        writeln!(f)?;
        print_yaml(value, indent + INDENT_AMT, f, PrintStyle::Block)?;
    }
    Ok(())
}

/// Print mapping entries (key: value pairs) at the given indent level
fn print_mapping_entries<'a, I>(entries: I, indent: usize, f: &mut fmt::Formatter) -> fmt::Result
where
    I: Iterator<Item = &'a Entry<'a>>,
{
    for entry in entries {
        // Print key
        if is_scalar(&entry.key) {
            print_indent(indent, f)?;
            print_yaml(&entry.key, indent, f, PrintStyle::Block)?;
        } else {
            print_yaml(&entry.key, indent + INDENT_AMT, f, PrintStyle::Block)?;
            print_indent(indent, f)?;
        }
        write!(f, ":")?;
        print_value_after_colon(&entry.value, indent, f)?;
    }
    Ok(())
}

fn print_yaml(
    node: &Yaml<'_>,
    indent: usize,
    f: &mut fmt::Formatter,
    style: PrintStyle,
) -> fmt::Result {
    match node {
        Yaml::Scalar(slice) => write!(f, "{}", slice),
        Yaml::String(s) => write!(f, "{}", s),
        Yaml::Int(i) => write!(f, "{}", i),
        Yaml::Float(fl) => write!(f, "{}", fl),
        Yaml::Bool(b) => write!(f, "{}", b),
        Yaml::Sequence(seq) => {
            match style {
                PrintStyle::Block => {
                    for el in seq.iter() {
                        print_indent(indent, f)?;
                        write!(f, "-")?;
                        if is_scalar(el) {
                            write!(f, " ")?;
                            print_yaml(el, indent, f, PrintStyle::Block)?;
                            writeln!(f)?;
                        } else if let Yaml::Sequence(..) = el {
                            writeln!(f)?;
                            print_yaml(el, indent + INDENT_AMT, f, style)?;
                        } else if let Yaml::Mapping(map) = el {
                            // Print first entry on same line as "-" if key is simple
                            if let Some((first, rest)) = map.split_first() {
                                let entry_indent = indent + INDENT_AMT;
                                if is_scalar(&first.key) {
                                    write!(f, " ")?;
                                    print_yaml(&first.key, indent, f, PrintStyle::Block)?;
                                } else {
                                    writeln!(f)?;
                                    print_yaml(
                                        &first.key,
                                        entry_indent + INDENT_AMT,
                                        f,
                                        PrintStyle::Block,
                                    )?;
                                    print_indent(entry_indent, f)?;
                                }
                                write!(f, ":")?;
                                print_value_after_colon(&first.value, entry_indent, f)?;
                                print_mapping_entries(rest.iter(), entry_indent, f)?;
                            } else {
                                writeln!(f, " {{}}")?;
                            }
                        }
                    }
                }
                PrintStyle::Flow => {
                    write!(f, "[ ")?;
                    let last_idx = seq.len() - 1;
                    for (idx, elem) in seq.iter().enumerate() {
                        if idx == last_idx {
                            write!(f, "{}", elem)?;
                        } else {
                            write!(f, "{}, ", elem)?;
                        }
                    }
                    write!(f, " ]")?;
                }
            }
            Ok(())
        }
        Yaml::Mapping(map) => {
            match style {
                PrintStyle::Block => {
                    // Check if this is a tagged mapping (__type field)
                    if let Some(tag) = get_tag_name(node) {
                        print_indent(indent, f)?;
                        write!(f, "!{}", tag)?;
                        // Check if it's __type + __value only
                        if map.len() == 2 {
                            if let Some(second) = map.get(1) {
                                if let Yaml::Scalar("__value") = &second.key {
                                    write!(f, " ")?;
                                    print_yaml(&second.value, indent, f, PrintStyle::Block)?;
                                    writeln!(f)?;
                                    return Ok(());
                                }
                            }
                        }
                        // Print remaining fields (skip __type)
                        writeln!(f)?;
                        print_mapping_entries(map.iter().skip(1), indent, f)?;
                        return Ok(());
                    }
                    // Regular mapping
                    print_mapping_entries(map.iter(), indent, f)?;
                }
                PrintStyle::Flow => {
                    write!(f, "{{")?;
                    let last_idx = map.len() - 1;
                    for (idx, entry) in map.iter().enumerate() {
                        if idx == last_idx {
                            write!(f, "{}", entry)?;
                        } else {
                            write!(f, "{}, ", entry)?;
                        }
                    }
                    write!(f, "}}")?;
                }
            }
            Ok(())
        }
    }
}

impl Display for Yaml<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_yaml(&self, 0, f, PrintStyle::Block)
    }
}

impl Yaml<'_> {
    /// Convert the Yaml value to a serde_json::Value.
    /// All scalars are treated as strings.
    /// This format is compatible with SQLite JSON extension.
    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Yaml::Scalar(s) => Value::String((*s).to_string()),
            Yaml::String(s) => Value::String(s.clone()),
            Yaml::Int(i) => Value::Number((*i).into()),
            Yaml::Float(f) => {
                Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into()))
            }
            Yaml::Bool(b) => Value::Bool(*b),
            Yaml::Sequence(seq) => Value::Array(seq.iter().map(|item| item.to_json()).collect()),
            Yaml::Mapping(entries) => {
                let mut map = Map::new();
                for entry in entries {
                    let key = match &entry.key {
                        Yaml::Scalar(s) => (*s).to_string(),
                        Yaml::Int(i) => i.to_string(),
                        Yaml::Float(f) => f.to_string(),
                        Yaml::Bool(b) => b.to_string(),
                        other => other.to_json().to_string(),
                    };
                    map.insert(key, entry.value.to_json());
                }
                Value::Object(map)
            }
        }
    }

    /// Convert the Yaml value to a serde_json::Value with mx transformation.
    ///
    /// The top-level value must be an object with keys matching the format
    /// `+name[label](value)` where `(value)` is optional.
    /// The key becomes `+name`, with `__name` set to the `[...]` content
    /// and `__value` set to the `(...)` content if present.
    ///
    /// If the format is invalid, returns `{"+error": {"__name": "error message", "__value": "yaml content"}}`
    #[must_use]
    pub fn to_mx(&self) -> Value {
        // Top level must be an object (Mapping)
        let entries = match self {
            Yaml::Mapping(entries) => entries,
            _ => {
                return Self::make_mx_error("Top level value must be an object", &self.to_string());
            }
        };

        let mut result_map = Map::new();

        for entry in entries {
            let key = match &entry.key {
                Yaml::Scalar(s) => (*s).to_string(),
                Yaml::Int(i) => i.to_string(),
                Yaml::Float(f) => f.to_string(),
                Yaml::Bool(b) => b.to_string(),
                other => other.to_json().to_string(),
            };

            if let Some((name_part, bracket_content, paren_content)) = Self::parse_mx_key(&key) {
                // Build the new key: +name
                let new_key = format!("+{}", name_part);

                // Build the value object with __name and optionally __value
                let mut value_obj = match entry.value.to_json() {
                    Value::Object(m) => m,
                    other => {
                        // If the value is not an object, wrap it
                        let mut m = Map::new();
                        m.insert("__content".to_string(), other);
                        m
                    }
                };

                value_obj.insert("__name".to_string(), Value::String(bracket_content));
                if let Some(paren) = paren_content {
                    value_obj.insert("__value".to_string(), Value::String(paren));
                }

                result_map.insert(new_key, Value::Object(value_obj));
            } else {
                // Key doesn't match the expected format
                return Self::make_mx_error(
                    &format!(
                        "Key '{}' does not match expected format +name[label](value)",
                        key
                    ),
                    &self.to_string(),
                );
            }
        }

        Value::Object(result_map)
    }

    /// Parse an mx key format: +name[label](value) where (value) is optional.
    /// Returns (name, bracket_content, optional_paren_content) on success.
    /// Allows any characters inside [] and ().
    fn parse_mx_key(key: &str) -> Option<(String, String, Option<String>)> {
        let key = key.strip_prefix('+')?;

        // Find the first '[' - everything before is the name
        let bracket_start = key.find('[')?;
        let name_part = &key[..bracket_start];

        // Name must not contain []()
        if name_part
            .chars()
            .any(|c| matches!(c, '[' | ']' | '(' | ')'))
        {
            return None;
        }

        // Check if we have a paren section at the end
        let (bracket_end, paren_content) = if key.ends_with(')') {
            // Find the matching '(' by scanning backwards
            let paren_close = key.len() - 1;
            let after_bracket = &key[bracket_start + 1..];

            // Find the last '](' pattern which separates bracket from paren
            if let Some(sep_pos) = after_bracket.rfind("](") {
                let bracket_end = bracket_start + 1 + sep_pos;
                let paren_start = bracket_end + 2; // skip "]("
                let paren_content = &key[paren_start..paren_close];
                (bracket_end, Some(paren_content.to_string()))
            } else {
                return None;
            }
        } else if key.ends_with(']') {
            // No paren section, bracket goes to the end
            (key.len() - 1, None)
        } else {
            return None;
        };

        let bracket_content = &key[bracket_start + 1..bracket_end];

        Some((
            name_part.to_string(),
            bracket_content.to_string(),
            paren_content,
        ))
    }

    fn make_mx_error(message: &str, yaml_content: &str) -> Value {
        let mut error_inner = Map::new();
        error_inner.insert("__name".to_string(), Value::String(message.to_string()));
        error_inner.insert(
            "__value".to_string(),
            Value::String(yaml_content.to_string()),
        );
        let mut error_obj = Map::new();
        error_obj.insert("+error".to_string(), Value::Object(error_inner));
        Value::Object(error_obj)
    }
}
#[cfg_attr(test, derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
/// A Yaml map entry
pub struct Entry<'a> {
    /// The key associated with the entry
    #[cfg_attr(test, serde(borrow))]
    pub key: Yaml<'a>,
    /// The value which the key maps to
    #[cfg_attr(test, serde(borrow))]
    pub value: Yaml<'a>,
}

impl<'a> Entry<'a> {
    #[allow(clippy::must_use_candidate)]
    pub fn new(key: Yaml<'a>, value: Yaml<'a>) -> Self {
        Self { key, value }
    }
}

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

/// Parse Yaml input. Returns the top level Yaml element on success
/// # Errors
/// Returns `Err` if the input is invalid Yaml, with a message indicating
/// where the error occurred and possibly more information on the cause
pub fn parse(input: &str) -> Result<Yaml<'_>> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

// WASM bindings

/// Parse YAML string and return JSON string.
/// Returns a JSON string on success, or throws an error on parse failure.
#[wasm_bindgen(js_name = parseYaml)]
pub fn parse_yaml_to_json(input: &str) -> std::result::Result<String, JsError> {
    let yaml = parse(input).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(yaml.to_json().to_string())
}

/// Parse YAML string and return mx-formatted JSON string.
/// Returns a JSON string with mx transformation on success, or throws an error on parse failure.
#[wasm_bindgen(js_name = parseYamlToMx)]
pub fn parse_yaml_to_mx(input: &str) -> std::result::Result<String, JsError> {
    let yaml = parse(input).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(yaml.to_mx().to_string())
}
