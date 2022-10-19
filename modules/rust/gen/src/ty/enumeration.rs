use crate::{Context, Error, Generate, Module};
use proc_macro2::TokenStream;
use quote::quote;
use shelves::Ref;

pub struct Enum<M> {
	ident: proc_macro2::Ident,
	variants: Vec<Variant<M>>,
}

impl<M> Enum<M> {
	pub fn new(ident: proc_macro2::Ident, variants: Vec<Variant<M>>) -> Self {
		Self { ident, variants }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn variants(&self) -> &[Variant<M>] {
		&self.variants
	}
}

pub struct Variant<M> {
	ident: proc_macro2::Ident,
	ty: Option<Ref<treeldr::layout::Definition<M>>>,
}

impl<M> Variant<M> {
	pub fn new(ident: proc_macro2::Ident, ty: Option<Ref<treeldr::layout::Definition<M>>>) -> Self {
		Self { ident, ty }
	}
}

impl<M> Generate<M> for Variant<M> {
	fn generate(
		&self,
		context: &Context<M>,
		scope: Option<Ref<Module<M>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<M>> {
		let ident = &self.ident;

		match self.ty.as_ref() {
			Some(ty) => {
				let ty = ty.with(context, scope).into_tokens()?;

				tokens.extend(quote! {
					#ident(#ty)
				})
			}
			None => tokens.extend(quote! { #ident }),
		}

		Ok(())
	}
}
