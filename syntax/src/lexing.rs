use super::{peekable3::Peekable3, Annotation, Literal};
use iref::IriRefBuf;
use locspan::{ErrAt, Loc, Location, Span};
use std::fmt;

/// Fallible tokens iterator with lookahead.
pub trait Tokens<F> {
	type Error: fmt::Debug;

	#[allow(clippy::type_complexity)]
	fn peek(&mut self) -> Result<Loc<Option<&Token>, F>, Loc<Self::Error, F>>;

	#[allow(clippy::type_complexity)]
	fn next(&mut self) -> Result<Loc<Option<Token>, F>, Loc<Self::Error, F>>;

	fn next_label(&mut self) -> Label;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(usize);

/// Identifier.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Id {
	Name(String),
	IriRef(IriRefBuf),
	Compact(String, IriRefBuf),
}

impl fmt::Display for Id {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Name(name) => name.fmt(f),
			Self::IriRef(iri) => iri.fmt(f),
			Self::Compact(prefix, suffix) => write!(f, "{}:{}", prefix, suffix),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenKind {
	Doc,
	Punct(Punct),
	Keyword(Keyword),
	Begin(Delimiter),
	End(Delimiter),
	Id,
	Literal,
}

impl TokenKind {
	pub fn matches_any(&self, list: &[Self]) -> bool {
		list.iter().any(|b| self.matches(b))
	}

	pub fn matches(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Keyword(_), Self::Id) => true,
			(a, b) => a == b,
		}
	}
}

impl fmt::Display for TokenKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Doc => write!(f, "documentation comment"),
			Self::Punct(p) => write!(f, "`{}`", p),
			Self::Keyword(k) => write!(f, "keyword `{}`", k),
			Self::Begin(d) => write!(f, "opening `{}`", d.start()),
			Self::End(d) => write!(f, "closing `{}`", d.end()),
			Self::Id => write!(f, "identifier"),
			Self::Literal => write!(f, "literal value"),
		}
	}
}

/// Syntax token.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
	/// Doc comment.
	Doc(String),

	/// Punctuation mark.
	Punct(Punct),

	/// Begin block.
	Begin(Delimiter),

	/// End block.
	End(Delimiter),

	/// Keyword.
	Keyword(Keyword),

	/// Identifier.
	Id(Id),

	/// Literal value.
	Literal(Literal),
}

impl Token {
	pub fn is_doc(&self) -> bool {
		matches!(self, Self::Doc(_))
	}

	pub fn into_doc(self) -> Option<String> {
		match self {
			Self::Doc(s) => Some(s),
			_ => None,
		}
	}

	pub fn kind(&self) -> TokenKind {
		match self {
			Self::Doc(_) => TokenKind::Doc,
			Self::Punct(p) => TokenKind::Punct(*p),
			Self::Begin(d) => TokenKind::Begin(*d),
			Self::End(d) => TokenKind::End(*d),
			Self::Keyword(k) => TokenKind::Keyword(*k),
			Self::Id(_) => TokenKind::Id,
			Self::Literal(_) => TokenKind::Literal,
		}
	}

	/// If this token is a keyword, returns the keyword as an identifier.
	/// Otherwise, return the token unchanged.
	pub fn no_keyword(self) -> Self {
		match self {
			Self::Keyword(kw) => Self::Id(Id::Name(kw.to_string())),
			token => token,
		}
	}
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Doc(_) => write!(f, "documentation"),
			Self::Punct(p) => write!(f, "punctuation mark `{}`", p),
			Self::Begin(d) => write!(f, "opening `{}`", d.start()),
			Self::End(d) => write!(f, "closing `{}`", d.end()),
			Self::Keyword(k) => write!(f, "keyword `{}`", k),
			Self::Id(id) => write!(f, "identifier `{}`", id),
			Self::Literal(Literal::String(s)) => write!(f, "string literal {}", s),
			Self::Literal(Literal::RegExp(_)) => write!(f, "regular expression"),
		}
	}
}

/// Block delimiter.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delimiter {
	Brace,
	Parenthesis,
	Bracket,
}

impl Delimiter {
	pub fn from_start(c: char) -> Option<Self> {
		match c {
			'{' => Some(Self::Brace),
			'(' => Some(Self::Parenthesis),
			'[' => Some(Self::Bracket),
			_ => None,
		}
	}

	pub fn from_end(c: char) -> Option<Self> {
		match c {
			'}' => Some(Self::Brace),
			')' => Some(Self::Parenthesis),
			']' => Some(Self::Bracket),
			_ => None,
		}
	}

	fn start(&self) -> char {
		match self {
			Self::Brace => '{',
			Self::Parenthesis => '(',
			Self::Bracket => '[',
		}
	}

	fn end(&self) -> char {
		match self {
			Self::Brace => '}',
			Self::Parenthesis => ')',
			Self::Bracket => ']',
		}
	}
}

/// Punctuation mark.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Punct {
	Comma,
	Colon,
	Ampersand,
	Pipe,
	Equal,
	Underscore,
}

impl Punct {
	pub fn from_char(c: char) -> Option<Self> {
		match c {
			',' => Some(Self::Comma),
			':' => Some(Self::Colon),
			'&' => Some(Self::Ampersand),
			'|' => Some(Self::Pipe),
			'=' => Some(Self::Equal),
			_ => None,
		}
	}
}

impl fmt::Display for Punct {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Comma => write!(f, ","),
			Self::Colon => write!(f, ":"),
			Self::Ampersand => write!(f, "&"),
			Self::Pipe => write!(f, "|"),
			Self::Equal => write!(f, "="),
			Self::Underscore => write!(f, "_"),
		}
	}
}

/// Keyword.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Keyword {
	Base,
	Use,
	Type,
	Layout,
	As,
	For,
	Annotation(Annotation),
}

impl Keyword {
	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"base" => Some(Keyword::Base),
			"use" => Some(Keyword::Use),
			"type" => Some(Keyword::Type),
			"layout" => Some(Keyword::Layout),
			"as" => Some(Keyword::As),
			"for" => Some(Keyword::For),
			_ => Annotation::from_name(name).map(Self::Annotation),
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Base => "base",
			Self::Use => "use",
			Self::Type => "type",
			Self::Layout => "layout",
			Self::As => "as",
			Self::For => "for",
			Self::Annotation(a) => a.as_str(),
		}
	}
}

impl fmt::Display for Keyword {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

/// Lexing error.
#[derive(Debug)]
pub enum Error<E> {
	InvalidId(String),
	InvalidCodepoint(u32),
	InvalidSuffix(String),
	Unexpected(Option<char>),
	Stream(E),
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidId(id) => write!(f, "invalid identifier `{}`", id),
			Self::InvalidCodepoint(c) => write!(f, "invalid character codepoint {:x}", c),
			Self::InvalidSuffix(s) => write!(f, "invalid compact IRI suffix `{}`", s),
			Self::Unexpected(None) => write!(f, "unexpected end of text"),
			Self::Unexpected(Some(c)) => write!(f, "unexpected character `{}`", c),
			Self::Stream(e) => e.fmt(f),
		}
	}
}

impl<E> From<E> for Error<E> {
	fn from(e: E) -> Self {
		Self::Stream(e)
	}
}

/// Characters iterator.
struct Chars<C: Iterator>(Peekable3<C>);

impl<E, C: Iterator<Item = Result<char, E>>> Chars<C> {
	fn peek(&mut self) -> Result<Option<char>, Error<E>> {
		match self.0.peek() {
			None => Ok(None),
			Some(Ok(c)) => Ok(Some(*c)),
			Some(Err(_)) => self.next(),
		}
	}

	fn peek2(&mut self) -> Result<Option<char>, Error<E>> {
		match self.0.peek2() {
			None => Ok(None),
			Some(Ok(c)) => Ok(Some(*c)),
			Some(Err(_)) => {
				self.next()?;
				self.next()
			}
		}
	}

	fn peek3(&mut self) -> Result<Option<char>, Error<E>> {
		match self.0.peek3() {
			None => Ok(None),
			Some(Ok(c)) => Ok(Some(*c)),
			Some(Err(_)) => {
				self.next()?;
				self.next()?;
				self.next()
			}
		}
	}

	fn next(&mut self) -> Result<Option<char>, Error<E>> {
		self.0.next().transpose().map_err(Error::Stream)
	}
}

struct Position<F> {
	file: F,
	span: Span,
	last_span: Span,
}

impl<F: Clone> Position<F> {
	fn current(&self) -> Location<F> {
		Location::new(self.file.clone(), self.span)
	}

	fn current_span(&self) -> Span {
		self.span
	}

	fn end(&self) -> Location<F> {
		Location::new(self.file.clone(), self.span.end())
	}

	fn last(&self) -> Location<F> {
		Location::new(self.file.clone(), self.last_span)
	}
}

/// Lexer.
///
/// Changes a character iterator into a `Token` iterator.
pub struct Lexer<F, E, C: Iterator<Item = Result<char, E>>> {
	chars: Chars<C>,
	pos: Position<F>,
	lookahead: Option<Loc<Token, F>>,
	label_count: usize,
}

pub enum PrefixedName {
	Keyword(Keyword),
	Name(String),
	CompactIri(String, IriRefBuf),
}

pub enum DocOrRegExp {
	Doc(String),
	RegExp(String),
}

impl<F: Clone, E, C: Iterator<Item = Result<char, E>>> Lexer<F, E, C> {
	pub fn new(file: F, chars: C) -> Self {
		Self {
			chars: Chars(Peekable3::new(chars)),
			pos: Position {
				file,
				span: Span::default(),
				last_span: Span::default(),
			},
			lookahead: None,
			label_count: 0,
		}
	}

	pub fn next_label(&mut self) -> Label {
		let l = self.label_count;
		self.label_count += 1;
		Label(l)
	}

	fn peek_char(&mut self) -> Result<Option<char>, Loc<Error<E>, F>> {
		self.chars.peek().err_at(|| self.pos.end())
	}

	fn peek_char2(&mut self) -> Result<Option<char>, Loc<Error<E>, F>> {
		let offset = self.peek_char()?.map(char::len_utf8).unwrap_or(0);
		self.chars
			.peek2()
			.err_at(|| Location::new(self.pos.file.clone(), self.pos.span.end() + offset))
	}

	fn peek_char3(&mut self) -> Result<Option<char>, Loc<Error<E>, F>> {
		let offset = self.peek_char()?.map(char::len_utf8).unwrap_or(0)
			+ self.peek_char2()?.map(char::len_utf8).unwrap_or(0);
		self.chars
			.peek3()
			.err_at(|| Location::new(self.pos.file.clone(), self.pos.span.end() + offset))
	}

	fn next_char(&mut self) -> Result<Option<char>, Loc<Error<E>, F>> {
		match self.chars.next().err_at(|| self.pos.end())? {
			Some(c) => {
				self.pos.span.push(c.len_utf8());
				self.pos.last_span.clear();
				self.pos.last_span.push(c.len_utf8());
				Ok(Some(c))
			}
			None => Ok(None),
		}
	}

	fn expect_char(&mut self) -> Result<char, Loc<Error<E>, F>> {
		self.next_char()?
			.ok_or_else(|| Loc(Error::Unexpected(None), self.pos.end()))
	}

	fn skip_whitespaces(&mut self) -> Result<(), Loc<Error<E>, F>> {
		while let Some(c) = self.peek_char()? {
			if c.is_whitespace() {
				self.next_char()?;
			} else if c == '/' {
				// maybe a comment or regexp?
				if self.peek_char2()? == Some('/') {
					// definitely a comment.
					if self.peek_char3()? == Some('/') {
						// doc comment.
						break;
					}

					self.next_char()?;
					self.next_char()?;
					while let Some(c) = self.next_char()? {
						if c == '\n' {
							break;
						}
					}
				} else {
					break;
				}
			} else {
				break;
			}
		}

		self.pos.span.clear();
		Ok(())
	}

	fn next_regexp(&mut self, first: char) -> Result<String, Loc<Error<E>, F>> {
		let mut regexp = String::new();

		let first = match first {
			'\\' => self.next_escape()?,
			c => c,
		};
		regexp.push(first);

		loop {
			let c = match self.expect_char()? {
				'/' => break,
				'\\' => {
					// escape sequence.
					self.next_escape()?
				}
				c => c,
			};

			regexp.push(c)
		}

		Ok(regexp)
	}

	fn next_doc_or_regexp(&mut self) -> Result<DocOrRegExp, Loc<Error<E>, F>> {
		match self.expect_char()? {
			'/' => {
				// doc
				self.next_char()?;

				let mut doc = String::new();
				while let Some(c) = self.next_char()? {
					if c == '\n' {
						break;
					}

					doc.push(c);
				}

				Ok(DocOrRegExp::Doc(doc))
			}
			c => Ok(DocOrRegExp::RegExp(self.next_regexp(c)?)),
		}
	}

	fn next_string_literal(&mut self) -> Result<String, Loc<Error<E>, F>> {
		let mut string = String::new();

		loop {
			let c = match self.expect_char()? {
				'\"' => break,
				'\\' => {
					// escape sequence.
					self.next_escape()?
				}
				c => c,
			};

			string.push(c)
		}

		Ok(string)
	}

	fn next_hex_char(&mut self, mut span: Span, len: u8) -> Result<char, Loc<Error<E>, F>> {
		let mut codepoint = 0;

		for _ in 0..len {
			let c = self.expect_char()?;
			match c.to_digit(16) {
				Some(d) => codepoint = codepoint << 4 | d,
				None => return Err(Loc(Error::Unexpected(Some(c)), self.pos.last())),
			}
		}

		span.set_end(self.pos.current_span().end());
		match char::try_from(codepoint) {
			Ok(c) => Ok(c),
			Err(_) => Err(Loc(
				Error::InvalidCodepoint(codepoint),
				Location::new(self.pos.file.clone(), span),
			)),
		}
	}

	fn next_escape(&mut self) -> Result<char, Loc<Error<E>, F>> {
		match self.next_char()? {
			Some(
				c @ ('_' | '~' | '.' | '-' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ','
				| ';' | '=' | '/' | '?' | '#' | '@' | '%'),
			) => Ok(c),
			unexpected => Err(Loc(Error::Unexpected(unexpected), self.pos.last())),
		}
	}

	/// Parse a `PrefixedName` according to the following grammar BNF rules:
	///
	/// ```abnf
	/// PrefixedName ::= NAME | PNAME_LN
	///
	/// NAME         ::= PN_CHARS_U PN_CHARS*
	/// PNAME_NS     ::= PN_PREFIX? ':'
	/// PNAME_LN     ::= PNAME_NS PN_LOCAL
	/// ```
	fn next_prefixed_name(&mut self, first: char) -> Result<PrefixedName, Loc<Error<E>, F>> {
		// PNAME_NS or Keyword
		let mut prefix = String::new();
		match first {
			':' => (),
			c if is_pn_chars_base(c) => {
				prefix.push(c);
				let mut only_pn_chars = true;
				let mut last_is_pn_chars = true;
				loop {
					match self.peek_char()? {
						Some(c) if is_pn_chars(c) => {
							prefix.push(self.expect_char()?);
							last_is_pn_chars = true
						}
						Some('.') => {
							prefix.push(self.expect_char()?);
							last_is_pn_chars = false;
							only_pn_chars = false
						}
						Some(':') if last_is_pn_chars => {
							if self
								.peek_char2()?
								.map(|c| c.is_whitespace())
								.unwrap_or(true)
							{
								return Ok(PrefixedName::Name(prefix));
							} else {
								self.expect_char()?;
								break;
							}
						}
						unexpected => {
							return if only_pn_chars {
								match Keyword::from_name(&prefix) {
									Some(kw) => Ok(PrefixedName::Keyword(kw)),
									None => Ok(PrefixedName::Name(prefix)),
								}
							} else {
								Err(Loc(Error::Unexpected(unexpected), self.pos.end()))
							}
						}
					}
				}
			}
			'_' => {
				// name
				let mut name = String::new();
				name.push(first);
				loop {
					match self.peek_char()? {
						Some(c) if is_pn_chars(c) => {
							name.push(self.expect_char()?);
						}
						_ => {
							return match Keyword::from_name(&name) {
								Some(kw) => Ok(PrefixedName::Keyword(kw)),
								None => Ok(PrefixedName::Name(name)),
							}
						}
					}
				}
			}
			unexpected => return Err(Loc(Error::Unexpected(Some(unexpected)), self.pos.last())),
		};

		// PN_LOCAL
		let mut suffix = String::new();
		let mut suffix_span = self.pos.current_span().next();

		let c = self.expect_char()?;
		if is_pn_chars_u(c) || c.is_ascii_digit() || matches!(c, ':' | '%' | '\\') {
			let c = match c {
				'%' => {
					// percent encoded.
					self.next_hex_char(self.pos.current_span().end().into(), 2)?
				}
				'\\' => {
					// escape sequence.
					self.next_escape()?
				}
				c => c,
			};

			suffix.push(c);

			loop {
				match self.peek_char()? {
					Some(c)
						if is_pn_chars(c)
							|| c.is_ascii_digit() || matches!(c, '%' | '\\')
							|| (c == ':'
								&& !self
									.peek_char2()?
									.map(|c| c.is_whitespace())
									.unwrap_or(true)) =>
					{
						let c = match self.expect_char()? {
							'%' => {
								// percent encoded.
								self.next_hex_char(self.pos.current_span().end().into(), 2)?
							}
							'\\' => {
								// escape sequence.
								self.next_escape()?
							}
							c => c,
						};

						suffix.push(c);
					}
					_ => {
						suffix_span.set_end(self.pos.current_span().end());
						break match IriRefBuf::from_string(suffix) {
							Ok(suffix) => Ok(PrefixedName::CompactIri(prefix, suffix)),
							Err((_, invalid_suffix)) => Err(Loc(
								Error::InvalidSuffix(invalid_suffix),
								Location::new(self.pos.file.clone(), suffix_span),
							)),
						};
					}
				}
			}
		} else {
			Err(Loc(Error::Unexpected(Some(c)), self.pos.last()))
		}
	}

	fn next_iri(&mut self) -> Result<Id, Loc<Error<E>, F>> {
		let mut iri = String::new();

		while let Some(c) = self.next_char()? {
			if c == '>' {
				break;
			} else {
				iri.push(c)
			}
		}

		let iri = match iri.try_into() {
			Ok(iri_ref) => iri_ref,
			Err((_, id)) => return Err(Loc::new(Error::InvalidId(id), self.pos.current())),
		};

		Ok(Id::IriRef(iri))
	}

	fn consume(&mut self) -> Result<Loc<Option<Token>, F>, Loc<Error<E>, F>> {
		self.skip_whitespaces()?;
		match self.next_char()? {
			Some(c) => match c {
				'/' => {
					let token = match self.next_doc_or_regexp()? {
						DocOrRegExp::Doc(doc) => Token::Doc(doc),
						DocOrRegExp::RegExp(exp) => Token::Literal(Literal::RegExp(exp)),
					};

					Ok(Loc::new(Some(token), self.pos.current()))
				}
				'"' => Ok(Loc(
					Some(Token::Literal(Literal::String(self.next_string_literal()?))),
					self.pos.current(),
				)),
				'<' => Ok(Loc::new(
					Some(Token::Id(self.next_iri()?)),
					self.pos.current(),
				)),
				c => {
					if c.is_alphabetic() {
						let token = match self.next_prefixed_name(c)? {
							PrefixedName::Keyword(kw) => Token::Keyword(kw),
							PrefixedName::CompactIri(prefix, suffix) => {
								Token::Id(Id::Compact(prefix, suffix))
							}
							PrefixedName::Name(name) => Token::Id(Id::Name(name)),
						};

						Ok(Loc::new(Some(token), self.pos.current()))
					} else {
						match Delimiter::from_start(c) {
							Some(d) => Ok(Loc::new(Some(Token::Begin(d)), self.pos.current())),
							None => match Delimiter::from_end(c) {
								Some(d) => Ok(Loc::new(Some(Token::End(d)), self.pos.current())),
								None => match Punct::from_char(c) {
									Some(p) => {
										Ok(Loc::new(Some(Token::Punct(p)), self.pos.current()))
									}
									None => Err(Loc::new(
										Error::Unexpected(Some(c)),
										self.pos.current(),
									)),
								},
							},
						}
					}
				}
			},
			None => Ok(Loc::new(None, self.pos.end())),
		}
	}

	#[allow(clippy::type_complexity)]
	fn peek(&mut self) -> Result<Loc<Option<&Token>, F>, Loc<Error<E>, F>> {
		if self.lookahead.is_none() {
			if let Loc(Some(token), loc) = self.consume()? {
				self.lookahead = Some(Loc::new(token, loc));
			}
		}

		match &self.lookahead {
			Some(Loc(token, loc)) => Ok(Loc::new(Some(token), loc.clone())),
			None => Ok(Loc::new(None, self.pos.end())),
		}
	}

	fn next(&mut self) -> Result<Loc<Option<Token>, F>, Loc<Error<E>, F>> {
		match self.lookahead.take() {
			Some(Loc(token, loc)) => Ok(Loc::new(Some(token), loc)),
			None => self.consume(),
		}
	}
}

impl<F: Clone, E: fmt::Debug, C: Iterator<Item = Result<char, E>>> Tokens<F> for Lexer<F, E, C> {
	type Error = Error<E>;

	fn peek(&mut self) -> Result<Loc<Option<&Token>, F>, Loc<Error<E>, F>> {
		self.peek()
	}

	fn next(&mut self) -> Result<Loc<Option<Token>, F>, Loc<Error<E>, F>> {
		self.next()
	}

	fn next_label(&mut self) -> Label {
		self.next_label()
	}
}

impl<F: Clone, E, C: Iterator<Item = Result<char, E>>> Iterator for Lexer<F, E, C> {
	type Item = Result<Loc<Token, F>, Loc<Error<E>, F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.next() {
			Ok(Loc(Some(token), loc)) => Some(Ok(Loc::new(token, loc))),
			Ok(Loc(None, _)) => None,
			Err(e) => Some(Err(e)),
		}
	}
}

fn is_pn_chars_base(c: char) -> bool {
	matches!(c, 'A'..='Z' | 'a'..='z' | '\u{00c0}'..='\u{00d6}' | '\u{00d8}'..='\u{00f6}' | '\u{00f8}'..='\u{02ff}' | '\u{0370}'..='\u{037d}' | '\u{037f}'..='\u{1fff}' | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' | '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' | '\u{fdf0}'..='\u{fffd}' | '\u{10000}'..='\u{effff}')
}

fn is_pn_chars_u(c: char) -> bool {
	is_pn_chars_base(c) || c == '_'
}

fn is_pn_chars(c: char) -> bool {
	is_pn_chars_u(c)
		|| matches!(c, '-' | '0'..='9' | '\u{00b7}' | '\u{0300}'..='\u{036f}' | '\u{203f}'..='\u{2040}')
}
