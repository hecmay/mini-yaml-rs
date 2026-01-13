#![warn(clippy::all, clippy::pedantic)]
mod bytes;
mod errors;
mod parse;
mod tests;

pub use crate::errors::YamlParseError;

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
                            Yaml::Scalar(..) | Yaml::Int(..) | Yaml::Float(..) | Yaml::Bool(..) => {
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
                            Yaml::Scalar(..) | Yaml::Int(..) | Yaml::Float(..) | Yaml::Bool(..) => {
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
