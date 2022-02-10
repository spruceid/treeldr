use super::*;
use crate::source;
use lexing::{Keyword, Token, TokenKind, Tokens};
use locspan::MapLocErr;
use std::{fmt, fmt::Debug};

pub enum Error<E: Debug> {
	Unexpected(Option<Token>, Vec<lexing::TokenKind>),
	InvalidImportId(Id),
	InvalidPrefix(Id),
	InvalidAlias(Id),
	Lexer(E),
}

impl<E: Debug + fmt::Display> crate::error::Diagnose for Loc<Error<E>> {
	fn message(&self) -> String {
		match self.value() {
			Error::Unexpected(_, _) => "parsing error".to_owned(),
			Error::InvalidImportId(_) => "invalid import IRI".to_owned(),
			Error::InvalidPrefix(_) => "invalid prefix".to_owned(),
			Error::InvalidAlias(_) => "invalid alias".to_owned(),
			Error::Lexer(_) => "lexing error".to_owned(),
		}
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<source::Id>> {
		vec![codespan_reporting::diagnostic::Label::primary(
			*self.location().file(),
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
pub trait Parse: Sized {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>>;
}

fn peek_token<L: Tokens>(tokens: &mut L) -> Result<Loc<Option<&Token>>, Loc<Error<L::Error>>> {
	tokens.peek().map_loc_err(Error::Lexer)
}

fn next_token<L: Tokens>(tokens: &mut L) -> Result<Loc<Option<Token>>, Loc<Error<L::Error>>> {
	tokens.next().map_loc_err(Error::Lexer)
}

fn next_expected_token<L: Tokens>(
	tokens: &mut L,
	expected: impl FnOnce() -> Vec<TokenKind>,
) -> Result<Loc<Token>, Loc<Error<L::Error>>> {
	match next_token(tokens)? {
		locspan::Loc(None, loc) => Err(Loc::new(Error::Unexpected(None, expected()), loc)),
		locspan::Loc(Some(token), loc) => Ok(Loc::new(token, loc)),
	}
}

fn parse_comma_separated_list<L: Tokens, T: Parse>(
	tokens: &mut L,
) -> Result<Vec<Loc<T>>, Loc<Error<L::Error>>> {
	let mut list = Vec::new();

	loop {
		if !list.is_empty() {
			match next_token(tokens)? {
				locspan::Loc(Some(Token::Punct(lexing::Punct::Comma)), _) => {
					// ...
				}
				locspan::Loc(Some(unexpected), loc) => {
					return Err(Loc::new(
						Error::Unexpected(
							Some(unexpected),
							vec![TokenKind::Punct(lexing::Punct::Comma)],
						),
						loc,
					))
				}
				locspan::Loc(None, _) => break,
			}
		}

		if peek_token(tokens)?.is_some() {
			let item = T::parse(tokens)?;
			list.push(item);
		} else {
			break;
		}
	}

	Ok(list)
}

#[allow(clippy::type_complexity)]
fn parse_block<L: Tokens, T: Parse>(
	tokens: &mut L,
) -> Result<Loc<Vec<Loc<T>>>, Loc<Error<L::Error>>> {
	let locspan::Loc(token, span) = next_expected_token(tokens, || vec![TokenKind::Block])?;

	match token {
		Token::Block(lexing::Delimiter::Brace, tokens) => {
			let mut block_tokens = tokens.into_tokens(span);
			let items = parse_comma_separated_list(&mut block_tokens)?;
			Ok(Loc::new(items, span))
		}
		unexpected => Err(Loc::new(
			Error::Unexpected(Some(unexpected), vec![TokenKind::Block]),
			span,
		)),
	}
}

fn parse_keyword<L: Tokens>(
	tokens: &mut L,
	keyword: lexing::Keyword,
) -> Result<(), Loc<Error<L::Error>>> {
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

impl Parse for Document {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let mut imports = Vec::new();
		let mut types = Vec::new();
		let mut layouts = Vec::new();

		let mut span = Span::default();
		loop {
			match peek_token(tokens)? {
				locspan::Loc(Some(_), _) => {
					let item = Item::parse(tokens)?;
					span.set_end(item.span().end());

					match item.into_value() {
						Item::Import(i) => imports.push(i),
						Item::Type(t) => types.push(t),
						Item::Layout(l) => layouts.push(l),
					}
				}
				locspan::Loc(None, loc) => {
					span.append(loc.span());
					break Ok(Loc::new(
						Self {
							imports,
							types,
							layouts,
						},
						Location::new(*loc.file(), span),
					));
				}
			}
		}
	}
}

impl Parse for Id {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let locspan::Loc(token, source) = next_expected_token(tokens, || vec![TokenKind::Id])?;
		match token {
			Token::Id(id) => Ok(Loc::new(id, source)),
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Id]),
				source,
			)),
		}
	}
}

impl Parse for Documentation {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let mut items = Vec::new();

		let mut span = Span::default();
		loop {
			match peek_token(tokens)? {
				locspan::Loc(Some(token), loc) if token.is_doc() => {
					let doc = next_token(tokens)?.unwrap().map(|t| t.into_doc().unwrap());
					span.set_end(loc.span().end());
					items.push(doc)
				}
				locspan::Loc(_, loc) => {
					span.append(loc.span().start().into());
					break Ok(Loc::new(Self::new(items), Location::new(*loc.file(), span)));
				}
			}
		}
	}
}

impl Parse for Item {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let doc = Documentation::parse(tokens)?;

		let locspan::Loc(token, source) = next_expected_token(tokens, || {
			vec![
				TokenKind::Keyword(lexing::Keyword::Type),
				TokenKind::Keyword(lexing::Keyword::Layout),
			]
		})?;
		let mut span = source.span();
		match token {
			Token::Keyword(lexing::Keyword::Import) => {
				let id = Id::parse(tokens)?;
				let iri = match id {
					locspan::Loc(Id::IriRef(iri_ref), loc) => match IriBuf::try_from(iri_ref) {
						Ok(iri) => Loc::new(iri, loc),
						Err(iri_ref) => {
							return Err(Loc::new(Error::InvalidImportId(Id::IriRef(iri_ref)), loc))
						}
					},
					locspan::Loc(id, loc) => return Err(Loc::new(Error::InvalidImportId(id), loc)),
				};

				parse_keyword(tokens, lexing::Keyword::As)?;

				let prefix = Prefix::parse(tokens)?;

				Ok(Loc::new(
					Item::Import(Loc::new(
						Import { iri, prefix, doc },
						Location::new(*source.file(), span),
					)),
					Location::new(*source.file(), span),
				))
			}
			Token::Keyword(lexing::Keyword::Type) => {
				let id = Id::parse(tokens)?;
				let locspan::Loc(properties, prop_source) = parse_block(tokens)?;
				span.set_end(prop_source.span().end());
				Ok(Loc::new(
					Item::Type(Loc::new(
						TypeDefinition {
							id,
							properties,
							doc,
						},
						Location::new(*source.file(), span),
					)),
					Location::new(*source.file(), span),
				))
			}
			Token::Keyword(lexing::Keyword::Layout) => {
				let id = Id::parse(tokens)?;
				parse_keyword(tokens, lexing::Keyword::For)?;
				let ty_id = Id::parse(tokens)?;
				let locspan::Loc(fields, field_source) = parse_block(tokens)?;
				span.set_end(field_source.span().end());
				Ok(Loc::new(
					Item::Layout(Loc::new(
						LayoutDefinition {
							id,
							ty_id,
							fields,
							doc,
						},
						Location::new(*source.file(), span),
					)),
					Location::new(*source.file(), span),
				))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(
					Some(unexpected),
					vec![
						TokenKind::Keyword(lexing::Keyword::Type),
						TokenKind::Keyword(lexing::Keyword::Layout),
					],
				),
				Location::new(*source.file(), span),
			)),
		}
	}
}

impl Parse for PropertyDefinition {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let doc = Documentation::parse(tokens)?;

		let id = Id::parse(tokens)?;
		let mut span = id.span();
		let file = *id.file();

		let ty = match peek_token(tokens)?.into_value() {
			Some(Token::Punct(lexing::Punct::Colon)) => {
				next_token(tokens)?;
				let ty = AnnotatedTypeExpr::parse(tokens)?;
				span.set_end(ty.span().end());
				Some(ty)
			}
			_ => None,
		};

		Ok(Loc::new(Self { id, ty, doc }, Location::new(file, span)))
	}
}

impl Parse for FieldDefinition {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let doc = Documentation::parse(tokens)?;

		let id = Id::parse(tokens)?;
		let mut span = id.span();
		let file = *id.file();

		let alias = match peek_token(tokens)? {
			locspan::Loc(Some(Token::Keyword(lexing::Keyword::As)), as_source) => {
				next_token(tokens)?;
				span.set_end(as_source.span().end());
				let alias = Alias::parse(tokens)?;
				span.set_end(alias.span().end());
				Some(alias)
			}
			_ => None,
		};

		let locspan::Loc(token, token_source) =
			next_expected_token(tokens, || vec![TokenKind::Punct(lexing::Punct::Colon)])?;
		let layout = match token {
			Token::Punct(lexing::Punct::Colon) => {
				let layout = AnnotatedLayoutExpr::parse(tokens)?;
				span.set_end(layout.span().end());
				layout
			}
			unexpected => {
				return Err(Loc::new(
					Error::Unexpected(
						Some(unexpected),
						vec![TokenKind::Punct(lexing::Punct::Colon)],
					),
					token_source,
				))
			}
		};

		// NOTE: if someday we have default layouts, to parse optional layout exprs.
		// let layout = match peek_token(tokens)? {
		// 	Some(token) => {
		// 		if let (Token::Punct(lexing::Punct::Colon), _) = token.parts() {
		// 			next_token(tokens)?;
		// 			let layout = LayoutExpr::parse(tokens)?;
		// 			span.set_end(layout.span().end());
		// 			Some(layout)
		// 		} else {
		// 			None
		// 		}
		// 	},
		// 	None => None,
		// };

		Ok(Loc::new(
			Self {
				id,
				layout,
				alias,
				doc,
			},
			Location::new(file, span),
		))
	}
}

impl Parse for Alias {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		match next_expected_token(tokens, || vec![TokenKind::Id])? {
			locspan::Loc(Token::Id(Id::Name(alias)), source) => Ok(Loc::new(Alias(alias), source)),
			locspan::Loc(Token::Id(id), source) => Err(Loc::new(Error::InvalidAlias(id), source)),
			locspan::Loc(unexpected, source) => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Id]),
				source,
			)),
		}
	}
}

impl Parse for Prefix {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		match next_expected_token(tokens, || vec![TokenKind::Id])? {
			locspan::Loc(Token::Id(Id::Name(alias)), source) => Ok(Loc::new(Prefix(alias), source)),
			locspan::Loc(Token::Id(id), source) => Err(Loc::new(Error::InvalidPrefix(id), source)),
			locspan::Loc(unexpected, source) => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Id]),
				source,
			)),
		}
	}
}

impl Parse for AnnotatedTypeExpr {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let mut annotations = Vec::new();

		while let locspan::Loc(Some(Token::Keyword(Keyword::Annotation(a))), loc) =
			peek_token(tokens)?
		{
			annotations.push(Loc::new(*a, loc));
			next_token(tokens)?;
		}

		let expr = TypeExpr::parse(tokens)?;
		let mut span = expr.span();

		if let Some(a) = annotations.first() {
			span.set_start(a.span().start())
		}

		let file = *expr.file();

		while let locspan::Loc(Some(Token::Keyword(Keyword::Annotation(a))), loc) =
			peek_token(tokens)?
		{
			annotations.push(Loc::new(*a, loc));
			span.append(loc.span());
			next_token(tokens)?;
		}

		Ok(Loc::new(
			Self { expr, annotations },
			Location::new(file, span),
		))
	}
}

impl Parse for TypeExpr {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		match next_token(tokens)? {
			locspan::Loc(Some(Token::Id(id)), source) => {
				Ok(Loc::new(Self::Id(Loc::new(id, source)), source))
			}
			locspan::Loc(Some(Token::Punct(lexing::Punct::Ampersand)), mut source) => {
				let arg = Self::parse(tokens)?;
				source.span_mut().set_end(arg.span().end());
				Ok(Loc::new(Self::Reference(Box::new(arg)), source))
			}
			locspan::Loc(unexpected, source) => Err(Loc::new(
				Error::Unexpected(
					unexpected,
					vec![
						TokenKind::Id,
						TokenKind::Punct(lexing::Punct::Ampersand)
					],
				),
				source,
			))
		}
	}
}

impl Parse for AnnotatedLayoutExpr {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let mut annotations = Vec::new();

		while let locspan::Loc(Some(Token::Keyword(Keyword::Annotation(a))), loc) =
			peek_token(tokens)?
		{
			annotations.push(Loc::new(*a, loc));
			next_token(tokens)?;
		}

		let expr = LayoutExpr::parse(tokens)?;
		let mut span = expr.span();

		if let Some(a) = annotations.first() {
			span.set_start(a.span().start())
		}

		let file = *expr.file();

		while let locspan::Loc(Some(Token::Keyword(Keyword::Annotation(a))), loc) =
			peek_token(tokens)?
		{
			annotations.push(Loc::new(*a, loc));
			span.append(loc.span());
			next_token(tokens)?;
		}

		Ok(Loc::new(
			Self { expr, annotations },
			Location::new(file, span),
		))
	}
}

impl Parse for LayoutExpr {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		match next_token(tokens)? {
			locspan::Loc(Some(Token::Id(id)), source) => {
				Ok(Loc::new(Self::Id(Loc::new(id, source)), source))
			}
			locspan::Loc(Some(Token::Punct(lexing::Punct::Ampersand)), mut source) => {
				let arg = Self::parse(tokens)?;
				source.span_mut().set_end(arg.span().end());
				Ok(Loc::new(Self::Reference(Box::new(arg)), source))
			}
			locspan::Loc(unexpected, source) => Err(Loc::new(
				Error::Unexpected(
					unexpected,
					vec![
						TokenKind::Id,
						TokenKind::Punct(lexing::Punct::Ampersand)
					],
				),
				source,
			))
		}
	}
}
