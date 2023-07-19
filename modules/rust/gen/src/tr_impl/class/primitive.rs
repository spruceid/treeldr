use quote::quote;
use treeldr::vocab::Primitive;

use crate::{syntax, tr::MethodType, Context, Error, GenerateSyntax};

use super::ClassTraitImpl;

impl<'a, M> GenerateSyntax<M> for ClassTraitImpl<'a, Primitive> {
	type Output = syntax::tr_impl::class::TraitImpl;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(I)).unwrap());
		scope.params.context = Some(syn::parse2(quote!(C)).unwrap());
		scope.params.lifetime = Some(syn::Lifetime::new("'r", proc_macro2::Span::call_site()));

		let type_path = self.ty_ref.generate_syntax(context, &scope)?;

		let trait_path = self.tr_ref.generate_syntax(context, &scope)?;

		let tr = context.type_trait(self.tr_ref).unwrap();
		let mut associated_types = Vec::with_capacity(tr.associated_types().len() * 2);
		for a in tr.associated_types() {
			let ident = a.ident();
			associated_types.push(syntax::tr_impl::class::AssociatedType {
				ident: ident.clone(),
				lifetime: None,
				value: syn::parse2(quote!(::std::convert::Infallible)).unwrap(),
			});

			if let Some(collection_ident) = a.collection_ident() {
				associated_types.push(syntax::tr_impl::class::AssociatedType {
					ident: collection_ident.clone(),
					lifetime: Some(syn::parse2(quote!('r)).unwrap()),
					value: syn::parse2(
						quote!(::std::iter::Empty<::treeldr_rust_prelude::Ref<'r, I, Self::#ident>>),
					)
					.unwrap(),
				});
			}
		}

		let methods: Vec<_> = tr
			.methods()
			.iter()
			.map(|m| {
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

				syntax::tr_impl::class::Method {
					ident: m.ident().clone(),
					return_ty: m.return_type_expr(tr),
					body,
				}
			})
			.collect();

		Ok(syntax::tr_impl::class::TraitImpl {
			type_path,
			trait_path,
			context_bounds: Vec::new(),
			associated_types,
			methods,
		})
	}
}
