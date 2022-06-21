use iref::IriBuf;
use litrs::Literal;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use std::path::PathBuf;
use syn::spanned::Spanned;
use thiserror::Error;

pub type GenContext<'a> = treeldr_rust_gen::Context<'a, treeldr_load::source::FileId>;

#[derive(Error, Debug)]
pub enum ParseError {
	#[error("expected a module definition")]
	NotAModule,

	#[error("missing IRI prefix")]
	MissingPrefixIri,

	#[error("unexpected prefix path argument")]
	UnexpectedPrefixPathArguments,

	#[error("unexpected token")]
	UnexpectedToken,

	#[error("missing argument")]
	MissingArgument,

	#[error("invalid IRI `{1}`: {0}")]
	InvalidIri(iref::Error, String),
}

pub type SpannedParseError = (ParseError, Span);

pub type GenError = treeldr_rust_gen::Error<treeldr_load::source::FileId>;

pub struct Inputs {
	list: Vec<Input>,
}

impl Inputs {
	pub fn from_stream(tokens: TokenStream) -> Result<Self, SpannedParseError> {
		let mut list = Vec::new();
		let mut tokens = tokens.into_iter();

		while let Some(token) = tokens.next() {
			let (s, span) = token_to_string(token)?;
			list.push(Input {
				filename: s.into(),
				span,
			});

			match tokens.next() {
				None => (),
				Some(TokenTree::Punct(p)) if p.as_char() == ',' => (),
				Some(token) => return Err((ParseError::UnexpectedToken, token.span())),
			}
		}

		Ok(Self { list })
	}

	fn iter(&self) -> std::slice::Iter<Input> {
		self.list.iter()
	}
}

impl<'a> IntoIterator for &'a Inputs {
	type Item = &'a Input;
	type IntoIter = std::slice::Iter<'a, Input>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl IntoIterator for Inputs {
	type Item = Input;
	type IntoIter = std::vec::IntoIter<Input>;

	fn into_iter(self) -> Self::IntoIter {
		self.list.into_iter()
	}
}

pub struct Input {
	pub filename: PathBuf,
	pub span: Span,
}

/// Rust+TreeLDR module.
pub struct Module {
	/// Attributes.
	attrs: Vec<syn::Attribute>,

	/// Visibility.
	vis: syn::Visibility,

	/// Identifier.
	ident: proc_macro2::Ident,

	/// Imported prefixes.
	prefixes: Vec<Prefix>,
}

impl Module {
	pub fn from_item(item: syn::Item) -> Result<Self, SpannedParseError> {
		match item {
			syn::Item::Mod(m) => {
				let mut prefixes = Vec::new();

				if let Some((_, items)) = m.content {
					for item in items {
						prefixes.push(Prefix::from_item(item)?)
					}
				}

				Ok(Self {
					attrs: m.attrs,
					vis: m.vis,
					ident: m.ident,
					prefixes,
				})
			}
			_ => Err((ParseError::NotAModule, item.span())),
		}
	}

	pub fn bind(&mut self, vocabulary: &treeldr_load::Vocabulary, context: &mut GenContext) {
		use std::collections::HashMap;
		let mut map = HashMap::new();

		for prefix in &mut self.prefixes {
			let module_ref = context.add_module(None, prefix.ident.clone());
			prefix.module = Some(module_ref);

			for (layout_ref, layout) in context.model().layouts() {
				if let treeldr::Id::Iri(term) = layout.id() {
					let iri = term.iri(vocabulary).unwrap();

					if iri
						.as_str()
						.strip_prefix(prefix.prefix_attrs.iri.0.as_str())
						.is_some()
					{
						map.insert(layout_ref, treeldr_rust_gen::ParentModule::Ref(module_ref));
					}
				}
			}
		}

		for (layout_ref, _) in context.model().layouts() {
			context.add_layout(
				map.get(&layout_ref)
					.cloned()
					.or(Some(treeldr_rust_gen::ParentModule::Extern)),
				layout_ref,
			)
		}
	}

	pub fn generate(&self, context: &GenContext) -> Result<TokenStream, GenError> {
		let attrs = &self.attrs;
		let vis = &self.vis;
		let ident = &self.ident;
		let mut content = TokenStream::new();

		for prefix in &self.prefixes {
			content.extend(prefix.generate(context)?);
		}

		Ok(quote! {
			#(#attrs)*
			#vis mod #ident {
				#content
			}
		})
	}
}

/// Imported prefix.
pub struct Prefix {
	/// Attributes.
	attrs: Vec<syn::Attribute>,

	/// Visibility.
	vis: syn::Visibility,

	/// Identifier.
	ident: proc_macro2::Ident,

	/// Prefix specific attributes.
	prefix_attrs: PrefixAttributes,

	/// Module content.
	content: Vec<syn::Item>,

	module: Option<treeldr_rust_gen::Ref<treeldr_rust_gen::Module<treeldr_load::source::FileId>>>,
}

pub struct PrefixAttributes {
	/// Imported IRI prefix.
	iri: (IriBuf, Span),
}

fn is_prefix_path(path: &syn::Path) -> bool {
	path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == "prefix"
}

fn literal_to_string(lit: proc_macro2::Literal) -> Result<(String, Span), SpannedParseError> {
	let span = lit.span();
	let lit: Literal<_> = lit.into();
	match lit {
		Literal::String(s) => Ok((s.into_value().into_owned(), span)),
		_ => Err((ParseError::UnexpectedToken, span)),
	}
}

fn token_to_string(token: TokenTree) -> Result<(String, Span), SpannedParseError> {
	match token {
		TokenTree::Literal(lit) => literal_to_string(lit),
		token => Err((ParseError::UnexpectedToken, token.span())),
	}
}

fn expect_string_literal(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span,
) -> Result<(String, Span), SpannedParseError> {
	match tokens.next() {
		Some(token) => token_to_string(token),
		None => Err((ParseError::MissingArgument, span)),
	}
}

fn expect_iri_literal(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span,
) -> Result<(IriBuf, Span), SpannedParseError> {
	let (s, span) = expect_string_literal(tokens, span)?;

	match IriBuf::from_string(s) {
		Ok(iri) => Ok((iri, span)),
		Err((e, s)) => Err((ParseError::InvalidIri(e, s), span)),
	}
}

impl PrefixAttributes {
	pub fn from_attributes(
		attrs: &mut Vec<syn::Attribute>,
		span: Span,
	) -> Result<Self, SpannedParseError> {
		let mut input_attrs = Vec::new();
		std::mem::swap(&mut input_attrs, attrs);

		let mut iri = None;

		for attr in input_attrs {
			if is_prefix_path(&attr.path) {
				if attr.path.segments[0].arguments.is_empty() {
					let mut tokens = attr.tokens.into_iter();
					match tokens.next() {
						Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
							iri = Some(expect_iri_literal(&mut tokens, attr.path.span())?);
						}
						Some(TokenTree::Group(g))
							if g.delimiter() == proc_macro2::Delimiter::Parenthesis =>
						{
							let mut tokens = g.stream().into_iter();
							iri = Some(expect_iri_literal(&mut tokens, attr.path.span())?);
						}
						Some(token) => return Err((ParseError::UnexpectedToken, token.span())),
						None => return Err((ParseError::MissingArgument, attr.path.span())),
					}
				} else {
					return Err((
						ParseError::UnexpectedPrefixPathArguments,
						attr.path.segments[0].arguments.span(),
					));
				}
			} else {
				attrs.push(attr)
			}
		}

		Ok(Self {
			iri: iri.ok_or((ParseError::MissingPrefixIri, span))?,
		})
	}
}

impl Prefix {
	pub fn from_item(item: syn::Item) -> Result<Self, SpannedParseError> {
		match item {
			syn::Item::Mod(mut m) => {
				let span = m.span();
				let prefix_attrs = PrefixAttributes::from_attributes(&mut m.attrs, span)?;

				Ok(Self {
					attrs: m.attrs,
					vis: m.vis,
					ident: m.ident,
					prefix_attrs,
					content: m.content.map(|(_, items)| items).unwrap_or_default(),
					module: None,
				})
			}
			_ => Err((ParseError::NotAModule, item.span())),
		}
	}

	pub fn generate(&self, context: &GenContext) -> Result<TokenStream, GenError> {
		use treeldr_rust_gen::Generate;
		let attrs = &self.attrs;
		let vis = &self.vis;
		let ident = &self.ident;
		let module_ref = self.module.unwrap();
		let module = context.module(module_ref).unwrap();
		let generated = module.with(context, Some(module_ref)).into_tokens()?;
		let rest = &self.content;

		Ok(quote! {
			#(#attrs)*
			#vis mod #ident {
				#generated
				#(#rest)*
			}
		})
	}
}
