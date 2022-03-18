use super::{peekable3::Peekable3, Annotation};
use iref::{IriRef, IriRefBuf};
use locspan::{ErrAt, Loc, Location, Span};
use std::fmt;

/// Fallible tokens iterator with lookahead.
pub trait Tokens<F> {
	type Error: fmt::Debug;

	#[allow(clippy::type_complexity)]
	fn peek(&mut self) -> Result<Loc<Option<&Token>, F>, Loc<Self::Error, F>>;

	#[allow(clippy::type_complexity)]
	fn next(&mut self) -> Result<Loc<Option<Token>, F>, Loc<Self::Error, F>>;
}

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
}

impl Punct {
	pub fn from_char(c: char) -> Option<Self> {
		match c {
			',' => Some(Self::Comma),
			':' => Some(Self::Colon),
			'&' => Some(Self::Ampersand),
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
	Unexpected(Option<char>),
	Stream(E),
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidId(id) => write!(f, "invalid identifier `{}`", id),
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
}

impl<F: Clone> Position<F> {
	fn current(&self) -> Location<F> {
		Location::new(self.file.clone(), self.span)
	}

	fn end(&self) -> Location<F> {
		Location::new(self.file.clone(), self.span.end())
	}
}

/// Lexer.
///
/// Changes a character iterator into a `Token` iterator.
pub struct Lexer<F, E, C: Iterator<Item = Result<char, E>>> {
	chars: Chars<C>,
	pos: Position<F>,
	lookahead: Option<Loc<Token, F>>,
}

impl<F: Clone, E, C: Iterator<Item = Result<char, E>>> Lexer<F, E, C> {
	pub fn new(file: F, chars: C) -> Self {
		Self {
			chars: Chars(Peekable3::new(chars)),
			pos: Position {
				file,
				span: Span::default(),
			},
			lookahead: None,
		}
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
				Ok(Some(c))
			}
			None => Ok(None),
		}
	}

	fn skip_whitespaces(&mut self) -> Result<(), Loc<Error<E>, F>> {
		while let Some(c) = self.peek_char()? {
			if c.is_whitespace() {
				self.next_char()?;
			} else if c == '/' {
				// maybe a comment?
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

	fn next_doc(&mut self) -> Result<String, Loc<Error<E>, F>> {
		self.next_char()?;
		self.next_char()?;

		let mut doc = String::new();
		while let Some(c) = self.next_char()? {
			if c == '\n' {
				break;
			}

			doc.push(c);
		}

		Ok(doc)
	}

	fn next_name(&mut self, first: char) -> Result<String, Loc<Error<E>, F>> {
		let mut id = String::new();
		id.push(first);

		while let Some(c) = self.peek_char()? {
			if c.is_alphanumeric() {
				id.push(self.next_char()?.unwrap())
			} else {
				break;
			}
		}

		Ok(id)
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

		// Is it a compact IRI?
		if let Some((prefix, suffix)) = iri.split_once(':') {
			if !suffix.starts_with("//") {
				let suffix = match IriRef::new(suffix) {
					Ok(iri_ref) => iri_ref.to_owned(),
					Err(_) => return Err(Loc::new(Error::InvalidId(iri), self.pos.current())),
				};

				return Ok(Id::Compact(prefix.to_string(), suffix));
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
				'/' => Ok(Loc::new(
					Some(Token::Doc(self.next_doc()?)),
					self.pos.current(),
				)),
				'<' => Ok(Loc::new(
					Some(Token::Id(self.next_iri()?)),
					self.pos.current(),
				)),
				c => {
					if c.is_alphabetic() {
						let name = self.next_name(c)?;
						match Keyword::from_name(&name) {
							Some(kw) => Ok(Loc::new(Some(Token::Keyword(kw)), self.pos.current())),
							None => Ok(Loc::new(
								Some(Token::Id(Id::Name(name))),
								self.pos.current(),
							)),
						}
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
