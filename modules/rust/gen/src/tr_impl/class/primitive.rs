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
		let associated_types = tr
			.associated_types()
			.iter()
			.map(|a| {
				let ty_expr = if a.is_collection() {
					let item_a = &tr.associated_types()[a.collection_item_type().unwrap()];
					let item_ident = item_a.ident();
					syn::parse2(quote!(::std::iter::Empty<Self::#item_ident <'r>>)).unwrap()
				} else {
					syn::parse2(quote!(&'r ::std::convert::Infallible)).unwrap()
				};

				(a.ident().clone(), ty_expr)
			})
			.collect();

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

		let dyn_table_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_path(context).unwrap())
			.generate_syntax(context, &scope)?;
		let dyn_table_instance_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_instance_path(context).unwrap())
			.generate_syntax(context, &scope)?;

		Ok(syntax::tr_impl::class::TraitImpl {
			type_path,
			type_params: Vec::new(),
			trait_path,
			context_bounds: Vec::new(),
			associated_types,
			methods,
			dyn_table_path,
			dyn_table_instance_path,
		})
	}
}
