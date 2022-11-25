use super::*;
use decoded_char::DecodedChar;
use lexing::{Delimiter, Keyword, Punct, Token, TokenKind, Tokens};
use locspan::{MaybeLocated, MaybeSpanned, Meta, Span};
use std::{fmt, fmt::Debug};
use treeldr::reporting;

pub type MetaError<E, M> = Meta<Box<Error<E>>, M>;

/// Parse error.
#[derive(Debug)]
pub enum Error<E> {
	Unexpected(Option<Token>, Vec<lexing::TokenKind>),
	InvalidUseId(Id),
	InvalidPrefix(Id),
	InvalidAlias(Id),
	Lexer(E),
}

impl<E: Debug + fmt::Display, M: MaybeLocated<Span = Span>> reporting::DiagnoseWithMetadata<M>
	for Error<E>
where
	M::File: Clone,
{
	fn message(&self, _cause: &M) -> String {
		match self {
			Self::Unexpected(_, _) => "parsing error".to_owned(),
			Self::InvalidUseId(_) => "invalid use IRI".to_owned(),
			Self::InvalidPrefix(_) => "invalid prefix".to_owned(),
			Self::InvalidAlias(_) => "invalid alias".to_owned(),
			Self::Lexer(_) => "lexing error".to_owned(),
		}
	}

	fn labels(&self, cause: &M) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		match cause.optional_location() {
			Some(loc) => {
				vec![codespan_reporting::diagnostic::Label::primary(
					loc.file().clone(),
					loc.optional_span().unwrap(),
				)
				.with_message(self.to_string())]
			}
			None => Vec::new(),
		}
	}

	fn notes(&self, _cause: &M) -> Vec<String> {
		if let Error::Unexpected(_, expected) = self {
			if !expected.is_empty() {
				let mut note = "expected ".to_owned();

				let len = expected.len();
				for (i, token) in expected.iter().enumerate() {
					if i > 0 {
						if i + 1 == len {
							note.push_str(" or ");
						} else {
							note.push_str(", ");
						}
					}

					note.push_str(&token.to_string())
				}

				return vec![note];
			}
		}

		Vec::new()
	}
}

impl<E: Debug + fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Unexpected(None, _) => write!(f, "unexpected end of text"),
			Self::Unexpected(Some(token), _) => write!(f, "unexpected {}", token),
			Self::InvalidUseId(id) => write!(f, "invalid use IRI `{}`", id),
			Self::InvalidPrefix(id) => write!(f, "invalid prefix `{}`", id),
			Self::InvalidAlias(id) => write!(f, "invalid alias `{}`", id),
			Self::Lexer(e) => write!(f, "tokens error: {}", e),
		}
	}
}

/// TreeLDR parser.
pub struct Parser<L, F> {
	lexer: L,
	metadata_builder: F,
}

impl<L: Tokens, F> Parser<L, F> {
	pub fn new(lexer: L, metadata_builder: F) -> Self {
		Self {
			lexer,
			metadata_builder,
		}
	}

	fn next<M>(&mut self) -> Result<Meta<Option<Token>, Span>, MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		self.lexer
			.next()
			.map_err(|Meta(e, span)| Meta(Box::new(Error::Lexer(e)), (self.metadata_builder)(span)))
	}

	fn next_label(&mut self) -> Label {
		self.lexer.next_label()
	}

	#[allow(clippy::type_complexity)]
	fn peek<M>(&mut self) -> Result<Meta<Option<&Token>, Span>, MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		self.lexer
			.peek()
			.map_err(|Meta(e, span)| Meta(Box::new(Error::Lexer(e)), (self.metadata_builder)(span)))
	}

	fn build_metadata<M>(&mut self, span: Span) -> M
	where
		F: FnMut(Span) -> M,
	{
		(self.metadata_builder)(span)
	}

	#[allow(clippy::type_complexity)]
	fn next_expected<M>(
		&mut self,
		expected: impl FnOnce() -> Vec<TokenKind>,
	) -> Result<Meta<Token, Span>, MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		match self.next()? {
			Meta(None, span) => Err(Meta::new(
				Box::new(Error::Unexpected(None, expected())),
				(self.metadata_builder)(span),
			)),
			Meta(Some(token), span) => Ok(Meta::new(token, span)),
		}
	}

	#[allow(clippy::type_complexity)]
	fn next_expected_token_from<M>(
		&mut self,
		token_opt: Option<Meta<Token, Span>>,
		expected: impl FnOnce() -> Vec<TokenKind>,
	) -> Result<Meta<Token, Span>, MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		match token_opt {
			Some(token) => Ok(token),
			None => self.next_expected(expected),
		}
	}

	#[allow(clippy::type_complexity)]
	fn parse_comma_separated_list<T: Parse<M>, M>(
		&mut self,
	) -> Result<Vec<Meta<T, M>>, MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		let mut list = Vec::new();

		while let Some(item) = T::try_parse(self)? {
			list.push(item);
			match self.peek()? {
				Meta(Some(Token::Punct(Punct::Comma)), _) => {
					self.next()?;
					// continue
				}
				_ => break,
			}
		}

		Ok(list)
	}

	#[allow(clippy::type_complexity)]
	fn parse_block<T: Parse<M>, M>(
		&mut self,
		mut span: Span,
	) -> Result<Meta<Vec<Meta<T, M>>, M>, MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		let items = self.parse_comma_separated_list()?;
		match self.next()? {
			Meta(Some(Token::End(Delimiter::Brace)), end_span) => {
				span.append(end_span);
				Ok(Meta(items, self.build_metadata(span)))
			}
			Meta(unexpected, span) => Err(Meta::new(
				Box::new(Error::Unexpected(
					unexpected,
					vec![
						TokenKind::Punct(Punct::Comma),
						TokenKind::End(Delimiter::Brace),
					],
				)),
				self.build_metadata(span),
			)),
		}
	}

	fn parse_keyword<M>(&mut self, keyword: lexing::Keyword) -> Result<(), MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		let Meta(token, span) = self.next_expected(|| vec![TokenKind::Keyword(keyword)])?;

		match token {
			Token::Keyword(k) if k == keyword => Ok(()),
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(
					Some(unexpected),
					vec![TokenKind::Keyword(keyword)],
				)),
				self.build_metadata(span),
			)),
		}
	}

	fn parse_punct<M>(&mut self, punct: lexing::Punct) -> Result<(), MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		let Meta(token, span) = self.next_expected(|| vec![TokenKind::Punct(punct)])?;

		match token {
			Token::Punct(p) if p == punct => Ok(()),
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(
					Some(unexpected),
					vec![TokenKind::Punct(punct)],
				)),
				self.build_metadata(span),
			)),
		}
	}

	fn parse_end<M>(&mut self, delimiter: Delimiter) -> Result<Span, MetaError<L::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		let Meta(token, span) = self.next_expected(|| vec![TokenKind::End(delimiter)])?;

		match token {
			Token::End(d) if d == delimiter => Ok(span),
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(
					Some(unexpected),
					vec![TokenKind::End(delimiter)],
				)),
				self.build_metadata(span),
			)),
		}
	}
}

/// Parsable abstract syntax nodes.
pub trait Parse<M>: Sized {
	const FIRST: &'static [TokenKind];

	#[inline(always)]
	fn parse_str<F>(
		content: &str,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, MetaError<lexing::Error, M>>
	where
		F: FnMut(Span) -> M,
	{
		Self::parse_utf8_infallible(content.chars(), metadata_builder)
	}

	#[inline(always)]
	fn parse_utf8_infallible<C, F>(
		chars: C,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, MetaError<lexing::Error, M>>
	where
		C: Iterator<Item = char>,
		F: FnMut(Span) -> M,
	{
		Self::parse_infallible(decoded_char::Utf8Decoded(chars), metadata_builder)
	}

	#[inline(always)]
	fn parse_utf8<C, F, E>(
		chars: C,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, MetaError<lexing::Error<E>, M>>
	where
		C: Iterator<Item = Result<char, E>>,
		F: FnMut(Span) -> M,
	{
		Self::parse(decoded_char::FallibleUtf8Decoded(chars), metadata_builder)
	}

	#[inline(always)]
	fn parse_infallible<C, F>(
		chars: C,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, MetaError<lexing::Error, M>>
	where
		C: Iterator<Item = DecodedChar>,
		F: FnMut(Span) -> M,
	{
		let lexer = Lexer::new(chars.map(Ok));
		let mut parser = Parser::new(lexer, metadata_builder);
		Self::parse_in(&mut parser)
	}

	#[inline(always)]
	fn parse<C, F, E>(
		chars: C,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, MetaError<lexing::Error<E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		let lexer = Lexer::new(chars);
		let mut parser = Parser::new(lexer, metadata_builder);
		Self::parse_in(&mut parser)
	}

	#[allow(clippy::type_complexity)]
	fn parse_in<L, F>(parser: &mut Parser<L, F>) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match parser.next()? {
			Meta(Some(token), loc) => Self::parse_from(parser, token, loc),
			Meta(None, loc) => Err(Meta(
				Box::new(Error::Unexpected(None, Self::FIRST.to_vec())),
				parser.build_metadata(loc),
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	fn parse_from_continuation<L, F>(
		parser: &mut Parser<L, F>,
		token_opt: Option<Meta<Token, Span>>,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token_opt {
			Some(Meta(token, loc)) => Self::parse_from(parser, token, loc),
			None => Self::parse_in(parser),
		}
	}

	#[allow(clippy::type_complexity)]
	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M;

	#[allow(clippy::type_complexity)]
	fn try_parse<L, F>(
		parser: &mut Parser<L, F>,
	) -> Result<Option<Meta<Self, M>>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match parser.peek()? {
			Meta(Some(token), _) => {
				if token.kind().matches_any(Self::FIRST) {
					Ok(Some(Self::parse_in(parser)?))
				} else {
					Ok(None)
				}
			}
			_ => Ok(None),
		}
	}

	#[allow(clippy::type_complexity)]
	fn try_parse_from_continuation<L, F>(
		parser: &mut Parser<L, F>,
		token_opt: Option<Meta<Token, Span>>,
	) -> Result<Option<Meta<Self, M>>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token_opt {
			Some(Meta(token, loc)) => {
				if token.kind().matches_any(Self::FIRST) {
					Ok(Some(Self::parse_from(parser, token, loc)?))
				} else {
					Ok(None)
				}
			}
			None => Self::try_parse(parser),
		}
	}

	#[allow(clippy::type_complexity)]
	fn try_parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<(Option<Meta<Self, M>>, Option<Meta<Token, Span>>), MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		if token.kind().matches_any(Self::FIRST) {
			Ok((Some(Self::parse_from(parser, token, loc)?), None))
		} else {
			Ok((None, Some(Meta(token, loc))))
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for Document<M> {
	const FIRST: &'static [TokenKind] = Item::<M>::FIRST;

	fn parse_in<L, F>(parser: &mut Parser<L, F>) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match parser.next()? {
			Meta(Some(token), span) => Self::parse_from(parser, token, span),
			Meta(None, span) => Ok(Meta::new(
				Self {
					bases: Vec::new(),
					uses: Vec::new(),
					types: Vec::new(),
					properties: Vec::new(),
					layouts: Vec::new(),
				},
				parser.build_metadata(span),
			)),
		}
	}

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let mut bases = Vec::new();
		let mut uses = Vec::new();
		let mut types = Vec::new();
		let mut properties = Vec::new();
		let mut layouts = Vec::new();

		let Meta(first_item, first_meta) = Item::parse_from(parser, token, loc)?;
		let mut loc = first_meta.optional_span().unwrap();
		match first_item {
			Item::Base(i) => bases.push(i),
			Item::Use(i) => uses.push(i),
			Item::Type(t) => types.push(t),
			Item::Property(p) => properties.push(p),
			Item::Layout(l) => layouts.push(l),
		}

		loop {
			match parser.peek()? {
				Meta(Some(_), _) => {
					let Meta(item, item_meta) = Item::parse_in(parser)?;
					loc.append(item_meta.optional_span().unwrap());

					match item {
						Item::Base(i) => bases.push(i),
						Item::Use(i) => uses.push(i),
						Item::Type(t) => types.push(t),
						Item::Property(p) => properties.push(p),
						Item::Layout(l) => layouts.push(l),
					}
				}
				Meta(None, _) => {
					break Ok(Meta::new(
						Self {
							bases,
							uses,
							types,
							properties,
							layouts,
						},
						parser.build_metadata(loc),
					));
				}
			}
		}
	}
}

impl<M> Parse<M> for Id {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		span: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token {
			Token::Id(id) => Ok(Meta::new(id, parser.build_metadata(span))),
			Token::Keyword(kw) => Ok(Meta::new(
				Id::Name(kw.to_string()),
				parser.build_metadata(span),
			)),
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id])),
				parser.build_metadata(span),
			)),
		}
	}
}

impl<M> Parse<M> for Documentation<M> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Doc];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		mut span: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let mut items = Vec::new();
		match token {
			Token::Doc(doc) => {
				items.push(Meta(doc, parser.build_metadata(span)));
				loop {
					match parser.peek()? {
						Meta(Some(token), _) if token.is_doc() => {
							let Meta(doc, doc_span) =
								parser.next()?.unwrap().map(|t| t.into_doc().unwrap());
							span.append(doc_span);
							items.push(Meta(doc, parser.build_metadata(doc_span)))
						}
						_ => break Ok(Meta::new(Self::new(items), parser.build_metadata(span))),
					}
				}
			}
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id])),
				parser.build_metadata(span),
			)),
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for Item<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Doc,
		TokenKind::Keyword(lexing::Keyword::Base),
		TokenKind::Keyword(lexing::Keyword::Type),
		TokenKind::Keyword(lexing::Keyword::Property),
		TokenKind::Keyword(lexing::Keyword::Layout),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		token_span: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let (doc, k) = Documentation::try_parse_from(parser, token, token_span)?;

		let Meta(token, mut loc) = parser.next_expected_token_from(k, || Self::FIRST.to_vec())?;

		match token {
			Token::Keyword(lexing::Keyword::Base) => {
				let id = Id::parse_in(parser)?;
				let iri = match id {
					Meta(Id::IriRef(iri_ref), loc) => match IriBuf::try_from(iri_ref) {
						Ok(iri) => Meta::new(iri, loc),
						Err(iri_ref) => {
							return Err(Meta::new(
								Box::new(Error::InvalidUseId(Id::IriRef(iri_ref))),
								loc,
							))
						}
					},
					Meta(id, loc) => return Err(Meta::new(Box::new(Error::InvalidUseId(id)), loc)),
				};

				parser.parse_punct(Punct::Semicolon)?;

				Ok(Meta::new(Item::Base(iri), parser.build_metadata(loc)))
			}
			Token::Keyword(lexing::Keyword::Use) => {
				let id = Id::parse_in(parser)?;
				let iri = match id {
					Meta(Id::IriRef(iri_ref), loc) => match IriBuf::try_from(iri_ref) {
						Ok(iri) => Meta::new(iri, loc),
						Err(iri_ref) => {
							return Err(Meta::new(
								Box::new(Error::InvalidUseId(Id::IriRef(iri_ref))),
								loc,
							))
						}
					},
					Meta(id, loc) => return Err(Meta::new(Box::new(Error::InvalidUseId(id)), loc)),
				};

				parser.parse_keyword(lexing::Keyword::As)?;

				let prefix = Prefix::parse_in(parser)?;
				loc.append(prefix.metadata().optional_span().unwrap());

				parser.parse_punct(Punct::Semicolon)?;

				Ok(Meta::new(
					Item::Use(Meta::new(
						Use { iri, prefix, doc },
						parser.build_metadata(loc),
					)),
					parser.build_metadata(loc),
				))
			}
			Token::Keyword(lexing::Keyword::Type) => {
				let id = Id::parse_in(parser)?;

				let (description, layout) = match parser.next()? {
					Meta(Some(Token::Begin(Delimiter::Brace)), block_loc) => {
						let Meta(properties, properties_loc) = parser.parse_block(block_loc)?;

						let layout = match parser.peek()?.value() {
							Some(Token::Keyword(Keyword::With)) => {
								parser.next()?;
								let layout = LayoutDescription::parse_in(parser)?;
								loc.append(layout.metadata().optional_span().unwrap());
								Some(layout)
							}
							_ => None,
						};

						(
							Meta(TypeDescription::Normal(properties), properties_loc),
							layout,
						)
					}
					Meta(Some(Token::Punct(Punct::Equal)), _) => {
						let Meta(expr, expr_loc) = OuterTypeExpr::parse_in(parser)?;

						let layout = match parser.peek()?.value() {
							Some(Token::Keyword(Keyword::With)) => {
								parser.next()?;
								let layout = LayoutDescription::parse_in(parser)?;
								loc.append(layout.metadata().optional_span().unwrap());
								Some(layout)
							}
							_ => {
								parser.parse_punct(Punct::Semicolon)?;
								None
							}
						};

						(Meta(TypeDescription::Alias(expr), expr_loc), layout)
					}
					Meta(Some(Token::Punct(Punct::Semicolon)), loc) => (
						Meta(
							TypeDescription::Normal(Vec::new()),
							parser.build_metadata(loc),
						),
						None,
					),
					Meta(Some(Token::Keyword(Keyword::With)), _) => {
						let layout = LayoutDescription::parse_in(parser)?;
						loc.append(layout.metadata().optional_span().unwrap());
						(
							Meta(
								TypeDescription::Normal(Vec::new()),
								parser.build_metadata(loc),
							),
							Some(layout),
						)
					}
					Meta(unexpected, loc) => {
						return Err(Meta::new(
							Box::new(Error::Unexpected(
								unexpected,
								vec![
									TokenKind::Begin(Delimiter::Brace),
									TokenKind::Punct(Punct::Equal),
								],
							)),
							parser.build_metadata(loc),
						));
					}
				};

				loc.append(description.metadata().optional_span().unwrap());
				Ok(Meta::new(
					Item::Type(Meta::new(
						TypeDefinition {
							id,
							description,
							doc,
							layout,
						},
						parser.build_metadata(loc),
					)),
					parser.build_metadata(loc),
				))
			}
			Token::Keyword(lexing::Keyword::Property) => {
				let id = Id::parse_in(parser)?;

				let ty = match parser.next()? {
					Meta(Some(Token::Punct(Punct::Colon)), _) => {
						let ty = AnnotatedTypeExpr::parse_in(parser)?;
						loc.append(ty.metadata().optional_span().unwrap());
						parser.parse_punct(Punct::Semicolon)?;
						Some(ty)
					}
					Meta(Some(Token::Punct(Punct::Semicolon)), _) => None,
					Meta(unexpected, loc) => {
						return Err(Meta::new(
							Box::new(Error::Unexpected(
								unexpected,
								vec![
									TokenKind::Punct(Punct::Colon),
									TokenKind::Punct(Punct::Semicolon),
								],
							)),
							parser.build_metadata(loc),
						));
					}
				};

				Ok(Meta::new(
					Item::Property(Meta::new(
						PropertyDefinition {
							id,
							alias: None,
							ty,
							doc,
						},
						parser.build_metadata(loc),
					)),
					parser.build_metadata(loc),
				))
			}
			Token::Keyword(lexing::Keyword::Layout) => {
				let id = Id::parse_in(parser)?;

				let ty_id = match parser.peek()?.value() {
					Some(Token::Keyword(Keyword::For)) => {
						parser.next()?;
						let ty_id = Id::parse_in(parser)?;
						loc.append(ty_id.metadata().optional_span().unwrap());
						Some(ty_id)
					}
					_ => None,
				};

				let description = match parser.next()? {
					Meta(Some(Token::Begin(Delimiter::Brace)), block_loc) => {
						let Meta(fields, fields_loc) = parser.parse_block(block_loc)?;
						Meta(LayoutDescription::Normal(fields), fields_loc)
					}
					Meta(Some(Token::Punct(Punct::Equal)), _) => {
						let Meta(expr, expr_loc) = OuterLayoutExpr::parse_in(parser)?;
						parser.parse_punct(Punct::Semicolon)?;
						Meta(LayoutDescription::Alias(expr), expr_loc)
					}
					Meta(Some(Token::Punct(Punct::Semicolon)), loc) => Meta(
						LayoutDescription::Normal(Vec::new()),
						parser.build_metadata(loc),
					),
					Meta(unexpected, loc) => {
						return Err(Meta::new(
							Box::new(Error::Unexpected(
								unexpected,
								vec![
									TokenKind::Begin(Delimiter::Brace),
									TokenKind::Punct(Punct::Equal),
								],
							)),
							parser.build_metadata(loc),
						));
					}
				};

				loc.append(description.metadata().optional_span().unwrap());
				Ok(Meta::new(
					Item::Layout(Meta::new(
						LayoutDefinition {
							id,
							ty_id,
							description,
							doc,
						},
						parser.build_metadata(loc),
					)),
					parser.build_metadata(loc),
				))
			}
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(Some(unexpected), Self::FIRST.to_vec())),
				parser.build_metadata(loc),
			)),
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for LayoutDescription<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Begin(Delimiter::Brace),
		TokenKind::Punct(Punct::Semicolon),
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token {
			Token::Begin(Delimiter::Brace) => {
				let Meta(fields, loc) = parser.parse_block(loc)?;
				Ok(Meta(LayoutDescription::Normal(fields), loc))
			}
			Token::Punct(Punct::Semicolon) => Ok(Meta(
				LayoutDescription::Normal(Vec::new()),
				parser.build_metadata(loc),
			)),
			other => {
				let Meta(expr, expr_loc) = OuterLayoutExpr::parse_from(parser, other, loc)?;
				parser.parse_punct(Punct::Semicolon)?;
				Ok(Meta(LayoutDescription::Alias(expr), expr_loc))
			}
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for PropertyDefinition<M> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Doc, TokenKind::Id];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		token_span: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let (doc, k) = Documentation::try_parse_from(parser, token, token_span)?;

		let id = Id::parse_from_continuation(parser, k)?;
		let mut loc = id.metadata().optional_span().unwrap();

		let alias = match parser.peek()? {
			Meta(Some(Token::Keyword(lexing::Keyword::As)), _) => {
				parser.next()?;
				let alias = Alias::parse_in(parser)?;
				loc.append(alias.metadata().optional_span().unwrap());
				Some(alias)
			}
			_ => None,
		};

		let ty = match parser.peek()?.into_value() {
			Some(Token::Punct(lexing::Punct::Colon)) => {
				parser.next()?;
				let ty = AnnotatedTypeExpr::parse_in(parser)?;
				loc.append(ty.metadata().optional_span().unwrap());
				Some(ty)
			}
			_ => None,
		};

		Ok(Meta::new(
			Self { id, alias, ty, doc },
			parser.build_metadata(loc),
		))
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for FieldDefinition<M> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Doc, TokenKind::Id];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		token_span: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let (doc, k) = Documentation::try_parse_from(parser, token, token_span)?;
		let id = Id::parse_from_continuation(parser, k)?;
		let mut loc = id.metadata().optional_span().unwrap();

		let alias = match parser.peek()? {
			Meta(Some(Token::Keyword(lexing::Keyword::As)), _) => {
				parser.next()?;
				let alias = Alias::parse_in(parser)?;
				loc.append(alias.metadata().optional_span().unwrap());
				Some(alias)
			}
			_ => None,
		};

		let layout = match parser.peek()?.into_value() {
			Some(Token::Punct(lexing::Punct::Colon)) => {
				parser.next()?;
				let ty = AnnotatedLayoutExpr::parse_in(parser)?;
				loc.append(ty.metadata().optional_span().unwrap());
				Some(ty)
			}
			_ => None,
		};

		Ok(Meta::new(
			Self {
				id,
				layout,
				alias,
				doc,
			},
			parser.build_metadata(loc),
		))
	}
}

impl<M> Parse<M> for Alias {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id, TokenKind::Literal];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token {
			Token::Id(Id::Name(alias)) => Ok(Meta::new(Alias(alias), parser.build_metadata(loc))),
			Token::Id(id) => Err(Meta::new(
				Box::new(Error::InvalidAlias(id)),
				parser.build_metadata(loc),
			)),
			Token::Literal(Literal::String(alias)) => {
				Ok(Meta::new(Alias(alias), parser.build_metadata(loc)))
			}
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id])),
				parser.build_metadata(loc),
			)),
		}
	}
}

impl<M> Parse<M> for Prefix {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token {
			Token::Id(Id::Name(alias)) => Ok(Meta::new(Prefix(alias), parser.build_metadata(loc))),
			Token::Id(id) => Err(Meta::new(
				Box::new(Error::InvalidPrefix(id)),
				parser.build_metadata(loc),
			)),
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id])),
				parser.build_metadata(loc),
			)),
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for AnnotatedTypeExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Keyword(Keyword::Annotation(Annotation::Multiple)),
		TokenKind::Keyword(Keyword::Annotation(Annotation::Required)),
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		mut loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let mut annotations = Vec::new();

		let k = match token {
			Token::Keyword(Keyword::Annotation(a)) => {
				annotations.push(Meta::new(a, parser.build_metadata(loc)));
				while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
					parser.peek()?
				{
					loc.append(a_loc);
					annotations.push(Meta::new(*a, parser.build_metadata(a_loc)));
					parser.next()?;
				}

				None
			}
			token => Some(Meta(token, loc)),
		};

		let expr = OuterTypeExpr::parse_from_continuation(parser, k)?;
		loc.append(expr.metadata().optional_span().unwrap());

		while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) = parser.peek()? {
			loc.append(a_loc);
			annotations.push(Meta::new(*a, parser.build_metadata(a_loc)));
			parser.next()?;
		}

		Ok(Meta::new(
			Self { expr, annotations },
			parser.build_metadata(loc),
		))
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for OuterTypeExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		mut loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let Meta(first, first_loc) = NamedInnerTypeExpr::parse_from(parser, token, loc)?;

		match parser.peek()? {
			Meta(Some(Token::Punct(Punct::Pipe)), _) => {
				let mut options = vec![Meta(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Pipe)), _) = parser.peek()? {
					parser.next()?;
					let item = NamedInnerTypeExpr::parse_in(parser)?;
					loc.append(item.metadata().optional_span().unwrap());
					options.push(item);
				}

				Ok(Meta(
					Self::Union(parser.next_label(), options),
					parser.build_metadata(loc),
				))
			}
			Meta(Some(Token::Punct(Punct::Ampersand)), _) => {
				let mut types = vec![Meta(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Ampersand)), _) = parser.peek()? {
					parser.next()?;
					let item = NamedInnerTypeExpr::parse_in(parser)?;
					loc.append(item.metadata().optional_span().unwrap());
					types.push(item);
				}

				Ok(Meta(
					Self::Intersection(parser.next_label(), types),
					parser.build_metadata(loc),
				))
			}
			_ => Ok(Meta(Self::Inner(first), first_loc)),
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for NamedInnerTypeExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let expr = InnerTypeExpr::parse_from(parser, token, loc)?;
		let mut loc = expr.metadata().optional_span().unwrap();

		let layout = match parser.peek()? {
			Meta(Some(Token::Keyword(Keyword::With)), _) => {
				parser.next()?;
				let layout = NamedInnerLayoutExpr::parse_in(parser)?;
				loc.append(layout.metadata().optional_span().unwrap());
				NamedInnerTypeExprLayout::Explicit(layout)
			}
			Meta(Some(Token::Keyword(Keyword::As)), _) => {
				parser.next()?;
				let name = Alias::parse_in(parser)?;
				loc.append(name.metadata().optional_span().unwrap());
				NamedInnerTypeExprLayout::Implicit(Some(name))
			}
			_ => NamedInnerTypeExprLayout::Implicit(None),
		};

		Ok(Meta(Self { expr, layout }, parser.build_metadata(loc)))
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for InnerTypeExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Keyword(Keyword::All),
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		mut loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token {
			Token::Keyword(Keyword::All) => {
				#[allow(clippy::type_complexity)]
				fn parse_property_restriction<L, F, M>(
					parser: &mut Parser<L, F>,
					prop: Meta<Id, M>,
					alias: Option<Meta<Alias, M>>,
					mut loc: Span,
				) -> Result<Meta<TypeRestrictedProperty<M>, M>, MetaError<L::Error, M>>
				where
					L: Tokens,
					F: FnMut(Span) -> M,
					M: MaybeSpanned<Span = Span>,
				{
					let ty = InnerTypeExpr::parse_in(parser)?;
					let restriction_loc = ty.metadata().optional_span().unwrap();
					loc.append(ty.metadata().optional_span().unwrap());
					Ok(Meta(
						TypeRestrictedProperty {
							prop,
							alias,
							restriction: Meta(
								TypePropertyRestriction::Range(TypePropertyRangeRestriction::All(
									Box::new(ty),
								)),
								parser.build_metadata(restriction_loc),
							),
						},
						parser.build_metadata(loc),
					))
				}

				let id = Id::parse_in(parser)?;

				match parser.next()? {
					Meta(Some(Token::Keyword(lexing::Keyword::As)), _) => {
						let alias = Alias::parse_in(parser)?;
						match parser
							.next_expected(|| vec![TokenKind::Punct(lexing::Punct::Colon)])?
						{
							Meta(Token::Punct(lexing::Punct::Colon), _) => {
								let restriction =
									parse_property_restriction(parser, id, Some(alias), loc)?;
								Ok(restriction.map(Self::PropertyRestriction))
							}
							Meta(unexpected, loc) => Err(Meta::new(
								Box::new(Error::Unexpected(
									Some(unexpected),
									vec![TokenKind::Punct(lexing::Punct::Colon)],
								)),
								parser.build_metadata(loc),
							)),
						}
					}
					Meta(Some(Token::Punct(lexing::Punct::Colon)), _) => {
						let restriction = parse_property_restriction(parser, id, None, loc)?;
						Ok(restriction.map(Self::PropertyRestriction))
					}
					Meta(unexpected, loc) => Err(Meta::new(
						Box::new(Error::Unexpected(
							unexpected,
							vec![
								TokenKind::Keyword(Keyword::As),
								TokenKind::Punct(lexing::Punct::Colon),
							],
						)),
						parser.build_metadata(loc),
					)),
				}
			}
			token => match token.no_keyword() {
				Token::Id(id) => Ok(Meta::new(
					Self::Id(Meta::new(id, parser.build_metadata(loc))),
					parser.build_metadata(loc),
				)),
				Token::Punct(lexing::Punct::Ampersand) => {
					let arg = Self::parse_in(parser)?;
					loc.set_end(arg.metadata().optional_span().unwrap().end());
					Ok(Meta::new(
						Self::Reference(Box::new(arg)),
						parser.build_metadata(loc),
					))
				}
				Token::Literal(lit) => {
					let label = parser.next_label();
					Ok(Meta::new(
						Self::Literal(label, lit),
						parser.build_metadata(loc),
					))
				}
				Token::Begin(Delimiter::Bracket) => {
					let label = parser.next_label();
					let item = OuterTypeExpr::parse_in(parser)?;
					let end_loc = parser.parse_end(Delimiter::Bracket)?;
					loc.append(end_loc);
					Ok(Meta::new(
						Self::List(label, Box::new(item)),
						parser.build_metadata(loc),
					))
				}
				Token::Begin(Delimiter::Parenthesis) => {
					let outer = OuterTypeExpr::parse_in(parser)?;
					loc.append(parser.parse_end(Delimiter::Parenthesis)?);
					Ok(Meta(
						Self::Outer(Box::new(outer)),
						parser.build_metadata(loc),
					))
				}
				unexpected => Err(Meta::new(
					Box::new(Error::Unexpected(Some(unexpected), Self::FIRST.to_vec())),
					parser.build_metadata(loc),
				)),
			},
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for AnnotatedLayoutExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Keyword(Keyword::Annotation(Annotation::Multiple)),
		TokenKind::Keyword(Keyword::Annotation(Annotation::Required)),
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		mut loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let mut annotations = Vec::new();

		let k = match token {
			Token::Keyword(Keyword::Annotation(a)) => {
				annotations.push(Meta::new(a, parser.build_metadata(loc)));
				while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
					parser.peek()?
				{
					loc.append(a_loc);
					annotations.push(Meta::new(*a, parser.build_metadata(a_loc)));
					parser.next()?;
				}

				None
			}
			token => Some(Meta(token, loc)),
		};

		let expr = OuterLayoutExpr::parse_from_continuation(parser, k)?;
		loc.append(expr.metadata().optional_span().unwrap());

		while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) = parser.peek()? {
			loc.append(a_loc);
			annotations.push(Meta::new(*a, parser.build_metadata(a_loc)));
			parser.next()?;
		}

		Ok(Meta::new(
			Self { expr, annotations },
			parser.build_metadata(loc),
		))
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for OuterLayoutExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		mut loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let Meta(first, first_loc) = NamedInnerLayoutExpr::parse_from(parser, token, loc)?;

		match parser.peek()? {
			Meta(Some(Token::Punct(Punct::Pipe)), _) => {
				let mut options = vec![Meta(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Pipe)), _) = parser.peek()? {
					parser.next()?;
					let item = NamedInnerLayoutExpr::parse_in(parser)?;
					loc.append(item.metadata().optional_span().unwrap());
					options.push(item);
				}

				Ok(Meta(Self::Union(None, options), parser.build_metadata(loc)))
			}
			Meta(Some(Token::Punct(Punct::Ampersand)), _) => {
				let mut layouts = vec![Meta(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Ampersand)), _) = parser.peek()? {
					parser.next()?;
					let item = NamedInnerLayoutExpr::parse_in(parser)?;
					loc.append(item.metadata().optional_span().unwrap());
					layouts.push(item);
				}

				Ok(Meta(
					Self::Intersection(None, layouts),
					parser.build_metadata(loc),
				))
			}
			_ => Ok(Meta(Self::Inner(first), first_loc)),
		}
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for NamedInnerLayoutExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		let expr = InnerLayoutExpr::parse_from(parser, token, loc)?;
		let mut loc = expr.metadata().optional_span().unwrap();
		let name = if let Meta(Some(Token::Keyword(Keyword::As)), _) = parser.peek()? {
			parser.next()?;
			let name = Alias::parse_in(parser)?;
			loc.append(name.metadata().optional_span().unwrap());
			Some(name)
		} else {
			None
		};

		Ok(Meta(Self { expr, name }, parser.build_metadata(loc)))
	}
}

impl<M: MaybeSpanned<Span = Span>> Parse<M> for InnerLayoutExpr<M> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L, F>(
		parser: &mut Parser<L, F>,
		token: Token,
		mut loc: Span,
	) -> Result<Meta<Self, M>, MetaError<L::Error, M>>
	where
		L: Tokens,
		F: FnMut(Span) -> M,
	{
		match token.no_keyword() {
			Token::Id(id) => Ok(Meta::new(
				Self::Id(Meta::new(id, parser.build_metadata(loc))),
				parser.build_metadata(loc),
			)),
			Token::Punct(lexing::Punct::Ampersand) => {
				let arg = InnerTypeExpr::parse_in(parser)?;
				loc.append(arg.metadata().optional_span().unwrap());
				Ok(Meta::new(
					Self::Reference(Box::new(arg)),
					parser.build_metadata(loc),
				))
			}
			Token::Literal(lit) => Ok(Meta::new(
				Self::Literal(None, lit),
				parser.build_metadata(loc),
			)),
			Token::Begin(Delimiter::Bracket) => {
				let item = OuterLayoutExpr::parse_in(parser)?;
				let end_loc = parser.parse_end(Delimiter::Bracket)?;
				loc.append(end_loc);
				Ok(Meta::new(
					Self::Array(None, Box::new(item)),
					parser.build_metadata(loc),
				))
			}
			Token::Begin(Delimiter::Parenthesis) => {
				let outer = OuterLayoutExpr::parse_in(parser)?;
				loc.append(parser.parse_end(Delimiter::Parenthesis)?);
				Ok(Meta(
					Self::Outer(Box::new(outer)),
					parser.build_metadata(loc),
				))
			}
			unexpected => Err(Meta::new(
				Box::new(Error::Unexpected(Some(unexpected), Self::FIRST.to_vec())),
				parser.build_metadata(loc),
			)),
		}
	}
}
