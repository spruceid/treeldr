use proc_macro2::TokenStream;
use quote::quote;

use crate::{ty::{generate::GenerateFor, enumeration::Enum, params::{ParametersValues, ParametersBounds}}, Context, Error, GenerateIn};

use super::ClassTraitImpl;

impl<M> GenerateFor<Enum, M> for ClassTraitImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		if let Some(tr) = context.type_trait(self.0) {
			let ident = ty.ident();
			let params_values = ParametersValues::default();
			let params_bounds = ParametersBounds::new_for_trait(quote!(?Sized));
			let params = ty
				.params()
				.with_context()
				.instantiate(&params_values)
				.with_bounds(&params_bounds);
			let ty_params = ty.params().instantiate(&params_values);
			let tr_path = self
				.0
				.generate_in_with(context, scope, &params_values)
				.into_tokens()?;

			let mut associated_types = Vec::new();
			for a in tr.associated_types() {
				let a_ident = a.ident();

				let a_expr = match a.trait_object_path(context, tr) {
					Some(path) => {
						let path = context.module_path(scope).to(&path);
						quote!(#path <'a, C>)
					},
					None => {
						let item_path = tr.associated_types()[a.collection_item_type().unwrap()].trait_object_path(context, tr).unwrap();
						let item_path = context.module_path(scope).to(&item_path);
						quote!(Box<dyn Iterator<Item = #item_path <'a, C>>>)
					}
				};

				associated_types.push(quote! {
					type #a_ident <'a> = #a_expr where Self: 'a , C: 'a;
				})
			}

			let mut methods = Vec::new();
			for m in tr.methods() {
				let m_ident = m.ident();
				let return_ty = m.return_type_expr(tr);
				methods.push(quote! {
					fn #m_ident <'a> (&'a self, context: &'a C) -> #return_ty {
						todo!()
					}
				})
			}

			tokens.extend(quote! {
				impl #params #tr_path for #ident #ty_params {
					#(#associated_types)*
					#(#methods)*
				}
			})
		}

		Ok(())
	}
}