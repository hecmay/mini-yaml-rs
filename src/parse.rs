use crate::bytes::ByteExt;
use crate::{Entry, Yaml, YamlParseError};
use core::iter::{Iterator, Peekable};
use std::str::Bytes;

use crate::Result;
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParseContext {
    FlowIn,
    FlowOut,
    FlowKey,
    BlockIn,
    BlockOut,
    BlockKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseContextKind {
    FlowMapping,
    Flow,
    BlockMapping,
    Block,
}

pub(crate) struct Parser<'a> {
    current: u8,
    stream: Peekable<Bytes<'a>>,
    bytes: &'a [u8],
    source: &'a str,
    idx: usize,
    indent: usize,
    expected: Vec<u8>,
    contexts: Vec<ParseContext>,
}

impl<'a, 'b> Parser<'a> {
    pub(crate) fn new(source: &'a str) -> Result<Self> {
        let mut stream = source.bytes().peekable();
        let first = stream.next().ok_or_else(|| YamlParseError {
            line: 0,
            col: 0,
            msg: Some("expected input".into()),
            source: None,
        })?;
        Ok(Self {
            current: first,
            bytes: source.as_bytes(),
            stream,
            source,
            idx: 0,
            indent: 0,
            expected: Vec::new(),
            contexts: Vec::new(),
        })
    }

    fn start_context(&mut self, context_kind: ParseContextKind) -> Result<()> {
        let context = match self.context() {
            Some(ctx) => match context_kind {
                ParseContextKind::Flow => ParseContext::FlowIn,
                ParseContextKind::FlowMapping => ParseContext::FlowKey,
                ParseContextKind::Block => match ctx {
                    ParseContext::FlowIn | ParseContext::FlowOut | ParseContext::FlowKey => {
                        return self.parse_error_with_msg(
                            "block collections cannot be values in flow collections",
                        )
                    }
                    ParseContext::BlockIn | ParseContext::BlockOut | ParseContext::BlockKey => {
                        ParseContext::BlockIn
                    }
                },
                ParseContextKind::BlockMapping => ParseContext::BlockKey,
            },
            None => match context_kind {
                ParseContextKind::Flow => ParseContext::FlowIn,
                ParseContextKind::FlowMapping => ParseContext::FlowKey,
                ParseContextKind::Block => ParseContext::BlockOut,
                ParseContextKind::BlockMapping => ParseContext::BlockKey,
            },
        };
        self.contexts.push(context);
        Ok(())
    }

    fn end_context(&mut self, expect: ParseContextKind) -> Result<()> {
        if let Some(actual) = self.contexts.pop() {
            let ctx_matches = match expect {
                ParseContextKind::Flow => {
                    matches!(actual, ParseContext::FlowIn | ParseContext::FlowOut)
                }
                ParseContextKind::FlowMapping => matches!(actual, ParseContext::FlowKey),
                ParseContextKind::Block => {
                    matches!(actual, ParseContext::BlockIn | ParseContext::BlockOut)
                }
                ParseContextKind::BlockMapping => matches!(actual, ParseContext::BlockKey),
            };
            if ctx_matches {
                Ok(())
            } else {
                self.parse_error_with_msg(format!(
                    "expected but failed to end context {:?}, instead found {:?}",
                    expect, actual
                ))
            }
        } else {
            self.parse_error_with_msg(format!(
                "expected context {:?} but no contexts remained",
                expect
            ))
        }
    }

    fn context(&self) -> Option<ParseContext> {
        self.contexts.last().copied()
    }

    fn bump(&mut self) -> bool {
        match self.stream.next() {
            Some(byte) => {
                self.idx += 1;
                self.current = byte;
                true
            }
            None => false,
        }
    }

    fn bump_newline(&mut self) -> bool {
        match self.stream.next() {
            Some(b'\n') | Some(b'\r') => {
                self.idx += 1; // Account for the newline char consumed from stream
                self.bump()
            }
            Some(byte) => {
                self.idx += 1;
                self.current = byte;
                true
            }
            None => false,
        }
    }

    fn advance(&mut self) -> Result<()> {
        if self.bump() {
            Ok(())
        } else {
            self.parse_error_with_msg("unexpected end of input")
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.stream.peek().copied()
    }

    fn at_end(&self) -> bool {
        self.idx == self.bytes.len() - 1
    }

    fn parse_mapping_maybe(&mut self, node: Yaml<'a>) -> Result<Yaml<'a>> {
        self.chomp_whitespace();
        self.chomp_comment();
        match self.current {
            b':' if !matches!(self.expected.last(), Some(b'}') | Some(b':')) => {
                self.parse_mapping_block(node)
            }
            _ => Ok(node),
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Yaml<'a>> {
        let context = self.context();
        let peeked = self.peek();
        let res = match self.current {
            b'#' => {
                self.chomp_comment();
                self.parse()?
            }
            b'-' if self.check_ahead_1(|val| val == b'-')
                && self.check_ahead_n(2, |val| val == b'-') =>
            {
                self.bump();
                self.bump();
                self.bump();
                self.parse()?
            }
            b'\n' | b'\r' => {
                self.chomp_newlines()?;
                self.indent = 0;
                self.parse()?
            }
            byt if byt.is_scalar_start(peeked, context) => self.parse_maybe_scalar()?,
            b'{' => {
                self.expected.push(b'}');
                let res = self.parse_mapping_flow()?;
                if let Some(b'}') = self.expected.last() {
                    self.pop_if_match(b'}')?;
                }
                self.parse_mapping_maybe(res)?
            }
            b'[' => {
                let node = self.parse_sequence_flow()?;
                self.parse_mapping_maybe(node)?
            }
            b'-' => match self.peek() {
                Some(byt) if byt.is_linebreak() || byt.is_ws() => self.parse_sequence_block()?,
                byt => unreachable!("unexpected {:?}", byt.map(char::from)),
            },

            b'}' | b']' => {
                return self.parse_error_with_msg(format!(
                    r#"unexpected symbol '{}'"#,
                    char::from(self.current)
                ))
            }
            b if b.is_ws() => {
                self.chomp_indent();
                if self.at_end() {
                    return self.parse_error_with_msg("unexpected end of input");
                }
                self.parse()?
            }
            b'!' => self.parse_tagged_value()?,
            b'|' => self.parse_literal_block_scalar()?,
            b'>' => self.parse_folded_block_scalar()?,
            _ => return self.parse_error_with_msg("failed to parse at top level"),
        };
        Ok(res)
    }
    pub(crate) fn parse_maybe_scalar(&mut self) -> Result<Yaml<'a>> {
        match self.context() {
            None => {
                self.start_context(ParseContextKind::BlockMapping)?;
                let node = self.parse_scalar()?;
                self.end_context(ParseContextKind::BlockMapping)?;
                self.parse_mapping_maybe(node)
            }
            Some(ctx) => match ctx {
                ParseContext::FlowIn | ParseContext::FlowOut | ParseContext::FlowKey => {
                    self.parse_scalar()
                }
                _ => {
                    self.start_context(ParseContextKind::BlockMapping)?;
                    let node = self.parse_scalar()?;
                    self.end_context(ParseContextKind::BlockMapping)?;
                    self.parse_mapping_maybe(node)
                }
            },
        }
    }

    pub(crate) fn parse_scalar(&mut self) -> Result<Yaml<'a>> {
        let context = self.context();
        match self.current {
            // Double-quoted string: strip the quotes
            b'\"' => {
                self.advance()?; // consume opening quote
                let scal_start = self.idx; // start after the quote
                let mut accept_dq = |tok: u8, _: Option<u8>| !matches!(tok, b'\"');
                let _ = self
                    .take_while(&mut accept_dq)
                    .map_err(|_| {
                        self.make_parse_error_with_msg("unexpected end of input; expected '\"'")
                    })?;
                let scal_end = self.idx; // end before the closing quote
                self.bump(); // consume closing quote
                let content = self.slice_range((scal_start, scal_end));
                Ok(Yaml::Scalar(content))
            }
            // Single-quoted string: strip the quotes
            b'\'' => {
                self.advance()?; // consume opening quote
                let scal_start = self.idx; // start after the quote
                let mut accept_sq = |tok: u8, _: Option<u8>| !matches!(tok, b'\'');
                self.take_while(&mut accept_sq)
                    .map_err(|_| {
                        self.make_parse_error_with_msg("unexpected end of input; expected '\''")
                    })?;
                let scal_end = self.idx; // end before the closing quote
                self.bump(); // consume closing quote
                let content = self.slice_range((scal_start, scal_end));
                Ok(Yaml::Scalar(content))
            }
            _ => {
                // Track bracket/paren depth to allow colons inside [] and ()
                let mut bracket_depth: i32 = 0;
                let mut paren_depth: i32 = 0;

                let mut accept = |tok: u8, nxt: Option<u8>| {
                    // Update bracket/paren depth
                    match tok {
                        b'[' => bracket_depth += 1,
                        b']' => bracket_depth = (bracket_depth - 1).max(0),
                        b'(' => paren_depth += 1,
                        b')' => paren_depth = (paren_depth - 1).max(0),
                        _ => {}
                    }

                    // When inside brackets or parens, allow colons even if followed by whitespace
                    if bracket_depth > 0 || paren_depth > 0 {
                        // Inside brackets/parens: allow everything except linebreak
                        // But still stop at # for comments
                        !tok.is_linebreak() && tok != b'#'
                    } else {
                        // Normal is_ns_plain behavior
                        tok.is_ns_plain(nxt, context)
                    }
                };

                let (start, mut end) = self.take_while(&mut accept).unwrap_or_else(|val| val);
                loop {
                    self.chomp_whitespace();
                    self.chomp_comment();
                    let (s, e) = self.take_while(&mut accept).unwrap_or_else(|val| val);
                    if s == e {
                        break;
                    } else {
                        end = e;
                    }
                    if self.at_end() {
                        break;
                    }
                }
                let entire_literal = self.slice_range((start, end));
                // Automatically infer type for unquoted scalars
                Ok(Self::infer_scalar_type(entire_literal))
            }
        }
    }

    /// Parse a tag name after the `!` character.
    /// Returns the tag name as a string slice.
    fn parse_tag(&mut self) -> Result<&'a str> {
        // Consume the '!'
        self.advance()?;

        // Capture tag name start
        let tag_start = self.idx;

        // Parse tag characters (alphanumeric, hyphen, underscore)
        while matches!(self.current, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_') {
            if !self.bump() {
                break;
            }
        }

        let tag_end = self.idx;
        let tag_name = self.slice_range((tag_start, tag_end));

        if tag_name.is_empty() {
            return self.parse_error_with_msg("expected tag name after '!'");
        }

        // Consume whitespace after tag
        self.chomp_whitespace();

        Ok(tag_name)
    }

    /// Parse a tagged value (!tagname value).
    /// All tags are wrapped in a mapping with __type field.
    fn parse_tagged_value(&mut self) -> Result<Yaml<'a>> {
        let tag_name = self.parse_tag()?;

        // Parse the value following the tag
        let value = self.parse()?;

        // Wrap the result based on value type
        let result = match value {
            Yaml::Mapping(mut entries) => {
                // Insert __type at the beginning
                entries.insert(
                    0,
                    Entry::new(Yaml::Scalar("__type"), Yaml::Scalar(tag_name)),
                );
                Yaml::Mapping(entries)
            }
            other => {
                // Wrap scalar or sequence in a mapping with __type and __value
                let entries = vec![
                    Entry::new(Yaml::Scalar("__type"), Yaml::Scalar(tag_name)),
                    Entry::new(Yaml::Scalar("__value"), other),
                ];
                Yaml::Mapping(entries)
            }
        };

        Ok(result)
    }

    /// Parse a boolean string value.
    /// Accepts: true/false, yes/no, on/off (case-insensitive)
    fn parse_bool(s: &str) -> Option<bool> {
        match s.to_lowercase().as_str() {
            "true" | "yes" | "on" => Some(true),
            "false" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    /// Infer the type of an unquoted scalar value.
    /// Returns Int, Float, Bool, or Scalar based on the content.
    fn infer_scalar_type(s: &str) -> Yaml<'_> {
        // Check for boolean values first
        if let Some(b) = Self::parse_bool(s) {
            return Yaml::Bool(b);
        }

        // Check for integer (digits with optional leading minus)
        if let Ok(i) = s.parse::<i64>() {
            return Yaml::Int(i);
        }

        // Check for float (contains decimal point or scientific notation)
        if s.contains('.') || s.contains('e') || s.contains('E') {
            if let Ok(f) = s.parse::<f64>() {
                return Yaml::Float(f);
            }
        }

        // Default to string
        Yaml::Scalar(s)
    }

    /// Parse a literal block scalar (|).
    /// Preserves newlines exactly as they appear.
    fn parse_literal_block_scalar(&mut self) -> Result<Yaml<'a>> {
        self.parse_block_scalar(false)
    }

    /// Parse a folded block scalar (>).
    /// Folds newlines into spaces, except for blank lines.
    fn parse_folded_block_scalar(&mut self) -> Result<Yaml<'a>> {
        self.parse_block_scalar(true)
    }

    /// Parse a block scalar (literal | or folded >).
    /// If `fold` is true, newlines are folded into spaces.
    fn parse_block_scalar(&mut self, fold: bool) -> Result<Yaml<'a>> {
        // Current character is | or >
        self.advance()?;

        // Parse optional chomping indicator (- or +) and indentation indicator (1-9)
        let mut chomp = 0i8; // 0 = clip (default), -1 = strip, 1 = keep
        let mut explicit_indent: Option<usize> = None;

        // Parse indicators (can be in any order: |2-, |-2, |-, |2, |+, etc.)
        for _ in 0..2 {
            match self.current {
                b'-' => {
                    chomp = -1;
                    self.bump();
                }
                b'+' => {
                    chomp = 1;
                    self.bump();
                }
                b'1'..=b'9' => {
                    explicit_indent = Some((self.current - b'0') as usize);
                    self.bump();
                }
                _ => break,
            }
        }

        // Skip any remaining whitespace and comments on the indicator line
        self.chomp_whitespace();
        self.chomp_comment();

        // Must have a newline after the indicator
        if !self.current.is_linebreak() {
            return self.parse_error_with_msg("expected newline after block scalar indicator");
        }

        // Skip the newline
        if !self.bump() {
            // End of input after indicator - return empty string
            return Ok(Yaml::String(String::new()));
        }

        let mut result = String::new();
        let mut trailing_newlines = 0usize;
        let mut content_indent: Option<usize> = explicit_indent;

        loop {
            // Skip empty lines (but track them for later)
            if self.current.is_linebreak() {
                trailing_newlines += 1;
                if !self.bump() {
                    break;
                }
                continue;
            }

            // Count leading whitespace for this line
            let mut line_indent = 0;
            while self.current.is_ws() {
                line_indent += 1;
                if !self.bump() {
                    break;
                }
            }

            // Check if this is an empty line (only whitespace followed by newline)
            if self.current.is_linebreak() {
                trailing_newlines += 1;
                if !self.bump() {
                    break;
                }
                continue;
            }

            // First content line determines the indentation
            if content_indent.is_none() {
                content_indent = Some(line_indent);
            }

            let content_indent = content_indent.unwrap();

            // Check if we've dedented (end of block)
            if line_indent < content_indent {
                // We need to "unread" the content we just read
                // Since we can't, we'll adjust the indent for the caller
                self.indent = line_indent;
                break;
            }

            // Add any accumulated blank lines
            for _ in 0..trailing_newlines {
                result.push('\n');
            }
            trailing_newlines = 0;

            // Add newline before content (except for first line)
            if !result.is_empty() {
                if fold {
                    result.push(' ');
                } else {
                    result.push('\n');
                }
            }

            // Add any extra indentation beyond content_indent
            for _ in content_indent..line_indent {
                result.push(' ');
            }

            // Collect the rest of the line
            while !self.current.is_linebreak() {
                result.push(self.current as char);
                if !self.bump() {
                    // End of input
                    break;
                }
            }

            // Move past the newline if present
            if self.current.is_linebreak() {
                if !self.bump() {
                    break;
                }
            } else {
                // End of input
                break;
            }
        }

        // Apply chomping
        match chomp {
            -1 => {
                // Strip: remove all trailing newlines (already done by not adding them)
            }
            0 => {
                // Clip: single trailing newline
                if !result.is_empty() {
                    result.push('\n');
                }
            }
            1 => {
                // Keep: preserve all trailing newlines
                result.push('\n');
                for _ in 0..trailing_newlines {
                    result.push('\n');
                }
            }
            _ => {}
        }

        Ok(Yaml::String(result))
    }

    fn lookup_line_col(&self) -> (usize, usize) {
        let err_off: usize = self.idx + 1;
        let mut off = 0;
        let mut line_len = 0;
        let mut chars = self.source.chars().map(|c| (c, c.len_utf8()));
        let mut line_lens = Vec::new();
        while let Some((chr, len)) = chars.next() {
            match chr {
                '\r' => {
                    if let Some(('\n', nxtlen)) = chars.next() {
                        line_lens.push(line_len + nxtlen + len);
                        line_len = 0;
                        continue;
                    }
                }
                '\n' => {
                    line_lens.push(line_len + len);
                    line_len = 0;
                    continue;
                }
                _ => line_len += len,
            }
        }
        let mut line_num = 0;
        for ((line_no, _), len) in self.source.lines().enumerate().zip(line_lens) {
            if err_off >= off && err_off < off + len {
                return (line_no + 1, err_off - off + 1);
            }
            line_num = line_no;
            off += len;
        }
        if err_off >= off {
            return (line_num + 1, err_off - off + 1);
        }
        eprintln!("Couldn't find error location, please report this bug");
        (0, 0)
    }

    #[allow(unused)]
    fn parse_error<T>(&self) -> Result<T> {
        let (line, col) = self.lookup_line_col();
        Err(YamlParseError {
            line,
            col,
            msg: Some(format!(
                r#"unexpectedly found "{}" while parsing"#,
                self.current
            )),
            source: None,
        })
    }

    fn make_parse_error_with_msg<S: Into<String>>(&self, msg: S) -> YamlParseError {
        let (line, col) = self.lookup_line_col();
        YamlParseError {
            line,
            col,
            msg: Some(msg.into()),
            source: None,
        }
    }

    fn parse_error_with_msg<T, S: Into<String>>(&self, msg: S) -> Result<T> {
        Err(self.make_parse_error_with_msg(msg))
    }

    pub(crate) fn parse_mapping_flow(&mut self) -> Result<Yaml<'a>> {
        match self.current {
            b'{' => (),
            _ => return self.parse_error_with_msg("expected left brace"),
        }
        self.advance()?;
        let mut entries: Vec<Entry<'a>> = Vec::new();
        loop {
            match &self.current {
                b'}' => {
                    self.bump();
                    return Ok(Yaml::Mapping(entries));
                }
                b',' => {
                    self.advance()?;
                }
                _ => {
                    self.expected.push(b':');
                    self.start_context(ParseContextKind::FlowMapping)?;
                    let key = self.parse()?;
                    self.end_context(ParseContextKind::FlowMapping)?;
                    self.chomp_whitespace();
                    self.chomp_comment();
                    match self.current {
                        b':' => {
                            self.pop_if_match(b':')?;
                            self.advance()?;
                            self.chomp_whitespace();
                            self.start_context(ParseContextKind::Flow)?;
                            let value = self.parse()?;
                            self.end_context(ParseContextKind::Flow)?;
                            self.chomp_whitespace();
                            self.chomp_comment();
                            entries.push(Entry { key, value })
                        }
                        // TODO: Provide error message
                        _ => return self.parse_error_with_msg("failed to parse flow mapping"),
                    }
                }
            }
        }
    }

    pub(crate) fn parse_mapping_block(&mut self, start_key: Yaml<'a>) -> Result<Yaml<'a>> {
        match self.context() {
            Some(ParseContext::FlowIn)
            | Some(ParseContext::FlowKey)
            | Some(ParseContext::FlowOut) => {
                return self
                    .parse_error_with_msg("block mappings may not appear in flow collections")
            }
            _ => {}
        }
        let indent = self.indent;
        match self.current {
            b':' => {
                self.advance()?;
                let mut entries = Vec::new();
                self.chomp_whitespace();
                self.chomp_comment();
                let value = self.parse()?;
                entries.push(Entry::new(start_key, value));
                loop {
                    match self.current {
                        _ if self.at_end() => break,
                        byt if byt.is_linebreak() => {
                            self.indent = 0;
                            if self.bump_newline() {
                                continue;
                            } else {
                                break;
                            }
                        }
                        byt if byt.is_ws() => {
                            self.chomp_indent();
                        }
                        b'#' => self.chomp_comment(),
                        _ if self.indent < indent => break,
                        _ => {
                            self.expected.push(b':');
                            let key = self.parse()?;
                            self.chomp_whitespace();
                            self.chomp_comment();
                            if let b':' = self.current {
                                self.pop_if_match(b':')?;
                                self.advance()?;
                                self.chomp_whitespace();
                                let value = self.parse()?;
                                entries.push(Entry::new(key, value));
                            } else {
                                // TODO: Provide error message
                                return self.parse_error_with_msg("failed to parse block mapping");
                            }
                        }
                    }
                }
                Ok(Yaml::Mapping(entries))
            }
            // TODO: Provide error message
            _ => self.parse_error_with_msg("failed to parse block mapping, expected ':'"),
        }
    }

    fn slice_range(&self, (start, end): (usize, usize)) -> &'a str {
        let end = usize::min(end, self.bytes.len());
        &self.source[start..end]
    }

    fn chomp_comment(&mut self) {
        if self.current == b'#' {
            self.bump();
            while !self.current.is_linebreak() {
                if !self.bump() {
                    break;
                }
            }
        }
    }

    fn chomp_whitespace(&mut self) {
        while let b' ' | b'\t' = self.current {
            if !self.bump() {
                break;
            }
        }
    }

    fn chomp_indent(&mut self) {
        let mut idt = 0;
        while let b' ' | b'\t' = self.current {
            if !self.bump() {
                break;
            }
            idt += 1;
        }
        self.indent = idt;
    }

    fn chomp_newlines(&mut self) -> Result<()> {
        while let b'\r' | b'\n' = self.current {
            self.advance()?;
        }
        Ok(())
    }

    pub(crate) fn parse_sequence_flow(&mut self) -> Result<Yaml<'a>> {
        self.start_context(ParseContextKind::Flow)?;
        match self.current {
            b'[' => {
                self.advance()?;
                let mut elements = Vec::new();
                loop {
                    match self.current {
                        b']' => {
                            self.bump();
                            self.end_context(ParseContextKind::Flow)?;
                            return Ok(Yaml::Sequence(elements));
                        }
                        b' ' | b'\t' => self.chomp_whitespace(),

                        b'#' => self.chomp_comment(),
                        _ => {
                            let elem = self.parse()?;
                            elements.push(elem);
                            self.chomp_whitespace();

                            match self.current {
                                b',' => {
                                    self.advance()?;
                                }
                                b'#' => self.chomp_comment(),
                                b']' => {
                                    self.bump();
                                    self.end_context(ParseContextKind::Flow)?;
                                    return Ok(Yaml::Sequence(elements));
                                }
                                // TODO: Provide error message
                                _ => {
                                    return self
                                        .parse_error_with_msg("failed to parse flow sequence")
                                }
                            }
                        }
                    }
                }
            }
            // TODO: Provide error message
            _ => self.parse_error_with_msg("failed to parse flow sequence"),
        }
    }

    fn check_ahead_1(&self, stop: impl Fn(u8) -> bool) -> bool {
        match self.bytes.get(self.idx + 1) {
            Some(&b) => stop(b),
            None => false,
        }
    }

    pub(crate) fn parse_sequence_block(&mut self) -> Result<Yaml<'a>> {
        match self.context() {
            Some(ParseContext::FlowIn)
            | Some(ParseContext::FlowKey)
            | Some(ParseContext::FlowOut) => {
                return self
                    .parse_error_with_msg("block sequences may not appear in flow collections")
            }
            _ => {}
        }
        self.start_context(ParseContextKind::Block)?;
        let indent = self.indent;
        match self.current {
            b'-' => {
                let mut seq = Vec::new();
                loop {
                    match self.current {
                        _ if self.at_end() => break,
                        b'#' => self.chomp_comment(),
                        byt if byt.is_linebreak() => {
                            self.indent = 0;
                            if self.bump_newline() {
                                continue;
                            } else {
                                break;
                            }
                        }
                        byt if byt.is_ws() => {
                            self.chomp_indent();
                        }
                        _ if self.indent < indent => break,
                        b'-' => {
                            if self.check_ahead_1(ByteExt::is_linebreak) {
                                self.advance()?;
                                self.advance()?;
                                self.indent = 0;
                                if self.current.is_ws() {
                                    self.chomp_indent();
                                    if self.indent < indent {
                                        break;
                                    } else {
                                        let node = self.parse()?;
                                        seq.push(node);
                                    }
                                } else if 0 < indent {
                                    break;
                                } else {
                                    let node = self.parse()?;
                                    seq.push(node);
                                }
                            } else if self.check_ahead_1(ByteExt::is_ws) {
                                self.advance()?;
                                self.advance()?;
                                // Update indent to account for "- " prefix
                                // Content after "- " is effectively at indent + 2
                                self.indent += 2;
                                let node = self.parse()?;
                                seq.push(node);
                            } else {
                                return self.parse_error_with_msg("unexpected '-'");
                            }
                        }
                        _ if self.indent == indent => break,
                        _ => return self.parse_error_with_msg("expected sequence item"),
                    }
                }
                self.end_context(ParseContextKind::Block)?;
                Ok(Yaml::Sequence(seq))
            }
            // TODO: Provide error message
            _ => self.parse_error_with_msg("failed to parse block sequence"),
        }
    }

    fn check_ahead_n(&self, n: usize, stop: impl Fn(u8) -> bool) -> bool {
        match self.bytes.get(self.idx + n) {
            Some(&b) => stop(b),
            None => false,
        }
    }

    fn take_while<F>(
        &mut self,
        accept: &mut F,
    ) -> std::result::Result<(usize, usize), (usize, usize)>
    where
        F: FnMut(u8, Option<u8>) -> bool,
    {
        let start = self.idx;
        let mut end = start;
        loop {
            let peeked = self.peek();
            if !accept(self.current, peeked) {
                break;
            } else if !self.bump() {
                end += 1;
                return Err((start, end));
            }
            end += 1;
        }
        Ok((start, end))
    }

    fn pop_if_match(&mut self, expect: u8) -> Result<()> {
        match self.expected.last() {
            Some(&val) if val == expect => {
                self.expected.pop();
                Ok(())
            }
            // TODO: Provide error message
            _ => self.parse_error_with_msg("token was not expected"),
        }
    }
}
