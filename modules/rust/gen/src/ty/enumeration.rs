use crate::{Context, Error, Generate, Module};
use proc_macro2::TokenStream;
use quote::quote;
use shelves::Ref;

pub struct Enum<F> {
	ident: proc_macro2::Ident,
	variants: Vec<Variant<F>>,
}

impl<F> Enum<F> {
	pub fn new(ident: proc_macro2::Ident, variants: Vec<Variant<F>>) -> Self {
		Self { ident, variants }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn variants(&self) -> &[Variant<F>] {
		&self.variants
	}
}

pub struct Variant<F> {
	ident: proc_macro2::Ident,
	ty: Option<Ref<treeldr::layout::Definition<F>>>,
}

impl<F> Variant<F> {
	pub fn new(ident: proc_macro2::Ident, ty: Option<Ref<treeldr::layout::Definition<F>>>) -> Self {
		Self { ident, ty }
	}
}

impl<F> Generate<F> for Variant<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
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
