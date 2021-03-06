use super::*;
use lexing::{Delimiter, Keyword, Punct, Token, TokenKind, Tokens};
use locspan::{Loc, Location, MapLocErr};
use std::{fmt, fmt::Debug};
use treeldr::reporting;

/// Parse error.
#[derive(Debug)]
pub enum Error<E> {
	Unexpected(Option<Token>, Vec<lexing::TokenKind>),
	InvalidUseId(Id),
	InvalidPrefix(Id),
	InvalidAlias(Id),
	Lexer(E),
}

impl<E: Debug + fmt::Display, F: Clone> reporting::DiagnoseWithCause<F> for Error<E> {
	fn message(&self, _cause: Option<&Location<F>>) -> String {
		match self {
			Self::Unexpected(_, _) => "parsing error".to_owned(),
			Self::InvalidUseId(_) => "invalid use IRI".to_owned(),
			Self::InvalidPrefix(_) => "invalid prefix".to_owned(),
			Self::InvalidAlias(_) => "invalid alias".to_owned(),
			Self::Lexer(_) => "lexing error".to_owned(),
		}
	}

	fn labels(&self, cause: Option<&Location<F>>) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		match cause {
			Some(loc) => {
				vec![
					codespan_reporting::diagnostic::Label::primary(loc.file().clone(), loc.span())
						.with_message(self.to_string()),
				]
			}
			None => Vec::new(),
		}
	}

	fn notes(&self, _cause: Option<&Location<F>>) -> Vec<String> {
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

/// Parsable abstract syntax nodes.
pub trait Parse<F>: Sized {
	const FIRST: &'static [TokenKind];

	#[allow(clippy::type_complexity)]
	fn parse<L: Tokens<F>>(lexer: &mut L) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match lexer.next().map_loc_err(Error::Lexer)? {
			Meta(Some(token), loc) => Self::parse_from(lexer, token, loc),
			Meta(None, loc) => Err(Loc(Error::Unexpected(None, Self::FIRST.to_vec()), loc)),
		}
	}

	#[allow(clippy::type_complexity)]
	fn parse_from_continuation<L: Tokens<F>>(
		lexer: &mut L,
		token_opt: Option<Loc<Token, F>>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token_opt {
			Some(Meta(token, loc)) => Self::parse_from(lexer, token, loc),
			None => Self::parse(lexer),
		}
	}

	#[allow(clippy::type_complexity)]
	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>>;

	#[allow(clippy::type_complexity)]
	fn try_parse<L: Tokens<F>>(
		lexer: &mut L,
	) -> Result<Option<Loc<Self, F>>, Loc<Error<L::Error>, F>> {
		match lexer.peek().map_loc_err(Error::Lexer)? {
			Meta(Some(token), _) => {
				if token.kind().matches_any(Self::FIRST) {
					Ok(Some(Self::parse(lexer)?))
				} else {
					Ok(None)
				}
			}
			_ => Ok(None),
		}
	}

	#[allow(clippy::type_complexity)]
	fn try_parse_from_continuation<L: Tokens<F>>(
		lexer: &mut L,
		token_opt: Option<Loc<Token, F>>,
	) -> Result<Option<Loc<Self, F>>, Loc<Error<L::Error>, F>> {
		match token_opt {
			Some(Meta(token, loc)) => {
				if token.kind().matches_any(Self::FIRST) {
					Ok(Some(Self::parse_from(lexer, token, loc)?))
				} else {
					Ok(None)
				}
			}
			None => Self::try_parse(lexer),
		}
	}

	#[allow(clippy::type_complexity)]
	fn try_parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<(Option<Loc<Self, F>>, Option<Loc<Token, F>>), Loc<Error<L::Error>, F>> {
		if token.kind().matches_any(Self::FIRST) {
			Ok((Some(Self::parse_from(lexer, token, loc)?), None))
		} else {
			Ok((None, Some(Loc(token, loc))))
		}
	}
}

#[allow(clippy::type_complexity)]
fn peek_token<F, L: Tokens<F>>(
	tokens: &mut L,
) -> Result<Loc<Option<&Token>, F>, Loc<Error<L::Error>, F>> {
	tokens.peek().map_loc_err(Error::Lexer)
}

#[allow(clippy::type_complexity)]
fn next_token<F, L: Tokens<F>>(
	tokens: &mut L,
) -> Result<Loc<Option<Token>, F>, Loc<Error<L::Error>, F>> {
	tokens.next().map_loc_err(Error::Lexer)
}

#[allow(clippy::type_complexity)]
fn next_expected_token<F, L: Tokens<F>>(
	tokens: &mut L,
	expected: impl FnOnce() -> Vec<TokenKind>,
) -> Result<Loc<Token, F>, Loc<Error<L::Error>, F>> {
	match next_token(tokens)? {
		Meta(None, loc) => Err(Loc::new(Error::Unexpected(None, expected()), loc)),
		Meta(Some(token), loc) => Ok(Loc::new(token, loc)),
	}
}

#[allow(clippy::type_complexity)]
fn next_expected_token_from<F, L: Tokens<F>>(
	tokens: &mut L,
	token_opt: Option<Loc<Token, F>>,
	expected: impl FnOnce() -> Vec<TokenKind>,
) -> Result<Loc<Token, F>, Loc<Error<L::Error>, F>> {
	match token_opt {
		Some(token) => Ok(token),
		None => next_expected_token(tokens, expected),
	}
}

#[allow(clippy::type_complexity)]
fn parse_comma_separated_list<F, L: Tokens<F>, T: Parse<F>>(
	lexer: &mut L,
) -> Result<Vec<Loc<T, F>>, Loc<Error<L::Error>, F>> {
	let mut list = Vec::new();

	while let Some(item) = T::try_parse(lexer)? {
		list.push(item);
		match peek_token(lexer)? {
			Meta(Some(Token::Punct(Punct::Comma)), _) => {
				next_token(lexer)?;
				// continue
			}
			_ => break,
		}
	}

	Ok(list)
}

#[allow(clippy::type_complexity)]
fn parse_block<F, L: Tokens<F>, T: Parse<F>>(
	lexer: &mut L,
	mut loc: Location<F>,
) -> Result<Loc<Vec<Loc<T, F>>, F>, Loc<Error<L::Error>, F>> {
	let items = parse_comma_separated_list(lexer)?;
	match next_token(lexer)? {
		Meta(Some(Token::End(Delimiter::Brace)), end_loc) => {
			loc.span_mut().append(end_loc.span());
			Ok(Loc::new(items, loc))
		}
		Meta(unexpected, loc) => Err(Loc::new(
			Error::Unexpected(
				unexpected,
				vec![
					TokenKind::Punct(Punct::Comma),
					TokenKind::End(Delimiter::Brace),
				],
			),
			loc,
		)),
	}
}

fn parse_keyword<F, L: Tokens<F>>(
	tokens: &mut L,
	keyword: lexing::Keyword,
) -> Result<(), Loc<Error<L::Error>, F>> {
	let Meta(token, span) = next_expected_token(tokens, || vec![TokenKind::Keyword(keyword)])?;

	match token {
		Token::Keyword(k) if k == keyword => Ok(()),
		unexpected => Err(Loc::new(
			Error::Unexpected(Some(unexpected), vec![TokenKind::Keyword(keyword)]),
			span,
		)),
	}
}

fn parse_punct<F, L: Tokens<F>>(
	tokens: &mut L,
	punct: lexing::Punct,
) -> Result<(), Loc<Error<L::Error>, F>> {
	let Meta(token, span) = next_expected_token(tokens, || vec![TokenKind::Punct(punct)])?;

	match token {
		Token::Punct(p) if p == punct => Ok(()),
		unexpected => Err(Loc::new(
			Error::Unexpected(Some(unexpected), vec![TokenKind::Punct(punct)]),
			span,
		)),
	}
}

fn parse_end<F, L: Tokens<F>>(
	tokens: &mut L,
	delimiter: Delimiter,
) -> Result<Location<F>, Loc<Error<L::Error>, F>> {
	let Meta(token, loc) = next_expected_token(tokens, || vec![TokenKind::End(delimiter)])?;

	match token {
		Token::End(d) if d == delimiter => Ok(loc),
		unexpected => Err(Loc::new(
			Error::Unexpected(Some(unexpected), vec![TokenKind::End(delimiter)]),
			loc,
		)),
	}
}

impl<F: Clone> Parse<F> for Document<F> {
	const FIRST: &'static [TokenKind] = Item::<F>::FIRST;

	fn parse<L: Tokens<F>>(lexer: &mut L) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match lexer.next().map_loc_err(Error::Lexer)? {
			Meta(Some(token), loc) => Self::parse_from(lexer, token, loc),
			Meta(None, loc) => Ok(Loc::new(
				Self {
					bases: Vec::new(),
					uses: Vec::new(),
					types: Vec::new(),
					properties: Vec::new(),
					layouts: Vec::new(),
				},
				loc,
			)),
		}
	}

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let mut bases = Vec::new();
		let mut uses = Vec::new();
		let mut types = Vec::new();
		let mut properties = Vec::new();
		let mut layouts = Vec::new();

		let Meta(first_item, mut loc) = Item::parse_from(lexer, token, loc)?;
		match first_item {
			Item::Base(i) => bases.push(i),
			Item::Use(i) => uses.push(i),
			Item::Type(t) => types.push(t),
			Item::Property(p) => properties.push(p),
			Item::Layout(l) => layouts.push(l),
		}

		loop {
			match peek_token(lexer)? {
				Meta(Some(_), _) => {
					let Meta(item, item_loc) = Item::parse(lexer)?;
					loc.span_mut().append(item_loc.span());

					match item {
						Item::Base(i) => bases.push(i),
						Item::Use(i) => uses.push(i),
						Item::Type(t) => types.push(t),
						Item::Property(p) => properties.push(p),
						Item::Layout(l) => layouts.push(l),
					}
				}
				Meta(None, _) => {
					break Ok(Loc::new(
						Self {
							bases,
							uses,
							types,
							properties,
							layouts,
						},
						loc,
					));
				}
			}
		}
	}
}

impl<F> Parse<F> for Id {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id];

	fn parse_from<L: Tokens<F>>(
		_lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token {
			Token::Id(id) => Ok(Loc::new(id, loc)),
			Token::Keyword(kw) => Ok(Loc::new(Id::Name(kw.to_string()), loc)),
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Id]),
				loc,
			)),
		}
	}
}

impl<F: Clone> Parse<F> for Documentation<F> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Doc];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let mut items = Vec::new();
		match token {
			Token::Doc(doc) => {
				items.push(Loc(doc, loc.clone()));
				loop {
					match peek_token(lexer)? {
						Meta(Some(token), _) if token.is_doc() => {
							let doc = next_token(lexer)?.unwrap().map(|t| t.into_doc().unwrap());
							loc.span_mut().append(doc.span());
							items.push(doc)
						}
						_ => break Ok(Loc::new(Self::new(items), loc)),
					}
				}
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Id]),
				loc,
			)),
		}
	}
}

impl<F: Clone> Parse<F> for Item<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Doc,
		TokenKind::Keyword(lexing::Keyword::Base),
		TokenKind::Keyword(lexing::Keyword::Type),
		TokenKind::Keyword(lexing::Keyword::Property),
		TokenKind::Keyword(lexing::Keyword::Layout),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		token_loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let (doc, k) = Documentation::try_parse_from(lexer, token, token_loc)?;

		let Meta(token, mut loc) = next_expected_token_from(lexer, k, || Self::FIRST.to_vec())?;

		match token {
			Token::Keyword(lexing::Keyword::Base) => {
				let id = Id::parse(lexer)?;
				let iri = match id {
					Meta(Id::IriRef(iri_ref), loc) => match IriBuf::try_from(iri_ref) {
						Ok(iri) => Loc::new(iri, loc),
						Err(iri_ref) => {
							return Err(Loc::new(Error::InvalidUseId(Id::IriRef(iri_ref)), loc))
						}
					},
					Meta(id, loc) => return Err(Loc::new(Error::InvalidUseId(id), loc)),
				};

				parse_punct(lexer, Punct::Semicolon)?;

				Ok(Loc::new(Item::Base(iri), loc))
			}
			Token::Keyword(lexing::Keyword::Use) => {
				let id = Id::parse(lexer)?;
				let iri = match id {
					Meta(Id::IriRef(iri_ref), loc) => match IriBuf::try_from(iri_ref) {
						Ok(iri) => Loc::new(iri, loc),
						Err(iri_ref) => {
							return Err(Loc::new(Error::InvalidUseId(Id::IriRef(iri_ref)), loc))
						}
					},
					Meta(id, loc) => return Err(Loc::new(Error::InvalidUseId(id), loc)),
				};

				parse_keyword(lexer, lexing::Keyword::As)?;

				let prefix = Prefix::parse(lexer)?;
				loc.span_mut().append(prefix.span());

				parse_punct(lexer, Punct::Semicolon)?;

				Ok(Loc::new(
					Item::Use(Loc::new(Use { iri, prefix, doc }, loc.clone())),
					loc,
				))
			}
			Token::Keyword(lexing::Keyword::Type) => {
				let id = Id::parse(lexer)?;

				let (description, layout) = match next_token(lexer)? {
					Meta(Some(Token::Begin(Delimiter::Brace)), block_loc) => {
						let Meta(properties, properties_loc) = parse_block(lexer, block_loc)?;

						let layout = match peek_token(lexer)?.value() {
							Some(Token::Keyword(Keyword::With)) => {
								next_token(lexer)?;
								let layout = LayoutDescription::parse(lexer)?;
								loc.span_mut().append(layout.span());
								Some(layout)
							}
							_ => None,
						};

						(
							Loc(TypeDescription::Normal(properties), properties_loc),
							layout,
						)
					}
					Meta(Some(Token::Punct(Punct::Equal)), _) => {
						let Meta(expr, expr_loc) = OuterTypeExpr::parse(lexer)?;

						let layout = match peek_token(lexer)?.value() {
							Some(Token::Keyword(Keyword::With)) => {
								next_token(lexer)?;
								let layout = LayoutDescription::parse(lexer)?;
								loc.span_mut().append(layout.span());
								Some(layout)
							}
							_ => {
								parse_punct(lexer, Punct::Semicolon)?;
								None
							}
						};

						(Loc(TypeDescription::Alias(expr), expr_loc), layout)
					}
					Meta(Some(Token::Punct(Punct::Semicolon)), loc) => {
						(Loc(TypeDescription::Normal(Vec::new()), loc), None)
					}
					Meta(Some(Token::Keyword(Keyword::With)), _) => {
						let layout = LayoutDescription::parse(lexer)?;
						loc.span_mut().append(layout.span());
						(
							Loc(TypeDescription::Normal(Vec::new()), loc.clone()),
							Some(layout),
						)
					}
					Meta(unexpected, loc) => {
						return Err(Loc::new(
							Error::Unexpected(
								unexpected,
								vec![
									TokenKind::Begin(Delimiter::Brace),
									TokenKind::Punct(Punct::Equal),
								],
							),
							loc,
						));
					}
				};

				loc.span_mut().append(description.span());
				Ok(Loc::new(
					Item::Type(Loc::new(
						TypeDefinition {
							id,
							description,
							doc,
							layout,
						},
						loc.clone(),
					)),
					loc,
				))
			}
			Token::Keyword(lexing::Keyword::Property) => {
				let id = Id::parse(lexer)?;

				let ty = match next_token(lexer)? {
					Meta(Some(Token::Punct(Punct::Colon)), _) => {
						let ty = AnnotatedTypeExpr::parse(lexer)?;
						loc.span_mut().append(ty.span());
						parse_punct(lexer, Punct::Semicolon)?;
						Some(ty)
					}
					Meta(Some(Token::Punct(Punct::Semicolon)), _) => None,
					Meta(unexpected, loc) => {
						return Err(Loc::new(
							Error::Unexpected(
								unexpected,
								vec![
									TokenKind::Punct(Punct::Colon),
									TokenKind::Punct(Punct::Semicolon),
								],
							),
							loc,
						));
					}
				};

				Ok(Loc::new(
					Item::Property(Loc::new(
						PropertyDefinition {
							id,
							alias: None,
							ty,
							doc,
						},
						loc.clone(),
					)),
					loc,
				))
			}
			Token::Keyword(lexing::Keyword::Layout) => {
				let id = Id::parse(lexer)?;

				let ty_id = match peek_token(lexer)?.value() {
					Some(Token::Keyword(Keyword::For)) => {
						next_token(lexer)?;
						let ty_id = Id::parse(lexer)?;
						loc.span_mut().append(ty_id.span());
						Some(ty_id)
					}
					_ => None,
				};

				let description = match next_token(lexer)? {
					Meta(Some(Token::Begin(Delimiter::Brace)), block_loc) => {
						let Meta(fields, fields_loc) = parse_block(lexer, block_loc)?;
						Loc(LayoutDescription::Normal(fields), fields_loc)
					}
					Meta(Some(Token::Punct(Punct::Equal)), _) => {
						let Meta(expr, expr_loc) = OuterLayoutExpr::parse(lexer)?;
						parse_punct(lexer, Punct::Semicolon)?;
						Loc(LayoutDescription::Alias(expr), expr_loc)
					}
					Meta(Some(Token::Punct(Punct::Semicolon)), loc) => {
						Loc(LayoutDescription::Normal(Vec::new()), loc)
					}
					Meta(unexpected, loc) => {
						return Err(Loc::new(
							Error::Unexpected(
								unexpected,
								vec![
									TokenKind::Begin(Delimiter::Brace),
									TokenKind::Punct(Punct::Equal),
								],
							),
							loc,
						));
					}
				};

				loc.span_mut().append(description.span());
				Ok(Loc::new(
					Item::Layout(Loc::new(
						LayoutDefinition {
							id,
							ty_id,
							description,
							doc,
						},
						loc.clone(),
					)),
					loc,
				))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), Self::FIRST.to_vec()),
				loc,
			)),
		}
	}
}

impl<F: Clone> Parse<F> for LayoutDescription<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Begin(Delimiter::Brace),
		TokenKind::Punct(Punct::Semicolon),
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token {
			Token::Begin(Delimiter::Brace) => {
				let Meta(fields, loc) = parse_block(lexer, loc)?;
				Ok(Loc(LayoutDescription::Normal(fields), loc))
			}
			Token::Punct(Punct::Semicolon) => Ok(Loc(LayoutDescription::Normal(Vec::new()), loc)),
			other => {
				let Meta(expr, expr_loc) = OuterLayoutExpr::parse_from(lexer, other, loc)?;
				parse_punct(lexer, Punct::Semicolon)?;
				Ok(Loc(LayoutDescription::Alias(expr), expr_loc))
			}
		}
	}
}

impl<F: Clone> Parse<F> for PropertyDefinition<F> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Doc, TokenKind::Id];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		token_loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let (doc, k) = Documentation::try_parse_from(lexer, token, token_loc)?;

		let id = Id::parse_from_continuation(lexer, k)?;
		let mut loc = id.location().clone();

		let alias = match peek_token(lexer)? {
			Meta(Some(Token::Keyword(lexing::Keyword::As)), _) => {
				next_token(lexer)?;
				let alias = Alias::parse(lexer)?;
				loc.span_mut().append(alias.span());
				Some(alias)
			}
			_ => None,
		};

		let ty = match peek_token(lexer)?.into_value() {
			Some(Token::Punct(lexing::Punct::Colon)) => {
				next_token(lexer)?;
				let ty = AnnotatedTypeExpr::parse(lexer)?;
				loc.span_mut().append(ty.span());
				Some(ty)
			}
			_ => None,
		};

		Ok(Loc::new(Self { id, alias, ty, doc }, loc))
	}
}

impl<F: Clone> Parse<F> for FieldDefinition<F> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Doc, TokenKind::Id];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		token_loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let (doc, k) = Documentation::try_parse_from(lexer, token, token_loc)?;
		let id = Id::parse_from_continuation(lexer, k)?;
		let mut loc = id.location().clone();

		let alias = match peek_token(lexer)? {
			Meta(Some(Token::Keyword(lexing::Keyword::As)), _) => {
				next_token(lexer)?;
				let alias = Alias::parse(lexer)?;
				loc.span_mut().append(alias.span());
				Some(alias)
			}
			_ => None,
		};

		let layout = match peek_token(lexer)?.into_value() {
			Some(Token::Punct(lexing::Punct::Colon)) => {
				next_token(lexer)?;
				let ty = AnnotatedLayoutExpr::parse(lexer)?;
				loc.span_mut().append(ty.span());
				Some(ty)
			}
			_ => None,
		};

		Ok(Loc::new(
			Self {
				id,
				layout,
				alias,
				doc,
			},
			loc,
		))
	}
}

impl<F> Parse<F> for Alias {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id];

	fn parse_from<L: Tokens<F>>(
		_lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token {
			Token::Id(Id::Name(alias)) => Ok(Loc::new(Alias(alias), loc)),
			Token::Id(id) => Err(Loc::new(Error::InvalidAlias(id), loc)),
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Id]),
				loc,
			)),
		}
	}
}

impl<F> Parse<F> for Prefix {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id];

	fn parse_from<L: Tokens<F>>(
		_lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token {
			Token::Id(Id::Name(alias)) => Ok(Loc::new(Prefix(alias), loc)),
			Token::Id(id) => Err(Loc::new(Error::InvalidPrefix(id), loc)),
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Id]),
				loc,
			)),
		}
	}
}

impl<F: Clone> Parse<F> for AnnotatedTypeExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Keyword(Keyword::Annotation(Annotation::Multiple)),
		TokenKind::Keyword(Keyword::Annotation(Annotation::Required)),
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let mut annotations = Vec::new();

		let k = match token {
			Token::Keyword(Keyword::Annotation(a)) => {
				annotations.push(Loc::new(a, loc.clone()));
				while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
					peek_token(lexer)?
				{
					loc.span_mut().append(a_loc.span());
					annotations.push(Loc::new(*a, a_loc));
					next_token(lexer)?;
				}

				None
			}
			token => Some(Loc(token, loc.clone())),
		};

		let expr = OuterTypeExpr::parse_from_continuation(lexer, k)?;
		loc.span_mut().append(expr.span());

		while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) = peek_token(lexer)? {
			loc.span_mut().append(a_loc.span());
			annotations.push(Loc::new(*a, a_loc));
			next_token(lexer)?;
		}

		Ok(Loc::new(Self { expr, annotations }, loc))
	}
}

impl<F: Clone> Parse<F> for OuterTypeExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let Meta(first, first_loc) = NamedInnerTypeExpr::parse_from(lexer, token, loc.clone())?;

		match peek_token(lexer)? {
			Meta(Some(Token::Punct(Punct::Pipe)), _) => {
				let mut options = vec![Loc(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Pipe)), _) = peek_token(lexer)? {
					next_token(lexer)?;
					let item = NamedInnerTypeExpr::parse(lexer)?;
					loc.span_mut().append(item.span());
					options.push(item);
				}

				Ok(Loc(Self::Union(lexer.next_label(), options), loc))
			}
			Meta(Some(Token::Punct(Punct::Ampersand)), _) => {
				let mut types = vec![Loc(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Ampersand)), _) = peek_token(lexer)? {
					next_token(lexer)?;
					let item = NamedInnerTypeExpr::parse(lexer)?;
					loc.span_mut().append(item.span());
					types.push(item);
				}

				Ok(Loc(Self::Intersection(lexer.next_label(), types), loc))
			}
			_ => Ok(Loc(Self::Inner(first), first_loc)),
		}
	}
}

impl<F: Clone> Parse<F> for NamedInnerTypeExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let expr = InnerTypeExpr::parse_from(lexer, token, loc)?;
		let mut loc = expr.location().clone();

		let layout = match peek_token(lexer)? {
			Meta(Some(Token::Keyword(Keyword::With)), _) => {
				next_token(lexer)?;
				let layout = NamedInnerLayoutExpr::parse(lexer)?;
				loc.span_mut().append(layout.span());
				NamedInnerTypeExprLayout::Explicit(layout)
			}
			Meta(Some(Token::Keyword(Keyword::As)), _) => {
				next_token(lexer)?;
				let name = Alias::parse(lexer)?;
				loc.span_mut().append(name.span());
				NamedInnerTypeExprLayout::Implicit(Some(name))
			}
			_ => NamedInnerTypeExprLayout::Implicit(None),
		};

		Ok(Loc(Self { expr, layout }, loc))
	}
}

impl<F: Clone> Parse<F> for InnerTypeExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Keyword(Keyword::All),
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token {
			Token::Keyword(Keyword::All) => {
				#[allow(clippy::type_complexity)]
				fn parse_property_restriction<F, L: Tokens<F>>(
					lexer: &mut L,
					prop: Loc<Id, F>,
					alias: Option<Loc<Alias, F>>,
					mut loc: Location<F>,
				) -> Result<Loc<TypeRestrictedProperty<F>, F>, Loc<Error<L::Error>, F>>
				where
					F: Clone,
				{
					let ty = InnerTypeExpr::parse(lexer)?;
					let restriction_loc = ty.location().clone();
					loc.span_mut().append(ty.span());
					Ok(Loc(
						TypeRestrictedProperty {
							prop,
							alias,
							restriction: Loc(
								TypePropertyRestriction::Range(TypePropertyRangeRestriction::All(
									Box::new(ty),
								)),
								restriction_loc,
							),
						},
						loc,
					))
				}

				let id = Id::parse(lexer)?;

				match next_token(lexer)? {
					Meta(Some(Token::Keyword(lexing::Keyword::As)), _) => {
						let alias = Alias::parse(lexer)?;
						match next_expected_token(lexer, || {
							vec![TokenKind::Punct(lexing::Punct::Colon)]
						})? {
							Meta(Token::Punct(lexing::Punct::Colon), _) => {
								let restriction =
									parse_property_restriction(lexer, id, Some(alias), loc)?;
								Ok(restriction.map(Self::PropertyRestriction))
							}
							Meta(unexpected, loc) => Err(Loc::new(
								Error::Unexpected(
									Some(unexpected),
									vec![TokenKind::Punct(lexing::Punct::Colon)],
								),
								loc,
							)),
						}
					}
					Meta(Some(Token::Punct(lexing::Punct::Colon)), _) => {
						let restriction = parse_property_restriction(lexer, id, None, loc)?;
						Ok(restriction.map(Self::PropertyRestriction))
					}
					Meta(unexpected, loc) => Err(Loc::new(
						Error::Unexpected(
							unexpected,
							vec![
								TokenKind::Keyword(Keyword::As),
								TokenKind::Punct(lexing::Punct::Colon),
							],
						),
						loc,
					)),
				}
			}
			token => match token.no_keyword() {
				Token::Id(id) => Ok(Loc::new(Self::Id(Loc::new(id, loc.clone())), loc)),
				Token::Punct(lexing::Punct::Ampersand) => {
					let arg = Self::parse(lexer)?;
					loc.span_mut().set_end(arg.span().end());
					Ok(Loc::new(Self::Reference(Box::new(arg)), loc))
				}
				Token::Literal(lit) => Ok(Loc::new(Self::Literal(lit), loc)),
				Token::Begin(Delimiter::Bracket) => {
					let label = lexer.next_label();
					let item = OuterTypeExpr::parse(lexer)?;
					let end_loc = parse_end(lexer, Delimiter::Bracket)?;
					loc.span_mut().append(end_loc.span());
					Ok(Loc::new(Self::List(label, Box::new(item)), loc))
				}
				Token::Begin(Delimiter::Parenthesis) => {
					let outer = OuterTypeExpr::parse(lexer)?;
					loc.span_mut()
						.append(parse_end(lexer, Delimiter::Parenthesis)?.span());
					Ok(Loc(Self::Outer(Box::new(outer)), loc))
				}
				unexpected => Err(Loc::new(
					Error::Unexpected(Some(unexpected), Self::FIRST.to_vec()),
					loc,
				)),
			},
		}
	}
}

impl<F: Clone> Parse<F> for AnnotatedLayoutExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Keyword(Keyword::Annotation(Annotation::Multiple)),
		TokenKind::Keyword(Keyword::Annotation(Annotation::Required)),
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let mut annotations = Vec::new();

		let k = match token {
			Token::Keyword(Keyword::Annotation(a)) => {
				annotations.push(Loc::new(a, loc.clone()));
				while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
					peek_token(lexer)?
				{
					loc.span_mut().append(a_loc.span());
					annotations.push(Loc::new(*a, a_loc));
					next_token(lexer)?;
				}

				None
			}
			token => Some(Loc(token, loc.clone())),
		};

		let expr = OuterLayoutExpr::parse_from_continuation(lexer, k)?;
		loc.span_mut().append(expr.span());

		while let Meta(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) = peek_token(lexer)? {
			loc.span_mut().append(a_loc.span());
			annotations.push(Loc::new(*a, a_loc));
			next_token(lexer)?;
		}

		Ok(Loc::new(Self { expr, annotations }, loc))
	}
}

impl<F: Clone> Parse<F> for OuterLayoutExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let Meta(first, first_loc) = NamedInnerLayoutExpr::parse_from(lexer, token, loc.clone())?;

		match peek_token(lexer)? {
			Meta(Some(Token::Punct(Punct::Pipe)), _) => {
				let mut options = vec![Loc(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Pipe)), _) = peek_token(lexer)? {
					next_token(lexer)?;
					let item = NamedInnerLayoutExpr::parse(lexer)?;
					loc.span_mut().append(item.span());
					options.push(item);
				}

				Ok(Loc(Self::Union(lexer.next_label(), options), loc))
			}
			Meta(Some(Token::Punct(Punct::Ampersand)), _) => {
				let mut layouts = vec![Loc(first, first_loc)];
				while let Meta(Some(Token::Punct(Punct::Ampersand)), _) = peek_token(lexer)? {
					next_token(lexer)?;
					let item = NamedInnerLayoutExpr::parse(lexer)?;
					loc.span_mut().append(item.span());
					layouts.push(item);
				}

				Ok(Loc(Self::Intersection(lexer.next_label(), layouts), loc))
			}
			_ => Ok(Loc(Self::Inner(first), first_loc)),
		}
	}
}

impl<F: Clone> Parse<F> for NamedInnerLayoutExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let expr = InnerLayoutExpr::parse_from(lexer, token, loc)?;
		let mut loc = expr.location().clone();
		let name = if let Meta(Some(Token::Keyword(Keyword::As)), _) = peek_token(lexer)? {
			next_token(lexer)?;
			let name = Alias::parse(lexer)?;
			loc.span_mut().append(name.span());
			Some(name)
		} else {
			None
		};

		Ok(Loc(Self { expr, name }, loc))
	}
}

impl<F: Clone> Parse<F> for InnerLayoutExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
		TokenKind::Literal,
		TokenKind::Begin(Delimiter::Bracket),
		TokenKind::Begin(Delimiter::Parenthesis),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token.no_keyword() {
			Token::Id(id) => Ok(Loc::new(Self::Id(Loc::new(id, loc.clone())), loc)),
			Token::Punct(lexing::Punct::Ampersand) => {
				let arg = InnerTypeExpr::parse(lexer)?;
				loc.span_mut().append(arg.span());
				Ok(Loc::new(Self::Reference(Box::new(arg)), loc))
			}
			Token::Literal(lit) => Ok(Loc::new(Self::Literal(lit), loc)),
			Token::Begin(Delimiter::Bracket) => {
				let label = lexer.next_label();
				let item = OuterLayoutExpr::parse(lexer)?;
				let end_loc = parse_end(lexer, Delimiter::Bracket)?;
				loc.span_mut().append(end_loc.span());
				Ok(Loc::new(Self::Array(label, Box::new(item)), loc))
			}
			Token::Begin(Delimiter::Parenthesis) => {
				let outer = OuterLayoutExpr::parse(lexer)?;
				loc.span_mut()
					.append(parse_end(lexer, Delimiter::Parenthesis)?.span());
				Ok(Loc(Self::Outer(Box::new(outer)), loc))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), Self::FIRST.to_vec()),
				loc,
			)),
		}
	}
}
