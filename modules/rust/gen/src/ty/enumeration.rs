use crate::{Context, Error, GenerateIn, Module};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex, TId};

use super::{params::ParametersValues, Parameters};

/// Rust `enum` type.
#[derive(Debug)]
pub struct Enum {
	ident: Ident,
	variants: Vec<Variant>,
	params: Parameters,
}

impl Enum {
	pub fn new(ident: Ident, variants: Vec<Variant>) -> Self {
		Self {
			ident,
			variants,
			params: Parameters::default(),
		}
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn params(&self) -> Parameters {
		self.params
	}

	pub(crate) fn set_params(&mut self, p: Parameters) {
		self.params = p
	}

	pub fn variants(&self) -> &[Variant] {
		&self.variants
	}

	pub(crate) fn compute_params(
		&self,
		mut dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		let mut result = Parameters::default();

		for v in &self.variants {
			if let Some(layout_ref) = v.ty() {
				result.append(dependency_params(layout_ref))
			}
		}

		result
	}
}

#[derive(Debug)]
pub struct Variant {
	ident: Ident,
	ty: Option<TId<treeldr::Layout>>,
}

impl Variant {
	pub fn new(ident: Ident, ty: Option<TId<treeldr::Layout>>) -> Self {
		Self { ident, ty }
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn ty(&self) -> Option<TId<treeldr::Layout>> {
		self.ty
	}
}

impl<M> GenerateIn<M> for Variant {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ident = &self.ident;

		match self.ty.as_ref() {
			Some(ty) => {
				let ty = ty
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;

				tokens.extend(quote! {
					#ident(#ty)
				})
			}
			None => tokens.extend(quote! { #ident }),
		}

		Ok(())
	}
}
