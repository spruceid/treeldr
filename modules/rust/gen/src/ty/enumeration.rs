use crate::{Context, Error, Generate, Module};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex, TId};

use super::Parameters;

/// Rust `enum` type.
pub struct Enum {
	ident: proc_macro2::Ident,
	variants: Vec<Variant>,
	params: Parameters
}

impl Enum {
	pub fn new(ident: proc_macro2::Ident, variants: Vec<Variant>) -> Self {
		Self { ident, variants, params: Parameters::default() }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn params(&self) -> Parameters {
		self.params
	}

	pub fn variants(&self) -> &[Variant] {
		&self.variants
	}
}

pub struct Variant {
	ident: proc_macro2::Ident,
	ty: Option<TId<treeldr::Layout>>,
}

impl Variant {
	pub fn new(ident: proc_macro2::Ident, ty: Option<TId<treeldr::Layout>>) -> Self {
		Self { ident, ty }
	}
}

impl<M> Generate<M> for Variant {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ident = &self.ident;

		match self.ty.as_ref() {
			Some(ty) => {
				let ty = ty.generate_with(context, scope).into_tokens()?;

				tokens.extend(quote! {
					#ident(#ty)
				})
			}
			None => tokens.extend(quote! { #ident }),
		}

		Ok(())
	}
}
