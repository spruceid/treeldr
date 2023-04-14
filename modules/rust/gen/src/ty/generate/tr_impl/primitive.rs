use proc_macro2::TokenStream;
use quote::quote;
use treeldr::{vocab::Primitive, TId};

use crate::{
	tr::MethodType,
	ty::{
		generate::GenerateFor,
		params::{Parameters, ParametersBounds, ParametersValues},
	},
	Context, Error, GenerateIn,
};

use super::ClassTraitImpl;

impl<M> GenerateFor<Primitive, M> for ClassTraitImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Primitive,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		if context.type_trait(self.0).is_some() {
			let id: TId<treeldr::Layout> = TId::new(ty.id());
			let params_values = ParametersValues::default();
			let params_bounds = ParametersBounds::new_for_trait(quote!(?Sized));
			let params = Parameters::context_parameter()
				.instantiate(&params_values)
				.with_bounds(&params_bounds);
			let ty_path = id
				.generate_in_with(context, scope, &params_values)
				.into_tokens()?;
			let tr_path = self
				.0
				.generate_in_with(context, scope, &params_values)
				.into_tokens()?;

			let tr = context.type_trait(self.0).unwrap();
			let assoc_types = tr.associated_types().iter().map(|a| {
				let ty_expr = if a.is_collection() {
					let item_a = &tr.associated_types()[a.collection_item_type().unwrap()];
					let item_ident = item_a.ident();
					quote!(::std::iter::Empty<Self::#item_ident <'a>>)
				} else {
					quote!(&'a ::std::convert::Infallible)
				};

				let a_ident = a.ident();

				quote! {
					type #a_ident <'a> = #ty_expr where Self: 'a, C: 'a;
				}
			});

			let methods: Vec<_> = tr
				.methods()
				.iter()
				.map(|m| {
					let m_ident = m.ident();
					let return_ty = m.return_type_expr(tr);
					let body = match m.type_() {
						MethodType::Option(_) => {
							quote! {
								None
							}
						}
						MethodType::Required(i) => {
							let m_a = &tr.associated_types()[*i];
							if m_a.is_collection() {
								quote! {
									::std::iter::empty()
								}
							} else {
								quote! {
									unreachable!()
								}
							}
						}
					};

					quote! {
						fn #m_ident <'a> (&'a self, context: &'a C) -> #return_ty {
							#body
						}
					}
				})
				.collect();

			let dyn_table_path = context
				.module_path(scope)
				.to(&tr.dyn_table_path(context).unwrap());
			let dyn_table_instance_path = context
				.module_path(scope)
				.to(&tr.dyn_table_instance_path(context).unwrap());

			tokens.extend(quote! {
				impl #params #tr_path for #ty_path {
					#(#assoc_types)*
					#(#methods)*
				}

				unsafe impl #params ::treeldr_rust_prelude::AsTraitObject<#dyn_table_path<C>> for #ty_path {
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
