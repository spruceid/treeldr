use super::*;
use crate::{source, Source};
use lexing::Token;
use std::{fmt, fmt::Debug, iter::Peekable};

pub enum Error<E: Debug> {
	Unexpected(Option<Token>),
	Lexer(E),
}

impl<E: Debug + fmt::Display> crate::error::Diagnose for Loc<Error<E>> {
	fn message(&self) -> String {
		match self.inner() {
			Error::Unexpected(_) => "parsing error".to_owned(),
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
}

impl<E: Debug + fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Unexpected(None) => write!(f, "unexpected end of text"),
			Self::Unexpected(Some(token)) => write!(f, "unexpected {}", token),
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

fn peek_token<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
	lexer: &mut Peekable<L>,
) -> Result<Option<Loc<Token>>, Loc<Error<E>>> {
	match lexer.peek() {
		None => Ok(None),
		Some(Ok(token)) => Ok(Some(token.clone())),
		Some(Err(_)) => Err(consume_token(lexer).unwrap_err()),
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
) -> Result<Loc<Token>, Loc<Error<E>>> {
	match lexer.next() {
		None => Err(Loc::new(
			Error::Unexpected(None),
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
					_ => return Err(token.map(|token| Error::Unexpected(Some(token)))),
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
	let (token, span) = consume_expected_token(file, lexer, start)?.into_parts();

	match token {
		Token::Block(lexing::Delimiter::Brace, tokens) => {
			let mut block_lexer = tokens.into_iter().map(Ok).peekable();
			let items = parse_comma_separated_list::<E, _, _>(file, &mut block_lexer, start)?;
			Ok(Loc::new(items, span))
		}
		unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected)), span)),
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
		let (token, source) = consume_expected_token(file, lexer, start)?.into_parts();
		match token {
			Token::Id(id) => Ok(Loc::new(id, source)),
			unexpected => Err(Loc::new(Error::Unexpected(Some(unexpected)), source)),
		}
	}
}

impl Parse for Item {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let (token, source) = consume_expected_token(file, lexer, start)?.into_parts();
		let mut span = source.span();
		match token {
			Token::Keyword(lexing::Keyword::Type) => {
				let id = Id::parse(file, lexer, span.end())?;
				let (properties, prop_source) =
					parse_block(file, lexer, id.span().end())?.into_parts();
				span.set_end(prop_source.span().end());
				Ok(Loc::new(
					Item::Type(TypeDefinition { id, properties }),
					Source::new(file, span),
				))
			}
			Token::Keyword(lexing::Keyword::Layout) => {
				let id = Id::parse(file, lexer, span.end())?;
				let (fields, field_source) =
					parse_block(file, lexer, id.span().end())?.into_parts();
				span.set_end(field_source.span().end());
				Ok(Loc::new(
					Item::Layout(LayoutDefinition { id, fields }),
					Source::new(file, span),
				))
			}
			unexpected => Err(Loc::new(
				Error::Unexpected(Some(unexpected)),
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
		let mut span: Span = start.into();

		let id = Id::parse(file, lexer, span.end())?;
		span = id.span();

		let ty = match peek_token(lexer)? {
			Some(token) => match token.into_parts() {
				(Token::Punct(lexing::Punct::Comma), _) => None,
				_ => {
					let ty = TypeExpr::parse(file, lexer, span.end())?;
					span.set_end(ty.span().end());
					Some(ty)
				}
			},
			None => None,
		};

		Ok(Loc::new(Self { id, ty }, Source::new(file, span)))
	}
}

impl Parse for FieldDefinition {
	fn parse<E: Debug, L: Iterator<Item = Result<Loc<Token>, Loc<E>>>>(
		file: source::Id,
		lexer: &mut Peekable<L>,
		start: usize,
	) -> Result<Loc<Self>, Loc<Error<E>>> {
		let mut span: Span = start.into();

		let id = Id::parse(file, lexer, span.end())?;
		span = id.span();

		let alias = match peek_token(lexer)? {
			Some(token) => match token.into_parts() {
				(Token::Keyword(lexing::Keyword::As), as_source) => {
					consume_token(lexer)?;
					span.set_end(as_source.span().end());
					let alias = Id::parse(file, lexer, span.end())?;
					span.set_end(alias.span().end());
					Some(alias)
				}
				_ => None,
			},
			None => None,
		};

		let layout = match peek_token(lexer)? {
			Some(token) => match token.into_parts() {
				(Token::Punct(lexing::Punct::Comma), _) => None,
				_ => {
					let layout = LayoutExpr::parse(file, lexer, span.end())?;
					span.set_end(layout.span().end());
					Some(layout)
				}
			},
			None => None,
		};

		Ok(Loc::new(
			Self { id, layout, alias },
			Source::new(file, span),
		))
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
			Some(token) => match token.into_parts() {
				(Token::Block(lexing::Delimiter::Parenthesis, tokens), args_source) => {
					consume_token(lexer)?;
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
			Some(token) => match token.into_parts() {
				(Token::Block(lexing::Delimiter::Parenthesis, tokens), args_source) => {
					consume_token(lexer)?;
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
