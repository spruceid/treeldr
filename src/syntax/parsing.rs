use super::*;
use crate::source;
use locspan::MapLocErr;
use lexing::{Tokens, Token, TokenKind};
use std::{fmt, fmt::Debug};

pub enum Error<E: Debug> {
	Unexpected(Option<Token>, Vec<lexing::TokenKind>),
	InvalidAlias(Id),
	Lexer(E),
}

impl<E: Debug + fmt::Display> crate::error::Diagnose for Loc<Error<E>> {
	fn message(&self) -> String {
		match self.value() {
			Error::Unexpected(_, _) => "parsing error".to_owned(),
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
				for (i, token) in expected.into_iter().enumerate() {
					if i > 0 {
						if i+1 == len {
							note.push_str("or ");
						} else {
							note.push_str(", ");
						}
					}
					
					note.push_str(&token.to_string())
				}
				
				return vec![note]
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

fn next_expected_token<L: Tokens>(tokens: &mut L, expected: impl FnOnce() -> Vec<TokenKind>) -> Result<Loc<Token>, Loc<Error<L::Error>>> {
	match next_token(tokens)? {
		locspan::Loc(None, loc) => Err(Loc::new(Error::Unexpected(None, expected()), loc)),
		locspan::Loc(Some(token), loc) => Ok(Loc::new(token, loc))
	}
}

fn parse_comma_separated_list<L: Tokens, T: Parse>(tokens: &mut L) -> Result<Vec<Loc<T>>, Loc<Error<L::Error>>> {
	let mut list = Vec::new();

	loop {
		if !list.is_empty() {
			match next_token(tokens)? {
				locspan::Loc(Some(Token::Punct(lexing::Punct::Comma)), _) => {
					// ...
				},
				locspan::Loc(Some(unexpected), loc) => {
					return Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Punct(lexing::Punct::Comma)]), loc))
				},
				locspan::Loc(None, _) => {
					break
				}
			}
		}

		if peek_token(tokens)?.is_some() {
			let item = T::parse(tokens)?;
			list.push(item);
		} else {
			break
		}
	}

	Ok(list)
}

fn parse_block<L: Tokens, T: Parse>(tokens: &mut L) -> Result<Loc<Vec<Loc<T>>>, Loc<Error<L::Error>>> {
	let locspan::Loc(token, span) = next_expected_token(tokens, || vec![TokenKind::Block])?;

	match token {
		Token::Block(lexing::Delimiter::Brace, tokens) => {
			let mut block_tokens = tokens.into_tokens(span);
			let items = parse_comma_separated_list(&mut block_tokens)?;
			Ok(Loc::new(items, span))
		}
		unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Block]), span)),
	}
}

fn parse_keyword<L: Tokens>(tokens: &mut L, keyword: lexing::Keyword) -> Result<(), Loc<Error<L::Error>>> {
	let locspan::Loc(token, span) = next_expected_token(tokens, || vec![TokenKind::Keyword(keyword)])?;

	match token {
		Token::Keyword(k) if k == keyword => {
			Ok(())
		}
		unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Keyword(keyword)]), span)),
	}
}

impl Parse for Document {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let mut items = Vec::new();

		let mut span = Span::default();
		loop {
			match peek_token(tokens)? {
				locspan::Loc(Some(_), _) => {
					let item = Item::parse(tokens)?;
					span.set_end(item.span().end());
					items.push(item)
				}
				locspan::Loc(None, loc) => {
					span.append(loc.span());
					break Ok(Loc::new(Self { items }, Location::new(*loc.file(), span)))
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
			unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id]), source)),
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
					break Ok(Loc::new(Self::new(items), Location::new(*loc.file(), span)))
				}
			}
		}
	}
}

impl Parse for Item {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let doc = Documentation::parse(tokens)?;

		let locspan::Loc(token, source) = next_expected_token(tokens, || vec![TokenKind::Keyword(lexing::Keyword::Type), TokenKind::Keyword(lexing::Keyword::Layout)])?;
		let mut span = source.span();
		match token {
			Token::Keyword(lexing::Keyword::Type) => {
				let id = Id::parse(tokens)?;
				let locspan::Loc(properties, prop_source) =
					parse_block(tokens)?;
				span.set_end(prop_source.span().end());
				Ok(Loc::new(
					Item::Type(
						Loc::new(
							TypeDefinition { id, properties, doc },
							Location::new(*source.file(), span)
						)
					),
					Location::new(*source.file(), span),
				))
			}
			Token::Keyword(lexing::Keyword::Layout) => {
				let id = Id::parse(tokens)?;
				parse_keyword(tokens, lexing::Keyword::For)?;
				let ty_id = Id::parse(tokens)?;
				let locspan::Loc(fields, field_source) =
					parse_block(tokens)?;
				span.set_end(field_source.span().end());
				Ok(Loc::new(
					Item::Layout(
						Loc::new(
							LayoutDefinition { id, ty_id, fields, doc },
							Location::new(*source.file(), span)
						)
					),
					Location::new(*source.file(), span),
				))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Keyword(lexing::Keyword::Type), TokenKind::Keyword(lexing::Keyword::Layout)]),
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
				let ty = TypeExpr::parse(tokens)?;
				span.set_end(ty.span().end());
				Some(ty)
			},
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
			},
			_ => None,
		};

		let locspan::Loc(token, token_source) = next_expected_token(tokens, || vec![TokenKind::Punct(lexing::Punct::Colon)])?;
		let layout = match token {
			Token::Punct(lexing::Punct::Colon) => {
				let layout = LayoutExpr::parse(tokens)?;
				span.set_end(layout.span().end());
				layout
			}
			unexpected => return Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Punct(lexing::Punct::Colon)]), token_source))
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
			Self { id, layout, alias, doc },
			Location::new(file, span),
		))
	}
}

impl Parse for Alias {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		match next_expected_token(tokens, || vec![TokenKind::Id])? {
			locspan::Loc(Token::Id(Id::Name(alias)), source) => {
				Ok(Loc::new(Alias(alias), source))
			}
			locspan::Loc(Token::Id(id), source) => Err(Loc::new(Error::InvalidAlias(id), source)),
			locspan::Loc(unexpected, source) => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id]), source)),
		}
	}
}

impl Parse for TypeExpr {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let ty = Id::parse(tokens)?;
		let mut span = ty.span();
		let file = *ty.file();

		let args = match peek_token(tokens)? {
			locspan::Loc(Some(Token::Block(lexing::Delimiter::Parenthesis, _)), args_source) => {
				let (_, tokens) = next_token(tokens)?.unwrap().into_value().into_block().unwrap();
				let mut block_tokens = tokens.into_tokens(args_source);
				let items =
					parse_comma_separated_list(&mut block_tokens)?;
				span.append(args_source.span());
				items
			},
			_ => Vec::new(),
		};

		Ok(Loc::new(Self { ty, args }, Location::new(file, span)))
	}
}

impl Parse for LayoutExpr {
	fn parse<L: Tokens>(tokens: &mut L) -> Result<Loc<Self>, Loc<Error<L::Error>>> {
		let layout = Id::parse(tokens)?;
		let mut span = layout.span();
		let file = *layout.file();

		let args = match peek_token(tokens)? {
			locspan::Loc(Some(Token::Block(lexing::Delimiter::Parenthesis, _)), args_source) => {
				let (_, tokens) = next_token(tokens)?.unwrap().into_value().into_block().unwrap();
				let mut block_tokens = tokens.into_tokens(args_source);
				let items =
					parse_comma_separated_list(&mut block_tokens)?;
				span.append(args_source.span());
				items
			},
			_ => Vec::new(),
		};

		Ok(Loc::new(Self { layout, args }, Location::new(file, span)))
	}
}
