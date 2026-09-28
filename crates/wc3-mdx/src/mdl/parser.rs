use super::{ReadError, ReadErrorKind, Lexer, MdlRead, Span, Token, TokenKind};
use crate::FixedText;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Pull parser with one borrowed lookahead token. Copies are cheap checkpoints.
/// Failed reads may advance; use a copy explicitly when rollback is needed.
#[derive(Clone, Copy, Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Result<Token<'a>, ReadError>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            lookahead: None,
        }
    }
    pub fn source(&self) -> &'a str {
        self.lexer.source()
    }
    pub fn position(&self) -> usize {
        match self.lookahead {
            Some(Ok(token)) => token.span.start,
            Some(Err(error)) => error.span.start,
            None => self.lexer.position(),
        }
    }
    pub fn peek(&mut self) -> Result<Option<Token<'a>>, ReadError> {
        if self.lookahead.is_none() {
            self.lookahead = self.lexer.next();
        }
        self.lookahead.transpose()
    }
    pub fn next_token(&mut self) -> Result<Token<'a>, ReadError> {
        let token = self
            .peek()?
            .ok_or_else(|| self.error(ReadErrorKind::Expected("a token")))?;
        self.lookahead = None;
        Ok(token)
    }
    pub fn error(&self, kind: ReadErrorKind) -> ReadError {
        let span = match self.lookahead {
            Some(Ok(token)) => token.span,
            Some(Err(error)) => error.span,
            None => Span::new(self.position(), self.position()),
        };
        ReadError::new(span, kind)
    }
    pub fn expect(&mut self, kind: TokenKind<'a>) -> Result<Token<'a>, ReadError> {
        let expected = match kind {
            TokenKind::Ident(_) => "the named identifier",
            TokenKind::Number(_) => "the numeric literal",
            TokenKind::Quoted(_) => "the quoted string",
            TokenKind::OpenBrace => "'{'",
            TokenKind::CloseBrace => "'}'",
            TokenKind::Comma => "','",
            TokenKind::Colon => "':'",
            TokenKind::Slot => "'<='",
        };
        match self.peek()? {
            Some(token) if token.kind == kind => self.next_token(),
            _ => Err(self.error(ReadErrorKind::Expected(expected))),
        }
    }
    pub fn expect_ident(&mut self, name: &'static str) -> Result<(), ReadError> {
        match self.peek()? {
            Some(Token {
                kind: TokenKind::Ident(actual),
                ..
            }) if actual == name => {
                self.next_token()?;
                Ok(())
            }
            _ => Err(self.error(ReadErrorKind::Expected(name))),
        }
    }
    pub fn consume(&mut self, kind: TokenKind<'a>) -> Result<bool, ReadError> {
        if self.peek()?.is_some_and(|token| token.kind == kind) {
            self.next_token()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn read<T: MdlRead>(&mut self) -> Result<T, ReadError> {
        T::read_mdl(self)
    }
    pub fn read_property<T: MdlRead>(&mut self) -> Result<T, ReadError> {
        let value = self.read()?;
        self.expect(TokenKind::Comma)?;
        Ok(value)
    }
    /// Borrows a literal string. Backslashes and embedded CR/LF are unchanged.
    pub fn read_string(&mut self) -> Result<&'a str, ReadError> {
        match self.peek()? {
            Some(Token {
                kind: TokenKind::Quoted(value),
                ..
            }) => {
                self.next_token()?;
                Ok(value)
            }
            _ => Err(self.error(ReadErrorKind::Expected("a quoted string"))),
        }
    }
    /// Copies directly into inline fixed-width storage, without a temporary String.
    pub fn read_fixed_text<const N: usize>(&mut self) -> Result<FixedText<N>, ReadError> {
        self.peek()?;
        let span = self.error(ReadErrorKind::Expected("a quoted string")).span;
        let value = self.read_string()?;
        let mut text = FixedText::default();
        text.set_text(value).map_err(|_| {
            ReadError::new(
                span,
                ReadErrorKind::InvalidString {
                    max_bytes: N.saturating_sub(1),
                },
            )
        })?;
        Ok(text)
    }
    /// Enters a body at its opening brace; does not scan ahead to its end.
    pub fn begin_block(&mut self) -> Result<Block<'_, 'a>, ReadError> {
        self.expect(TokenKind::OpenBrace)?;
        Ok(Block {
            parser: self,
            ended: false,
            closing_span: None,
        })
    }
    /// Enters a count-prefixed body, yielding structured items on demand.
    /// Each item's MdlRead implementation owns its punctuation.
    pub fn counted<T: MdlRead>(&mut self) -> Result<Counted<'_, 'a, T>, ReadError> {
        let expected = self.read::<u32>()? as usize;
        self.expect(TokenKind::OpenBrace)?;
        Ok(Counted {
            parser: self,
            expected,
            actual: 0,
            ended: false,
            failure: None,
            marker: PhantomData,
        })
    }
    pub fn finish(&mut self) -> Result<(), ReadError> {
        if self.peek()?.is_none() {
            Ok(())
        } else {
            Err(self.error(ReadErrorKind::TrailingInput))
        }
    }
}

/// A block field's borrowed identifier and its source range.
/// Unlike a lexical token, a field always has an identifier name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field<'a> {
    pub name: &'a str,
    pub span: Span,
}

/// A scoped block body. Consume fields through next_field, then call finish.
/// Dropping a body does not consume or validate any remaining input.
pub struct Block<'p, 'a> {
    parser: &'p mut Parser<'a>,
    ended: bool,
    closing_span: Option<Span>,
}
impl<'a> Block<'_, 'a> {
    /// Returns the next field name, or consumes the closing brace and returns None.
    /// The caller must consume the field's value and punctuation before calling again.
    pub fn next_field(&mut self) -> Result<Option<Field<'a>>, ReadError> {
        if self.ended {
            return Ok(None);
        }
        if let Some(token) = self.parser.peek()? {
            if token.kind == TokenKind::CloseBrace {
                self.parser.next_token()?;
                self.closing_span = Some(token.span);
                self.ended = true;
                return Ok(None);
            }
        }
        match self.parser.peek()? {
            Some(Token {
                kind: TokenKind::Ident(name),
                span,
            }) => {
                self.parser.next_token()?;
                Ok(Some(Field { name, span }))
            }
            _ => Err(self
                .parser
                .error(ReadErrorKind::Expected("a field name or '}'"))),
        }
    }
    /// Reports an error at the closing brace when this body has ended.
    pub fn error(&self, kind: ReadErrorKind) -> ReadError {
        match self.closing_span {
            Some(span) => ReadError::new(span, kind),
            None => self.parser.error(kind),
        }
    }
    /// Rejects an unread field rather than silently skipping it.
    pub fn finish(mut self) -> Result<(), ReadError> {
        if !self.ended {
            self.parser.expect(TokenKind::CloseBrace)?;
            self.ended = true;
        }
        Ok(())
    }
}
impl<'a> Deref for Block<'_, 'a> {
    type Target = Parser<'a>;
    fn deref(&self) -> &Self::Target {
        self.parser
    }
}
impl DerefMut for Block<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.parser
    }
}

/// A fallible streaming list. Exhaustion validates the declared count and closing
/// brace; callers that stop early must call finish, which drains and validates.
pub struct Counted<'p, 'a, T> {
    parser: &'p mut Parser<'a>,
    expected: usize,
    actual: usize,
    ended: bool,
    failure: Option<ReadError>,
    marker: PhantomData<T>,
}
impl<T: MdlRead> Counted<'_, '_, T> {
    pub fn declared_count(&self) -> usize {
        self.expected
    }
    pub fn finish(mut self) -> Result<(), ReadError> {
        for value in self.by_ref() {
            value?;
        }
        self.failure.map_or(Ok(()), Err)
    }
    fn read_next(&mut self) -> Result<Option<T>, ReadError> {
        let token = self
            .parser
            .peek()?
            .ok_or_else(|| self.parser.error(ReadErrorKind::Expected("'}'")))?;
        if token.kind == TokenKind::CloseBrace {
            self.parser.next_token()?;
            self.ended = true;
            if self.actual != self.expected {
                return Err(ReadError::new(
                    token.span,
                    ReadErrorKind::CountMismatch {
                        expected: self.expected,
                        actual: self.actual,
                    },
                ));
            }
            return Ok(None);
        }
        if self.actual == self.expected {
            return Err(self.parser.error(ReadErrorKind::CountMismatch {
                expected: self.expected,
                actual: self.actual + 1,
            }));
        }
        let start = self.parser.position();
        let value = self.parser.read()?;
        if self.parser.position() == start {
            return Err(self.parser.error(ReadErrorKind::NoProgress));
        }
        self.actual += 1;
        Ok(Some(value))
    }
}
impl<T: MdlRead> Iterator for Counted<'_, '_, T> {
    type Item = Result<T, ReadError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }
        match self.read_next() {
            Ok(value) => value.map(Ok),
            Err(error) => {
                self.ended = true;
                self.failure = Some(error);
                Some(Err(error))
            }
        }
    }
}

impl<T: MdlRead> FusedIterator for Counted<'_, '_, T> {}

macro_rules! integers {
    ($($ty:ty),*) => { $(impl MdlRead for $ty {
        fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError> {
            let token = parser.next_token()?;
            match token.kind {
                TokenKind::Number(raw) => raw.parse().map_err(|_| ReadError::new(token.span, ReadErrorKind::InvalidNumber(stringify!($ty)))),
                _ => Err(ReadError::new(token.span, ReadErrorKind::Expected(stringify!($ty)))),
            }
        }
    })* };
}
integers!(u8, u16, u32, i32);
impl MdlRead for f32 {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError> {
        let token = parser.next_token()?;
        let raw = match token.kind {
            TokenKind::Number(raw) | TokenKind::Ident(raw) => raw,
            _ => return Err(ReadError::new(token.span, ReadErrorKind::Expected("f32"))),
        };
        let value = if raw.eq_ignore_ascii_case("nan") {
            f32::NAN
        } else if raw.eq_ignore_ascii_case("inf") || raw.eq_ignore_ascii_case("+inf") {
            f32::INFINITY
        } else if raw.eq_ignore_ascii_case("-inf") {
            f32::NEG_INFINITY
        } else {
            // Do not accept Rust's additional spellings or overflow to infinity.
            if !matches!(token.kind, TokenKind::Number(_)) {
                return Err(ReadError::new(token.span, ReadErrorKind::InvalidNumber("f32")));
            }
            let value: f32 = raw
                .parse()
                .map_err(|_| ReadError::new(token.span, ReadErrorKind::InvalidNumber("f32")))?;
            if !value.is_finite() {
                return Err(ReadError::new(token.span, ReadErrorKind::InvalidNumber("f32")));
            }
            value
        };
        Ok(value)
    }
}
impl<T: MdlRead + Default + Copy, const N: usize> MdlRead for [T; N] {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError> {
        parser.expect(TokenKind::OpenBrace)?;
        let mut values = [T::default(); N];
        for (i, value) in values.iter_mut().enumerate() {
            if i != 0 {
                parser.expect(TokenKind::Comma)?;
            }
            *value = parser.read()?;
        }
        parser.expect(TokenKind::CloseBrace)?;
        Ok(values)
    }
}
impl<const N: usize> MdlRead for FixedText<N> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError> {
        parser.read_fixed_text()
    }
}
