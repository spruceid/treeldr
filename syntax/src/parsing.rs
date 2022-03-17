use super::*;
use lexing::{Delimiter, Keyword, Punct, Token, TokenKind, Tokens};
use locspan::{Loc, Location, MapLocErr};
use std::{fmt, fmt::Debug};

/// Parse error.
#[derive(Debug)]
pub enum Error<E> {
	Unexpected(Option<Token>, Vec<lexing::TokenKind>),
	InvalidImportId(Id),
	InvalidPrefix(Id),
	InvalidAlias(Id),
	Lexer(E),
}

impl<E: Debug + fmt::Display, F: Clone> reporting::Diagnose<F> for Loc<Error<E>, F> {
	fn message(&self) -> String {
		match self.value() {
			Error::Unexpected(_, _) => "parsing error".to_owned(),
			Error::InvalidImportId(_) => "invalid import IRI".to_owned(),
			Error::InvalidPrefix(_) => "invalid prefix".to_owned(),
			Error::InvalidAlias(_) => "invalid alias".to_owned(),
			Error::Lexer(_) => "lexing error".to_owned(),
		}
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		vec![codespan_reporting::diagnostic::Label::primary(
			self.location().file().clone(),
			self.location().span(),
		)
		.with_message(self.to_string())]
	}

	fn notes(&self) -> Vec<String> {
		if let Error::Unexpected(_, expected) = self.value() {
			if !expected.is_empty() {
				let mut note = "expected ".to_owned();

				let len = expected.len();
				for (i, token) in expected.iter().enumerate() {
					if i > 0 {
						if i + 1 == len {
							note.push_str("or ");
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
			Self::InvalidImportId(id) => write!(f, "invalid import IRI `{}`", id),
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
			Loc(Some(token), loc) => Self::parse_from(lexer, token, loc),
			Loc(None, loc) => Err(Loc(Error::Unexpected(None, Self::FIRST.to_vec()), loc)),
		}
	}

	#[allow(clippy::type_complexity)]
	fn parse_from_continuation<L: Tokens<F>>(
		lexer: &mut L,
		token_opt: Option<Loc<Token, F>>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token_opt {
			Some(Loc(token, loc)) => Self::parse_from(lexer, token, loc),
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
			Loc(Some(token), _) => {
				if Self::FIRST.contains(&token.kind()) {
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
			Some(Loc(token, loc)) => {
				if Self::FIRST.contains(&token.kind()) {
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
		if Self::FIRST.contains(&token.kind()) {
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
		locspan::Loc(None, loc) => Err(Loc::new(Error::Unexpected(None, expected()), loc)),
		locspan::Loc(Some(token), loc) => Ok(Loc::new(token, loc)),
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
			Loc(Some(Token::Punct(Punct::Comma)), _) => {
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
) -> Result<Loc<Vec<Loc<T, F>>, F>, Loc<Error<L::Error>, F>> {
	match next_token(lexer)? {
		Loc(Some(Token::Begin(Delimiter::Brace)), mut loc) => {
			let items = parse_comma_separated_list(lexer)?;
			match next_token(lexer)? {
				Loc(Some(Token::End(Delimiter::Brace)), end_loc) => {
					loc.span_mut().append(end_loc.span());
					Ok(Loc::new(items, loc))
				}
				Loc(unexpected, loc) => Err(Loc::new(
					Error::Unexpected(unexpected, vec![TokenKind::Begin(Delimiter::Brace)]),
					loc,
				)),
			}
		}
		Loc(unexpected, loc) => Err(Loc::new(
			Error::Unexpected(unexpected, vec![TokenKind::Begin(Delimiter::Brace)]),
			loc,
		)),
	}
}

fn parse_keyword<F, L: Tokens<F>>(
	tokens: &mut L,
	keyword: lexing::Keyword,
) -> Result<(), Loc<Error<L::Error>, F>> {
	let locspan::Loc(token, span) =
		next_expected_token(tokens, || vec![TokenKind::Keyword(keyword)])?;

	match token {
		Token::Keyword(k) if k == keyword => Ok(()),
		unexpected => Err(Loc::new(
			Error::Unexpected(Some(unexpected), vec![TokenKind::Keyword(keyword)]),
			span,
		)),
	}
}

impl<F: Clone> Parse<F> for Document<F> {
	const FIRST: &'static [TokenKind] = Item::<F>::FIRST;

	fn parse<L: Tokens<F>>(lexer: &mut L) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match lexer.next().map_loc_err(Error::Lexer)? {
			Loc(Some(token), loc) => Self::parse_from(lexer, token, loc),
			Loc(None, loc) => Ok(Loc::new(
				Self {
					bases: Vec::new(),
					imports: Vec::new(),
					types: Vec::new(),
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
		let mut imports = Vec::new();
		let mut types = Vec::new();
		let mut layouts = Vec::new();

		let Loc(first_item, mut loc) = Item::parse_from(lexer, token, loc)?;
		match first_item {
			Item::Base(i) => bases.push(Loc(i, loc.clone())),
			Item::Import(i) => imports.push(i),
			Item::Type(t) => types.push(t),
			Item::Layout(l) => layouts.push(l),
		}

		loop {
			match peek_token(lexer)? {
				locspan::Loc(Some(_), _) => {
					let Loc(item, item_loc) = Item::parse(lexer)?;
					loc.span_mut().append(item_loc.span());

					match item {
						Item::Base(i) => bases.push(Loc(i, item_loc)),
						Item::Import(i) => imports.push(i),
						Item::Type(t) => types.push(t),
						Item::Layout(l) => layouts.push(l),
					}
				}
				locspan::Loc(None, _) => {
					break Ok(Loc::new(
						Self {
							bases,
							imports,
							types,
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
						locspan::Loc(Some(token), _) if token.is_doc() => {
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
		TokenKind::Keyword(lexing::Keyword::Layout),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		token_loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let (doc, next_token) = Documentation::try_parse_from(lexer, token, token_loc)?;

		let Loc(token, mut loc) =
			next_expected_token_from(lexer, next_token, || Self::FIRST.to_vec())?;

		match token {
			Token::Keyword(lexing::Keyword::Import) => {
				let id = Id::parse(lexer)?;
				let iri = match id {
					locspan::Loc(Id::IriRef(iri_ref), loc) => match IriBuf::try_from(iri_ref) {
						Ok(iri) => Loc::new(iri, loc),
						Err(iri_ref) => {
							return Err(Loc::new(Error::InvalidImportId(Id::IriRef(iri_ref)), loc))
						}
					},
					locspan::Loc(id, loc) => return Err(Loc::new(Error::InvalidImportId(id), loc)),
				};

				parse_keyword(lexer, lexing::Keyword::As)?;

				let prefix = Prefix::parse(lexer)?;
				loc.span_mut().append(prefix.span());

				Ok(Loc::new(
					Item::Import(Loc::new(Import { iri, prefix, doc }, loc.clone())),
					loc,
				))
			}
			Token::Keyword(lexing::Keyword::Type) => {
				let id = Id::parse(lexer)?;
				let properties = parse_block(lexer)?;
				loc.span_mut().append(properties.span());
				Ok(Loc::new(
					Item::Type(Loc::new(
						TypeDefinition {
							id,
							properties,
							doc,
						},
						loc.clone(),
					)),
					loc,
				))
			}
			Token::Keyword(lexing::Keyword::Layout) => {
				let id = Id::parse(lexer)?;
				parse_keyword(lexer, lexing::Keyword::For)?;
				let ty_id = Id::parse(lexer)?;
				let fields = parse_block(lexer)?;
				loc.span_mut().append(fields.span());
				Ok(Loc::new(
					Item::Layout(Loc::new(
						LayoutDefinition {
							id,
							ty_id,
							fields,
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
		let ty = match peek_token(lexer)?.into_value() {
			Some(Token::Punct(lexing::Punct::Colon)) => {
				next_token(lexer)?;
				let ty = AnnotatedTypeExpr::parse(lexer)?;
				loc.span_mut().append(ty.span());
				Some(ty)
			}
			_ => None,
		};

		Ok(Loc::new(Self { id, ty, doc }, loc))
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
			locspan::Loc(Some(Token::Keyword(lexing::Keyword::As)), _) => {
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
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let mut annotations = Vec::new();

		let k = match token {
			Token::Keyword(Keyword::Annotation(a)) => {
				next_token(lexer)?;
				annotations.push(Loc::new(a, loc.clone()));
				while let Loc(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
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

		let expr = TypeExpr::parse_from_continuation(lexer, k)?;

		while let locspan::Loc(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
			peek_token(lexer)?
		{
			loc.span_mut().append(a_loc.span());
			annotations.push(Loc::new(*a, a_loc));
			next_token(lexer)?;
		}

		Ok(Loc::new(Self { expr, annotations }, loc))
	}
}

impl<F: Clone> Parse<F> for TypeExpr<F> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id, TokenKind::Punct(Punct::Ampersand)];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token {
			Token::Id(id) => Ok(Loc::new(Self::Id(Loc::new(id, loc.clone())), loc)),
			Token::Punct(lexing::Punct::Ampersand) => {
				let arg = Self::parse(lexer)?;
				loc.span_mut().set_end(arg.span().end());
				Ok(Loc::new(Self::Reference(Box::new(arg)), loc))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), Self::FIRST.to_vec()),
				loc,
			)),
		}
	}
}

impl<F: Clone> Parse<F> for AnnotatedLayoutExpr<F> {
	const FIRST: &'static [TokenKind] = &[
		TokenKind::Keyword(Keyword::Annotation(Annotation::Multiple)),
		TokenKind::Keyword(Keyword::Annotation(Annotation::Required)),
		TokenKind::Id,
		TokenKind::Punct(Punct::Ampersand),
	];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		let mut annotations = Vec::new();

		let k = match token {
			Token::Keyword(Keyword::Annotation(a)) => {
				next_token(lexer)?;
				annotations.push(Loc::new(a, loc.clone()));
				while let Loc(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
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

		let expr = LayoutExpr::parse_from_continuation(lexer, k)?;

		while let locspan::Loc(Some(Token::Keyword(Keyword::Annotation(a))), a_loc) =
			peek_token(lexer)?
		{
			loc.span_mut().append(a_loc.span());
			annotations.push(Loc::new(*a, a_loc));
			next_token(lexer)?;
		}

		Ok(Loc::new(Self { expr, annotations }, loc))
	}
}

impl<F: Clone> Parse<F> for LayoutExpr<F> {
	const FIRST: &'static [TokenKind] = &[TokenKind::Id, TokenKind::Punct(Punct::Ampersand)];

	fn parse_from<L: Tokens<F>>(
		lexer: &mut L,
		token: Token,
		mut loc: Location<F>,
	) -> Result<Loc<Self, F>, Loc<Error<L::Error>, F>> {
		match token {
			Token::Id(id) => Ok(Loc::new(Self::Id(Loc::new(id, loc.clone())), loc)),
			Token::Punct(lexing::Punct::Ampersand) => {
				let arg = Self::parse(lexer)?;
				loc.span_mut().append(arg.span());
				Ok(Loc::new(Self::Reference(Box::new(arg)), loc))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), Self::FIRST.to_vec()),
				loc,
			)),
		}
	}
}
