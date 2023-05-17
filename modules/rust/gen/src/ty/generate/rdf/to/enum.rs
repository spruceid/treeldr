use std::collections::BTreeSet;

use quote::quote;

use crate::{syntax, ty::enumeration::Enum, GenerateSyntax};

use super::{
	collect_bounds, quads_and_values_iterator_name_from, quads_and_values_iterator_of, RdfQuadsImpl,
};

impl<'a, M> GenerateSyntax<M> for RdfQuadsImpl<'a, Enum> {
	type Output = syntax::tr_impl::rdf::QuadsImpl;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, crate::Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(N::Id)).unwrap());

		let ident = self.ty.ident();
		let iterator_ident = quads_and_values_iterator_name_from(ident);
		let mut iterator_variants = Vec::with_capacity(self.ty.variants().len());
		let mut bounds_set = BTreeSet::new();
		for variant in self.ty.variants() {
			let variant_ident = variant.ident();
			let variant_type = match variant.ty() {
				Some(variant_type) => {
					let variant_iterator_type =
						quads_and_values_iterator_of(context, &scope, variant_type, quote!('a))?;
					collect_bounds(context, variant_type, |b| {
						bounds_set.insert(b);
					});

					Some(variant_iterator_type)
				}
				None => None,
			};

			iterator_variants.push(syntax::tr_impl::rdf::IteratorVariant {
				ident: variant_ident.clone(),
				ty: variant_type,
			})
		}

		let mut bounds = Vec::with_capacity(bounds_set.len());
		for b in bounds_set {
			bounds.push(b.generate_syntax(context, &scope)?)
		}

		let type_path = self.ty_ref.generate_syntax(context, &scope)?;

		Ok(syntax::tr_impl::rdf::QuadsImpl {
			type_path,
			iterator_ty: syntax::tr_impl::rdf::IteratorType::Enum(
				syntax::tr_impl::rdf::IteratorEnum {
					ident: iterator_ident,
					variants: iterator_variants,
				},
			),
			bounds,
		})
	}
}
