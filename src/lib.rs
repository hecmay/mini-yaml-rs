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

fn print_yaml(
    node: &Yaml<'_>,
    indent: usize,
    f: &mut fmt::Formatter,
    style: PrintStyle,
) -> fmt::Result {
    const INDENT_AMT: usize = 2;
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
                        match el {
                            Yaml::Scalar(slice) => writeln!(f, " {scal}", scal = slice)?,
                            Yaml::String(s) => writeln!(f, " {}", s)?,
                            Yaml::Int(i) => writeln!(f, " {}", i)?,
                            Yaml::Float(fl) => writeln!(f, " {}", fl)?,
                            Yaml::Bool(b) => writeln!(f, " {}", b)?,
                            Yaml::Sequence(..) | Yaml::Mapping(..) => {
                                #[allow(clippy::write_with_newline)]
                                write!(f, "\n")?;
                                print_yaml(el, indent + INDENT_AMT, f, style)?;
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
                    for entry in map.iter() {
                        match &entry.key {
                            Yaml::Scalar(..)
                            | Yaml::String(..)
                            | Yaml::Int(..)
                            | Yaml::Float(..)
                            | Yaml::Bool(..) => {
                                print_indent(indent, f)?;
                                print_yaml(&entry.key, indent, f, PrintStyle::Block)?;
                                write!(f, " ")?;
                            }
                            Yaml::Sequence(..) | Yaml::Mapping(..) => {
                                print_yaml(&entry.key, indent + INDENT_AMT, f, PrintStyle::Block)?;
                                print_indent(indent, f)?;
                            }
                        }
                        write!(f, ":")?;
                        match &entry.value {
                            Yaml::Scalar(..)
                            | Yaml::String(..)
                            | Yaml::Int(..)
                            | Yaml::Float(..)
                            | Yaml::Bool(..) => {
                                write!(f, " ")?;
                                print_yaml(&entry.value, indent, f, PrintStyle::Block)?;
                                #[allow(clippy::write_with_newline)]
                                write!(f, "\n")?;
                            }
                            Yaml::Sequence(..) | Yaml::Mapping(..) => {
                                #[allow(clippy::write_with_newline)]
                                write!(f, "\n")?;
                                print_yaml(&entry.value, indent + INDENT_AMT, f, PrintStyle::Block)?
                            }
                        }
                    }
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
        // match self.key
        write!(f, "{} : {}", self.key, self.value)
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
