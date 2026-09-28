use super::{ReadError, ReadErrorKind, Span};
use std::iter::FusedIterator;

/// A borrowed lexical token. String contents are not unescaped by the lexer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind<'a> {
    Ident(&'a str),
    Number(&'a str),
    Quoted(&'a str),
    OpenBrace,
    CloseBrace,
    Comma,
    Colon,
    Slot,
}

/// A token and its byte range, including quotes for a quoted token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

/// Allocation-free token iterator over resident UTF-8 text.
///
/// Skips whitespace, a leading BOM and line comments. An error
/// terminates iteration. Numeric spelling is validated by the typed reader.
#[derive(Clone, Copy, Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    failed: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: if source.starts_with('\u{feff}') { 3 } else { 0 },
            failed: false,
        }
    }

    pub fn position(&self) -> usize {
        self.offset
    }
    pub fn source(&self) -> &'a str {
        self.source
    }

    fn scan(&mut self) -> Result<Option<Token<'a>>, ReadError> {
        let bytes = self.source.as_bytes();
        loop {
            while bytes.get(self.offset).is_some_and(u8::is_ascii_whitespace) {
                self.offset += 1;
            }
            if bytes.get(self.offset..self.offset + 2) == Some(b"//") {
                while bytes.get(self.offset).is_some_and(|b| *b != b'\n') {
                    self.offset += 1;
                }
            } else {
                break;
            }
        }
        let start = self.offset;
        let Some(&byte) = bytes.get(start) else {
            return Ok(None);
        };
        self.offset += 1;
        let kind = match byte {
            b'{' => TokenKind::OpenBrace,
            b'}' => TokenKind::CloseBrace,
            b',' => TokenKind::Comma,
            b':' => TokenKind::Colon,
            b'<' if bytes.get(self.offset) == Some(&b'=') => {
                self.offset += 1;
                TokenKind::Slot
            }
            b'"' => {
                let content = self.offset;
                while let Some(&byte) = bytes.get(self.offset) {
                    if byte == b'"' {
                        let raw = &self.source[content..self.offset];
                        self.offset += 1;
                        return Ok(Some(Token {
                            kind: TokenKind::Quoted(raw),
                            span: Span::new(start, self.offset),
                        }));
                    }
                    self.offset += 1;
                }
                return Err(ReadError::new(
                    Span::new(start, bytes.len()),
                    ReadErrorKind::UnterminatedString,
                ));
            }
            b if b.is_ascii_alphabetic() || b == b'_' => {
                while bytes
                    .get(self.offset)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                {
                    self.offset += 1;
                }
                let raw = &self.source[start..self.offset];
                if raw.eq_ignore_ascii_case("nan") || raw.eq_ignore_ascii_case("inf") {
                    TokenKind::Number(raw)
                } else {
                    TokenKind::Ident(raw)
                }
            }
            b if b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.') => {
                while bytes
                    .get(self.offset)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
                {
                    self.offset += 1;
                }
                TokenKind::Number(&self.source[start..self.offset])
            }
            _ => {
                self.offset = start + self.source[start..].chars().next().unwrap().len_utf8();
                return Err(ReadError::new(
                    Span::new(start, self.offset),
                    ReadErrorKind::InvalidCharacter,
                ));
            }
        };
        Ok(Some(Token {
            kind,
            span: Span::new(start, self.offset),
        }))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, ReadError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        match self.scan() {
            Ok(token) => token.map(Ok),
            Err(error) => {
                self.failed = true;
                Some(Err(error))
            }
        }
    }
}

impl FusedIterator for Lexer<'_> {}
