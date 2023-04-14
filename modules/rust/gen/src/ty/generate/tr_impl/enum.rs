use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
	tr::{CollectContextBounds, MethodType},
	ty::{
		enumeration::Enum,
		generate::GenerateFor,
		params::{ParametersBounds, ParametersValues},
	},
	Context, Error, GenerateIn,
};

use super::{context_bounds_tokens, ClassTraitImpl};

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
			let mut context_bounds = BTreeSet::new();
			ty.collect_context_bounds(context, self.0, |b| {
				context_bounds.insert(b);
			});
			let params_bounds = ParametersBounds::new_for_trait(context_bounds_tokens(
				&context_bounds,
				context,
				scope,
				&params_values,
			)?);
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
					}
					None => {
						let item_path = tr.associated_types()[a.collection_item_type().unwrap()]
							.trait_object_path(context, tr)
							.unwrap();
						let item_path = context.module_path(scope).to(&item_path);
						quote!(Box<dyn 'a + Iterator<Item = #item_path <'a, C>>>)
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

				let variants = ty.variants().iter().map(|v| {
					let v_ident = v.ident();
					match m.type_() {
						MethodType::Option(i) => {
							let m_a = &tr.associated_types()[*i];
							let m_path = context
								.module_path(scope)
								.to(&m_a.trait_object_path(context, tr).unwrap());

							if v.ty().is_some() {
								quote! {
									Self::#v_ident (value) => {
										value.#m_ident(context).map(#m_path::<'a, C>::new)
									}
								}
							} else {
								quote! {
									Self::#v_ident => {
										None
									}
								}
							}
						}
						MethodType::Required(i) => {
							if v.ty().is_some() {
								let m_a = &tr.associated_types()[*i];

								match m_a.trait_object_path(context, tr) {
									Some(path) => {
										let path = context.module_path(scope).to(&path);
										quote! {
											Self::#v_ident (value) => {
												#path::new(value.#m_ident(context))
											}
										}
									}
									None => {
										let item_a = &tr.associated_types()
											[m_a.collection_item_type().unwrap()];
										let path = context
											.module_path(scope)
											.to(&item_a.trait_object_path(context, tr).unwrap());
										quote! {
											Self::#v_ident (value) => {
												Box::new(value.#m_ident(context).map(#path::new))
											}
										}
									}
								}
							} else {
								quote! {
									Self::#v_ident => {
										Box::new(::std::iter::empty())
									}
								}
							}
						}
					}
				});

				methods.push(quote! {
					fn #m_ident <'a> (&'a self, context: &'a C) -> #return_ty {
						match self {
							#(#variants)*
						}
					}
				});
			}

			let dyn_table_path = context
				.module_path(scope)
				.to(&tr.dyn_table_path(context).unwrap());
			let dyn_table_instance_path = context
				.module_path(scope)
				.to(&tr.dyn_table_instance_path(context).unwrap());

			tokens.extend(quote! {
				impl #params #tr_path for #ident #ty_params {
					#(#associated_types)*
					#(#methods)*
				}

				unsafe impl #params ::treeldr_rust_prelude::AsTraitObject<#dyn_table_path<C>> for #ident #ty_params {
					fn as_trait_object(&self) -> (*const u8, #dyn_table_instance_path<C>) {
						let table = #dyn_table_instance_path::new::<Self>();
						(self as *const Self as *const u8, table)
					}
				}
			})
		}

		Ok(())
	}
}
