//! This library defines the derive macros for the `linked_data` library. It is
//! not meant to be used directly. It is reexported by the `linked_data`
//! library.
use std::fs;

use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput};
use treeldr_layouts::{abs, Layouts};

mod generate;
mod parse;

// struct Input {
// 	// ...
// }

// impl syn::parse::Parse for Input {
// 	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
// 		todo!()
// 	}
// }

#[derive(Debug, thiserror::Error)]
enum Error {
	#[error("parse error: {0}")]
	Json(serde_json::Error),

	#[error("build error: {0}")]
	Layout(abs::syntax::BuildError),
}

struct Attribute(syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]>);

impl Attribute {
	pub fn build(self) -> Result<Layouts, Error> {
		let mut builder = abs::Builder::new();

		for lit in self.0.into_iter() {
			let filename = lit.value();
			let content = fs::read_to_string(filename).unwrap();

			match serde_json::from_str::<abs::syntax::Layout>(&content) {
				Ok(abstract_layout) => {
					if let Err(e) = abstract_layout.build(&mut builder) {
						return Err(Error::Layout(e));
					}
				}
				Err(e) => return Err(Error::Json(e)),
			}
		}

		Ok(builder.build())
	}
}

impl syn::parse::Parse for Attribute {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		input
			.parse_terminated(syn::parse::Parse::parse, syn::Token![,])
			.map(Self)
	}
}

fn generate_layouts(
	layouts: &Layouts,
	gen_options: treeldr_gen_rust::Options,
) -> Result<proc_macro2::TokenStream, treeldr_gen_rust::Error> {
	let mut result = proc_macro2::TokenStream::new();

	for (layout_ref, _) in layouts {
		let layout_result = treeldr_gen_rust::generate(
			treeldr_layouts::distill::RdfContext::default(),
			layouts,
			layout_ref,
			&gen_options,
		);

		match layout_result {
			Ok(tokens) => result.extend(tokens),
			Err(e) => {
				abort_call_site!(e)
			}
		}
	}

	Ok(result)
}

fn use_tree_ident(tree: &syn::UseTree) -> Option<syn::Ident> {
	match tree {
		syn::UseTree::Name(name) => Some(name.ident.clone()),
		syn::UseTree::Rename(rename) => Some(rename.rename.clone()),
		syn::UseTree::Path(path) => use_tree_ident(&path.tree),
		_ => None,
	}
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn tldr(attr: TokenStream, item: TokenStream) -> TokenStream {
	let attr = syn::parse_macro_input!(attr as Attribute);
	let item = syn::parse_macro_input!(item as syn::ItemMod);

	match attr.build() {
		Ok(layouts) => {
			let content = match item.content {
				Some((_, items)) => {
					let mut gen_options = treeldr_gen_rust::Options::new();
					let mut uses = Vec::new();
					for item in items {
						match item {
							syn::Item::Use(mut u) => {
								let mut prefix = None;
								for attr in std::mem::take(&mut u.attrs) {
									if attr.path().is_ident("tldr") {
										match attr.meta {
											syn::Meta::List(list) => {
												let span = list.span();
												match syn::parse2::<syn::LitStr>(list.tokens) {
													Ok(lit) => prefix = Some(lit.value()),
													Err(e) => {
														abort!(span, e)
													}
												}
											}
											meta => {
												abort!(meta, "invalid `tldr` attribute")
											}
										}
									} else {
										abort!(attr, "unsupported attribute")
									}
								}

								if let Some(prefix) = prefix {
									let span = u.tree.span();
									match use_tree_ident(&u.tree) {
										Some(path) => {
											if let Err(e) =
												gen_options.use_module(prefix, path.into())
											{
												abort!(e.span(), e)
											}
										}
										None => {
											abort!(span, "invalid module path")
										}
									}
								}

								uses.push(u);
							}
							item => {
								abort!(item, "unsupported item")
							}
						}
					}

					match generate_layouts(&layouts, gen_options) {
						Ok(tokens) => {
							quote! {
								#(#uses)*
								#tokens
							}
						}
						Err(e) => {
							abort_call_site!(e)
						}
					}
				}
				None => proc_macro2::TokenStream::new(),
			};

			let ident = item.ident;
			let vis = item.vis;

			quote! {
				#vis mod #ident {
					#content
				}
			}
			.into()
		}
		Err(e) => {
			abort_call_site!(e)
		}
	}
}

#[proc_macro]
#[proc_macro_error]
pub fn tldr_include(item: TokenStream) -> TokenStream {
	let attr = syn::parse_macro_input!(item as Attribute);

	match attr.build() {
		Ok(layouts) => {
			let gen_options = treeldr_gen_rust::Options::new();
			match generate_layouts(&layouts, gen_options) {
				Ok(tokens) => tokens.into(),
				Err(e) => {
					abort_call_site!(e)
				}
			}
		}
		Err(e) => {
			abort_call_site!(e)
		}
	}
}

#[proc_macro_derive(SerializeLd, attributes(tldr))]
#[proc_macro_error]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as DeriveInput);
	let mut output = proc_macro2::TokenStream::new();

	match generate::ser::generate(input) {
		Ok(tokens) => output.extend(tokens),
		Err(e) => {
			abort!(e.span(), e)
		}
	}

	output.into()
}

#[proc_macro_derive(DeserializeLd, attributes(tldr))]
#[proc_macro_error]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as DeriveInput);
	let mut output = proc_macro2::TokenStream::new();

	match generate::de::generate(input) {
		Ok(tokens) => output.extend(tokens),
		Err(e) => {
			abort!(e.span(), e)
		}
	}

	output.into()
}
