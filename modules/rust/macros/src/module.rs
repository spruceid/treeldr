use iref::IriBuf;
use litrs::Literal;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use rdf_types::Vocabulary;
use std::path::PathBuf;
use syn::spanned::Spanned;
use thiserror::Error;
use treeldr::{BlankIdIndex, IriIndex, TId};
use treeldr_rust_gen::{tr::TraitModules, DedicatedSubModule, ModulePathBuilder};

pub type GenContext<'a, V> = treeldr_rust_gen::Context<'a, V, treeldr_load::Metadata>;

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

pub type GenError = treeldr_rust_gen::Error;

pub struct Inputs {
	list: Vec<Input>,
	no_rdf: bool,
}

impl Inputs {
	pub fn from_stream(tokens: TokenStream) -> Result<Self, SpannedParseError> {
		let mut list = Vec::new();
		let mut no_rdf = false;
		let mut tokens = tokens.into_iter();

		while let Some(token) = tokens.next() {
			match token {
				TokenTree::Ident(id) if id == "no_rdf" => no_rdf = true,
				token => {
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
			}
		}

		Ok(Self { list, no_rdf })
	}

	pub fn no_rdf(&self) -> bool {
		self.no_rdf
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

	pub fn bind<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &V,
		context: &mut GenContext<V>,
	) {
		use std::collections::HashMap;
		let mut type_map = HashMap::new();
		let mut layout_map = HashMap::new();

		for prefix in &mut self.prefixes {
			let module_ref = context.add_module(
				None,
				prefix.extern_path.clone(),
				prefix.ident.clone(),
				prefix.vis.clone(),
			);

			let mut sub_modules = ModulePathBuilder::new(module_ref);

			prefix.module = Some(module_ref);

			for (id, node) in context.model().nodes() {
				if let treeldr::Id::Iri(term) = id {
					let iri = vocabulary.iri(&term).unwrap();

					if let Some(suffix) = iri
						.as_str()
						.strip_prefix(prefix.prefix_attrs.iri.0.as_str())
					{
						let path = treeldr_rust_gen::ModulePathBuilder::split_iri_path(suffix).0;

						if node.is_type() {
							type_map.insert(
								TId::new(id),
								TraitModules {
									main: Some(treeldr_rust_gen::module::Parent::Ref(
										sub_modules.get(context, path, None),
									)),
									provider: Some(treeldr_rust_gen::module::Parent::Ref(
										sub_modules.get(
											context,
											path,
											Some(DedicatedSubModule::ClassProviders),
										),
									)),
								},
							);
						}

						if node.is_layout() {
							let sub_module = context
								.options()
								.impl_rdf
								.then_some(DedicatedSubModule::Layouts);
							layout_map.insert(
								TId::new(id),
								treeldr_rust_gen::module::Parent::Ref(
									sub_modules.get(context, path, sub_module),
								),
							);
						}
					}
				}
			}
		}

		for (id, node) in context.model().nodes() {
			if node.is_type() {
				let type_ref = TId::new(id);
				context.add_type(
					type_map.get(&type_ref).cloned().unwrap_or_default(),
					type_ref,
				);
			}

			if node.is_layout() {
				let layout_ref = TId::new(id);
				context.add_layout(
					layout_map
						.get(&layout_ref)
						.cloned()
						.or(Some(treeldr_rust_gen::module::Parent::Extern)),
					layout_ref,
				)
			}
		}
	}

	pub fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &GenContext<V>,
	) -> Result<TokenStream, GenError> {
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

	/// Optional extern path.
	extern_path: Option<treeldr_rust_gen::module::ExternPath>,

	/// Visibility.
	vis: syn::Visibility,

	/// Identifier.
	ident: proc_macro2::Ident,

	/// Prefix specific attributes.
	prefix_attrs: PrefixAttributes,

	/// Module content.
	content: Vec<syn::Item>,

	module: Option<treeldr_rust_gen::Ref<treeldr_rust_gen::Module>>,
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
			if is_prefix_path(attr.path()) {
				let span = attr.path().span();
				if attr.path().segments[0].arguments.is_empty() {
					match attr.meta {
						syn::Meta::List(list) => {
							let mut tokens = list.tokens.into_iter();
							iri = Some(expect_iri_literal(&mut tokens, span)?);
						}
						syn::Meta::NameValue(n) => {
							return Err((ParseError::UnexpectedToken, n.eq_token.span()))
						}
						syn::Meta::Path(_) => return Err((ParseError::MissingArgument, span)),
					}
				} else {
					return Err((
						ParseError::UnexpectedPrefixPathArguments,
						attr.path().segments[0].arguments.span(),
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
					extern_path: None,
					ident: m.ident,
					prefix_attrs,
					content: m.content.map(|(_, items)| items).unwrap_or_default(),
					module: None,
				})
			}
			syn::Item::Use(mut u) => {
				let span = u.span();

				match u.tree {
					syn::UseTree::Path(p) => {
						let prefix_attrs = PrefixAttributes::from_attributes(&mut u.attrs, span)?;

						match treeldr_rust_gen::module::ExternPathWithRenaming::try_from(p) {
							Ok(p) => {
								let (path, ident) = p.into_path_and_ident();
								Ok(Self {
									attrs: u.attrs,
									vis: u.vis,
									extern_path: Some(path),
									ident,
									prefix_attrs,
									content: Vec::new(),
									module: None,
								})
							}
							Err(_) => Err((ParseError::NotAModule, span)),
						}
					}
					_ => Err((ParseError::NotAModule, span)),
				}
			}
			_ => Err((ParseError::NotAModule, item.span())),
		}
	}

	pub fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &GenContext<V>,
	) -> Result<TokenStream, GenError> {
		use treeldr_rust_gen::GenerateSyntax;
		let attrs = &self.attrs;
		let vis = &self.vis;
		let ident = &self.ident;
		let module_ref = self.module.unwrap();
		let module = context.module(module_ref).unwrap();

		match module.extern_path() {
			Some(path) => {
				if path.ident() == ident {
					Ok(quote! {
						#(#attrs)*
						#vis use #path;
					})
				} else {
					Ok(quote! {
						#(#attrs)*
						#vis use #path as #ident;
					})
				}
			}
			None => {
				let scope = treeldr_rust_gen::Scope::new(Some(module_ref));
				let generated = module.generate_syntax(context, &scope)?;
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
	}
}
