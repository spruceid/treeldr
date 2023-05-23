use quote::quote;

use crate::{
	syntax,
	tr::{CollectContextBounds, MethodType},
	ty::enumeration::Enum,
	Context, Error, GenerateSyntax,
};

use super::ClassTraitImpl;

impl<'a, M> GenerateSyntax<M> for ClassTraitImpl<'a, Enum> {
	type Output = syntax::tr_impl::class::TraitImpl;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let tr = context.type_trait(self.tr_ref).unwrap();

		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(I)).unwrap());
		scope.params.context = Some(syn::parse2(quote!(C)).unwrap());
		scope.params.lifetime = Some(syn::Lifetime::new("'r", proc_macro2::Span::call_site()));

		let context_bounds = self
			.ty
			.generate_context_bounds(context, self.tr_ref, &scope)?;

		let type_path = self.ty_ref.generate_syntax(context, &scope)?;
		let trait_path = self.tr_ref.generate_syntax(context, &scope)?;

		let mut associated_types = Vec::new();
		for a in tr.associated_types() {
			let a_expr = match a.trait_object_path(context, tr) {
				Some(path) => {
					let path = context
						.module_path(scope.module)
						.to(&path)
						.generate_syntax(context, &scope)?;
					syn::parse2(quote!(#path)).unwrap()
				}
				None => {
					let item_path = tr.associated_types()[a.collection_item_type().unwrap()]
						.trait_object_path(context, tr)
						.unwrap();
					let item_path = context
						.module_path(scope.module)
						.to(&item_path)
						.generate_syntax(context, &scope)?;
					syn::parse2(quote!(Box<dyn 'r + Iterator<Item = #item_path>>)).unwrap()
				}
			};

			associated_types.push((a.ident().clone(), a_expr))
		}

		let mut methods = Vec::new();
		for m in tr.methods() {
			let m_ident = m.ident();
			let return_ty = m.return_type_expr(tr);

			let mut cases = Vec::with_capacity(self.ty.variants().len());
			for v in self.ty.variants() {
				let v_ident = v.ident();
				let case = match m.type_() {
					MethodType::Option(i) => {
						let m_a = &tr.associated_types()[*i];
						let m_path = context
							.module_path(scope.module)
							.to(&m_a.trait_object_path(context, tr).unwrap())
							.generate_syntax(context, &scope)?;

						if v.ty().is_some() {
							quote! {
								Self::#v_ident (value) => {
									value.#m_ident(context).map(#m_path::new)
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
									let path = context
										.module_path(scope.module)
										.to(&path)
										.generate_syntax(context, &scope)?;
									quote! {
										Self::#v_ident (value) => {
											#path::new(value.#m_ident(context))
										}
									}
								}
								None => {
									let item_a =
										&tr.associated_types()[m_a.collection_item_type().unwrap()];
									let path = context
										.module_path(scope.module)
										.to(&item_a.trait_object_path(context, tr).unwrap())
										.generate_syntax(context, &scope)?;
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
				};

				cases.push(case)
			}

			methods.push(syntax::tr_impl::class::Method {
				ident: m.ident().clone(),
				return_ty,
				body: quote! {
					match self {
						#(#cases)*
					}
				},
			});
		}

		let dyn_table_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_path(context).unwrap())
			.generate_syntax(context, &scope)?;
		let dyn_table_instance_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_instance_path(context).unwrap())
			.generate_syntax(context, &scope)?;

		let mut type_params = Vec::new();
		if self.ty.params().identifier {
			type_params.push(syn::parse2(quote!(I)).unwrap());
		}

		Ok(syntax::tr_impl::class::TraitImpl {
			type_path,
			type_params,
			trait_path,
			context_bounds,
			associated_types,
			methods,
			dyn_table_path,
			dyn_table_instance_path,
		})
	}
}