use super::{Loc, Span};
use crate::{source, Source};
use iref::{IriRef, IriRefBuf};
use std::{fmt, iter::Peekable};

/// Identifier.
#[derive(Clone, Debug)]
pub enum Id {
	IriRef(IriRefBuf),
	Compact(String, IriRefBuf),
}

impl fmt::Display for Id {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::IriRef(iri) => iri.fmt(f),
			Self::Compact(prefix, suffix) => write!(f, "{}:{}", prefix, suffix),
		}
	}
}

#[derive(Clone, Debug)]
pub enum TokenKind {
	Block,
	Punct(Punct),
	Keyword(Keyword),
	Id
}

impl fmt::Display for TokenKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Block => write!(f, "block"),
			Self::Punct(p) => write!(f, "`{}`", p),
			Self::Keyword(k) => write!(f, "keyword `{}`", k),
			Self::Id => write!(f, "identifier")
		}
	}
}

/// Syntax token.
#[derive(Clone, Debug)]
pub enum Token {
	/// Doc comment.
	Doc(String),

	/// Punctuation mark.
	Punct(Punct),

	/// Block.
	Block(Delimiter, Vec<Loc<Token>>),

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
			_ => None
		}
	}

	pub fn into_block(self) -> Option<(Delimiter, Vec<Loc<Token>>)> {
		match self {
			Self::Block(d, tokens) => Some((d, tokens)),
			_ => None
		}
	}
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Doc(_) => write!(f, "documentation"),
			Self::Punct(p) => write!(f, "punctuation mark `{}`", p),
			Self::Block(d, _) => write!(f, "`{} {}` block", d.start(), d.end()),
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
#[derive(Clone, Copy, Debug)]
pub enum Punct {
	Comma,
	Colon
}

impl Punct {
	pub fn from_char(c: char) -> Option<Self> {
		match c {
			',' => Some(Self::Comma),
			':' => Some(Self::Colon),
			_ => None
		}
	}
}

impl fmt::Display for Punct {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Comma => write!(f, ","),
			Self::Colon => write!(f, ":"),
		}
	}
}

/// Keyword.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Keyword {
	Type,
	Layout,
	As,
	For
}

impl Keyword {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Type => "type",
			Self::Layout => "layout",
			Self::As => "as",
			Self::For => "for"
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

/// Lexer.
///
/// Changes a character iterator into a `Token` iterator.
pub struct Lexer<E, C: Iterator<Item = Result<char, E>>> {
	file: source::Id,
	chars: Peekable<C>,
	span: Span,
	delimiters: Vec<(Delimiter, Span)>
}

impl<E, C: Iterator<Item = Result<char, E>>> Lexer<E, C> {
	pub fn new(file: source::Id, chars: C) -> Self {
		Self {
			file,
			chars: chars.peekable(),
			span: Span::default(),
			delimiters: Vec::new()
		}
	}

	fn last_delimiter(&self) -> Option<Delimiter> {
		self.delimiters.last().map(|(d, _)| *d)
	}
}

fn peek_char<E, C: Iterator<Item = Result<char, E>>>(
	lexer: &mut Lexer<E, C>,
) -> Result<Option<char>, Loc<Error<E>>> {
	match lexer.chars.peek() {
		None => Ok(None),
		Some(Ok(c)) => Ok(Some(*c)),
		Some(Err(_)) => consume_char(lexer),
	}
}

fn consume_char<E, C: Iterator<Item = Result<char, E>>>(
	lexer: &mut Lexer<E, C>,
) -> Result<Option<char>, Loc<Error<E>>> {
	match lexer.chars.next() {
		None => Ok(None),
		Some(Ok(c)) => {
			lexer.span.push(c);
			Ok(Some(c))
		}
		Some(Err(e)) => {
			lexer.span.clear();
			Err(Loc::new(
				Error::Stream(e),
				Source::new(lexer.file, lexer.span),
			))
		}
	}
}

fn skip_whitespaces<E, C: Iterator<Item = Result<char, E>>>(
	lexer: &mut Lexer<E, C>,
) -> Result<Option<Loc<Token>>, Loc<Error<E>>> {
	lexer.span.clear();

	while let Some(c) = peek_char(lexer)? {
		if !c.is_whitespace() {
			if c == '/' { // maybe a comment?
				lexer.span.clear();
				consume_char(lexer)?;

				if let Some('/') = peek_char(lexer)? { // definitely a comment.
					consume_char(lexer)?;
		
					if let Some('/') = peek_char(lexer)? { // doc comment.
						consume_char(lexer)?;
						
						let mut comment = String::new();
						while let Some(c) = peek_char(lexer)? {
							if c == '\n' {
								break;
							}
		
							consume_char(lexer)?;
							comment.push(c);
						}
		
						let span = lexer.span;
						consume_char(lexer)?;
						return Ok(Some(Loc::new(Token::Doc(comment), Source::new(lexer.file, span))))
					} else {
						// Regular comment.
						while let Some(c) = peek_char(lexer)? {
							if c == '\n' {
								break;
							}

							consume_char(lexer)?;
						}
					}
				} else {
					return Err(Loc::new(Error::Unexpected(Some('/')), Source::new(lexer.file, lexer.span)))
				}
			} else {
				break
			}
		}

		consume_char(lexer)?;
	}

	lexer.span.clear();
	Ok(None)
}

fn next_id<E, C: Iterator<Item = Result<char, E>>>(
	lexer: &mut Lexer<E, C>,
) -> Result<Loc<Token>, Loc<Error<E>>> {
	let mut id = String::new();

	while let Some(c) = peek_char(lexer)? {
		if c.is_alphabetic() {
			id.push(consume_char(lexer)?.unwrap())
		} else {
			break;
		}
	}

	let span = lexer.span;
	lexer.span.clear();

	let token = match id.as_str() {
		"type" => Token::Keyword(Keyword::Type),
		"layout" => Token::Keyword(Keyword::Layout),
		"as" => Token::Keyword(Keyword::As),
		"for" => Token::Keyword(Keyword::For),
		_ => {
			// Is it a compact IRI?
			if let Some((prefix, suffix)) = id.split_once(':') {
				if !suffix.starts_with("//") {
					let suffix = match IriRef::new(suffix) {
						Ok(iri_ref) => iri_ref.to_owned(),
						Err(_) => {
							return Err(Loc::new(
								Error::InvalidId(id),
								Source::new(lexer.file, span),
							))
						}
					};

					return Ok(Loc::new(
						Token::Id(Id::Compact(prefix.to_string(), suffix)),
						Source::new(lexer.file, span),
					));
				}
			}

			let id = match id.try_into() {
				Ok(iri_ref) => iri_ref,
				Err((_, id)) => {
					return Err(Loc::new(
						Error::InvalidId(id),
						Source::new(lexer.file, span),
					))
				}
			};

			Token::Id(Id::IriRef(id))
		}
	};

	Ok(Loc::new(token, Source::new(lexer.file, span)))
}

fn next_block<E, C: Iterator<Item = Result<char, E>>>(
	lexer: &mut Lexer<E, C>,
	delimiter: Delimiter,
) -> Result<Loc<Token>, Loc<Error<E>>> {
	let mut tokens = Vec::new();
	let mut span: Span = lexer.span.start().into();

	consume_char(lexer)?; // skip the first delimiter.

	lexer.delimiters.push((delimiter, lexer.span));
	while let Some(token) = next_token(lexer)? {
		span.set_end(token.span().end());
		tokens.push(token);
	}
	lexer.delimiters.pop();

	match consume_char(lexer)? {
		Some(c) if c == delimiter.end() => {
			span.set_end(lexer.span.end());
			Ok(Loc::new(
				Token::Block(delimiter, tokens),
				Source::new(lexer.file, span),
			))
		},
		unexpected => Err(Loc::new(Error::Unexpected(unexpected), Source::new(lexer.file, lexer.span)))
	}
}

fn next_token<E, C: Iterator<Item = Result<char, E>>>(
	lexer: &mut Lexer<E, C>,
) -> Result<Option<Loc<Token>>, Loc<Error<E>>> {
	match skip_whitespaces(lexer)? {
		Some(token) => Ok(Some(token)),
		None => {
			match peek_char(lexer)? {
				Some(c) if c.is_alphabetic() => Ok(Some(next_id(lexer)?)),
				Some(c) => match Delimiter::from_start(c) {
					Some(delimiter) => next_block(lexer, delimiter).map(Option::Some),
					None => {
						match Punct::from_char(c) {
							Some(punct) => {
								consume_char(lexer)?;
								Ok(Some(Loc::new(
									Token::Punct(punct),
									Source::new(lexer.file, lexer.span),
								)))
							},
							None => {
								match Delimiter::from_end(c) {
									Some(delimiter) if lexer.last_delimiter() == Some(delimiter) => {
										Ok(None)
									},
									_ => {
										lexer.span.clear();
										consume_char(lexer)?;
										Err(Loc::new(
											Error::Unexpected(Some(c)),
											Source::new(lexer.file, lexer.span),
										))
									}
								}
							}
						}
					}
				},
				None => Ok(None),
			}
		}
	}
}

impl<E, C: Iterator<Item = Result<char, E>>> Iterator for Lexer<E, C> {
	type Item = Result<Loc<Token>, Loc<Error<E>>>;

	fn next(&mut self) -> Option<Self::Item> {
		next_token(self).transpose()
	}
}
