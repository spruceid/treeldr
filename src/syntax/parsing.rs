use super::*;
use crate::{source, Source};
use lexing::{Token, TokenKind};
use std::{fmt, fmt::Debug, iter::Peekable};

pub enum Error<E: Debug> {
	Unexpected(Option<Token>, Vec<lexing::TokenKind>),
	InvalidAlias(Id),
	Lexer(E),
}

impl<E: Debug + fmt::Display> crate::error::Diagnose for Loc<Error<E>> {
	fn message(&self) -> String {
		match self.inner() {
			Error::Unexpected(_, _) => "parsing error".to_owned(),
			Error::InvalidAlias(_) => "invalid alias".to_owned(),
			Error::Lexer(_) => "lexing error".to_owned(),
		}
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<source::Id>> {
		vec![codespan_reporting::diagnostic::Label::primary(
			self.source().file(),
			self.source().span(),
		)
		.with_message(self.to_string())]
	}

	fn notes(&self) -> Vec<String> {
		if let Error::Unexpected(_, expected) = self.inner() {
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
			Self::Lexer(e) => write!(f, "lexer error: {}", e),
		}
	}
}

/// Parsable abstract syntax nodes.
pub trait Parse: Sized {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>>;
}

fn peek_token<'l, E: 'l + Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
	lexer: &'l mut Peekable<L>,
) -> Result<Option<&'l Loc<Token>>, Loc<Error<E>>> {
	match lexer.peek_mut() {
		None => Ok(None),
		Some(Ok(token)) => Ok(Some(token)),
		Some(e) => {
			let source = e.as_ref().err().unwrap().source();
			// replace the next item with a dummy token to be able to get the actual error.
			let mut result = Ok(Loc::new(Token::Doc(String::new()), source));
			std::mem::swap(e, &mut result);
			Err(result.err().unwrap().map(Error::Lexer))
		},
	}
}

fn consume_token<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
	lexer: &mut Peekable<L>,
) -> Result<Option<Loc<Token>>, Loc<Error<E>>> {
	match lexer.next() {
		None => Ok(None),
		Some(Ok(token)) => Ok(Some(token)),
		Some(Err(e)) => Err(e.map(Error::Lexer)),
	}
}

// fn peek_expected_token<E: Debug, L: Iterator<Item=Result<Loc<Token>, Loc<E>>>>(file: source::Id, lexer: &mut Peekable<L>, start: usize) -> Result<Loc<Token>, Loc<Error<E>>> {
// 	match lexer.peek() {
// 		None => Err(Loc::new(Error::Unexpected(None), start.into())),
// 		Some(Ok(token)) => return Ok(token.clone()),
// 		Some(Err(_)) => Err(consume_token(lexer).unwrap_err())
// 	}
// }

fn consume_expected_token<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
	file: source::Id,
	lexer: &mut Peekable<L>,
	start: usize,
	expected: impl FnOnce() -> Vec<lexing::TokenKind>
) -> Result<Loc<Token>, Loc<Error<E>>> {
	match lexer.next() {
		None => Err(Loc::new(
			Error::Unexpected(None, expected()),
			Source::new(file, start.into()),
		)),
		Some(Ok(token)) => Ok(token),
		Some(Err(e)) => Err(e.map(Error::Lexer)),
	}
}

fn parse_comma_separated_list<
	E: Debug,
	L: Iterator<Item = Result<Loc<Token>, Loc<E>>>,
	T: Parse,
>(
	file: source::Id,
	lexer: &mut Peekable<L>,
	start: usize,
) -> Result<Vec<Loc<T>>, Loc<Error<E>>> {
	let mut list = Vec::new();
	let mut end = start;

	loop {
		if !list.is_empty() {
			match consume_token(lexer)? {
				Some(token) => match token.inner() {
					Token::Punct(lexing::Punct::Comma) => {
						end = token.span().end();
					}
					_ => return Err(token.map(|token| Error::Unexpected(Some(token), vec![TokenKind::Punct(lexing::Punct::Comma)]))),
				},
				None => break,
			}
		}

		match peek_token(lexer)? {
			Some(_) => {
				let item = T::parse(file, lexer, end)?;
				end = item.span().end();
				list.push(item);
			}
			None => break,
		}
	}

	Ok(list)
}

fn parse_block<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>, T: Parse>(
	file: source::Id,
	lexer: &mut Peekable<L>,
	start: usize,
) -> Result<Loc<Vec<Loc<T>>>, Loc<Error<E>>> {
	let (token, span) = consume_expected_token(file, lexer, start, || vec![TokenKind::Block])?.into_parts();

	match token {
		Token::Block(lexing::Delimiter::Brace, tokens) => {
			let mut block_lexer = tokens.into_iter().map(Ok).peekable();
			let items = parse_comma_separated_list::<E, _, _>(file, &mut block_lexer, start)?;
			Ok(Loc::new(items, span))
		}
		unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Block]), span)),
	}
}

fn parse_keyword<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
	file: source::Id,
	lexer: &mut Peekable<L>,
	start: usize,
	keyword: lexing::Keyword
) -> Result<(), Loc<Error<E>>> {
	let (token, span) = consume_expected_token(file, lexer, start, || vec![TokenKind::Keyword(keyword)])?.into_parts();

	match token {
		Token::Keyword(k) if k == keyword => {
			Ok(())
		}
		unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Keyword(keyword)]), span)),
	}
}

// impl<T: Parse> Parse for Option<T> {
// 	fn parse<E: Debug, L: Iterator<Item=Result<Loc<Token>, Loc<E>>>>(file: source::Id, lexer: &mut Peekable<L>, start: usize) -> Result<Loc<Self>, Loc<Error<E>>> {
// 		match lexer.peek() {
// 			None => Ok(Loc::new(None, start.into())),
// 			Some(Ok(_)) => Ok(T::parse(lexer, start)?.map(Option::Some)),
// 			Some(Err(_)) => Err(lexer.next().unwrap().unwrap_err().map(Error::Lexer))
// 		}
// 	}
// }

impl Parse for Document {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let mut span: Span = start.into();
		let mut items = Vec::new();

		while peek_token(lexer)?.is_some() {
			let item = Item::parse(file, lexer, span.end())?;
			span.set_end(item.span().end());
			items.push(item)
		}

		Ok(Loc::new(Self { items }, Source::new(file, span)))
	}
}

impl Parse for Id {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let (token, source) = consume_expected_token(file, lexer, start, || vec![TokenKind::Id])?.into_parts();
		match token {
			Token::Id(id) => Ok(Loc::new(id, source)),
			unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id]), source)),
		}
	}
}

impl Documentation {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(lexer: &mut Peekable<L>) -> Result<Self, Loc<Error<E>>> {
		let mut items = Vec::new();
		while peek_token(lexer)?.map(|token| token.is_doc()).unwrap_or(false) {
			items.push(consume_token(lexer)?.unwrap().map(|t| t.into_doc().unwrap()))
		}

		Ok(Self::new(items))
	}
}

impl Parse for Item {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let doc = Documentation::parse(lexer)?;
		let (token, source) = consume_expected_token(file, lexer, start, || vec![TokenKind::Keyword(lexing::Keyword::Type), TokenKind::Keyword(lexing::Keyword::Layout)])?.into_parts();
		let mut span = source.span();
		match token {
			Token::Keyword(lexing::Keyword::Type) => {
				let id = Id::parse(file, lexer, span.end())?;
				let (properties, prop_source) =
					parse_block(file, lexer, id.span().end())?.into_parts();
				span.set_end(prop_source.span().end());
				Ok(Loc::new(
					Item::Type(
						Loc::new(
							TypeDefinition { id, properties, doc },
							Source::new(file, span)
						)
					),
					Source::new(file, span),
				))
			}
			Token::Keyword(lexing::Keyword::Layout) => {
				let id = Id::parse(file, lexer, span.end())?;
				parse_keyword(file, lexer, span.end(), lexing::Keyword::For)?;
				let ty_id = Id::parse(file, lexer, span.end())?;
				let (fields, field_source) =
					parse_block(file, lexer, id.span().end())?.into_parts();
				span.set_end(field_source.span().end());
				Ok(Loc::new(
					Item::Layout(
						Loc::new(
							LayoutDefinition { id, ty_id, fields, doc },
							Source::new(file, span)
						)
					),
					Source::new(file, span),
				))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected), vec![TokenKind::Keyword(lexing::Keyword::Type), TokenKind::Keyword(lexing::Keyword::Layout)]),
				Source::new(file, span),
			)),
		}
	}
}

impl Parse for PropertyDefinition {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let doc = Documentation::parse(lexer)?;
		let mut span: Span = start.into();

		let id = Id::parse(file, lexer, span.end())?;
		span = id.span();

		let ty = match peek_token(lexer)? {
			Some(token) => {
				if let Token::Punct(lexing::Punct::Colon) = token.inner() {
					consume_token(lexer)?;
					let ty = TypeExpr::parse(file, lexer, span.end())?;
					span.set_end(ty.span().end());
					Some(ty)
				} else {
					None
				}
			},
			None => None,
		};

		Ok(Loc::new(Self { id, ty, doc }, Source::new(file, span)))
	}
}

impl Parse for FieldDefinition {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let doc = Documentation::parse(lexer)?;
		let mut span: Span = start.into();

		let id = Id::parse(file, lexer, span.end())?;
		span = id.span();

		let alias = match peek_token(lexer)? {
			Some(token) => match token.parts() {
				(Token::Keyword(lexing::Keyword::As), as_source) => {
					consume_token(lexer)?;
					span.set_end(as_source.span().end());
					let alias = Alias::parse(file, lexer, span.end())?;
					span.set_end(alias.span().end());
					Some(alias)
				}
				_ => None,
			},
			None => None,
		};

		let (token, token_source) = consume_expected_token(file, lexer, span.end(), || vec![TokenKind::Punct(lexing::Punct::Colon)])?.into_parts();
		let layout = match token {
			Token::Punct(lexing::Punct::Colon) => {
				let layout = LayoutExpr::parse(file, lexer, span.end())?;
				span.set_end(layout.span().end());
				layout
			}
			unexpected => return Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Punct(lexing::Punct::Colon)]), token_source))
		};

		// NOTE: if someday we have default layouts, to parse optional layout exprs.
		// let layout = match peek_token(lexer)? {
		// 	Some(token) => {
		// 		if let (Token::Punct(lexing::Punct::Colon), _) = token.parts() {
		// 			consume_token(lexer)?;
		// 			let layout = LayoutExpr::parse(file, lexer, span.end())?;
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
			Source::new(file, span),
		))
	}
}

impl Parse for Alias {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let (token, source) = consume_expected_token(file, lexer, start, || vec![TokenKind::Id])?.into_parts();
		match token {
			Token::Id(Id::IriRef(iri_ref)) => {
				if iri_ref.scheme().is_none() && iri_ref.authority().is_none() && iri_ref.query().is_none() && iri_ref.fragment().is_none() && iri_ref.path().len() == 1 && iri_ref.path().is_closed() {
					Ok(Loc::new(Alias(iri_ref.path().as_str().to_owned()), source))
				} else {
					Err(Loc::new(Error::InvalidAlias(Id::IriRef(iri_ref)), source))
				}
			}
			Token::Id(id @ Id::Compact(_, _)) => Err(Loc::new(Error::InvalidAlias(id), source)),
			unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected), vec![TokenKind::Id]), source)),
		}
	}
}

impl Parse for TypeExpr {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let mut span: Span = start.into();

		let ty = Id::parse(file, lexer, span.end())?;
		span = ty.span();

		let args = match peek_token(lexer)? {
			Some(token) => match token.parts() {
				(Token::Block(lexing::Delimiter::Parenthesis, _), args_source) => {
					let (_, tokens) = consume_token(lexer)?.unwrap().into_inner().into_block().unwrap();
					let mut block_lexer = tokens.into_iter().map(Ok).peekable();
					let items =
						parse_comma_separated_list::<E, _, _>(file, &mut block_lexer, start)?;
					span.set_end(args_source.span().end());
					items
				}
				_ => Vec::new(),
			},
			None => Vec::new(),
		};

		Ok(Loc::new(Self { ty, args }, Source::new(file, span)))
	}
}

impl Parse for LayoutExpr {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let mut span: Span = start.into();

		let layout = Id::parse(file, lexer, span.end())?;
		span = layout.span();

		let args = match peek_token(lexer)? {
			Some(token) => match token.parts() {
				(Token::Block(lexing::Delimiter::Parenthesis, _), args_source) => {
					let (_, tokens) = consume_token(lexer)?.unwrap().into_inner().into_block().unwrap();
					let mut block_lexer = tokens.into_iter().map(Ok).peekable();
					let items =
						parse_comma_separated_list::<E, _, _>(file, &mut block_lexer, start)?;
					span.set_end(args_source.span().end());
					items
				}
				_ => Vec::new(),
			},
			None => Vec::new(),
		};

		Ok(Loc::new(Self { layout, args }, Source::new(file, span)))
	}
}
