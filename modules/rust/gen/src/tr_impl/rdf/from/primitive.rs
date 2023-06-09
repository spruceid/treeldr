use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex};

use crate::{syntax, ty::primitive::Derived, Context, Error, GenerateSyntax};

use super::FromRdfImpl;

impl<'a, M> GenerateSyntax<M> for FromRdfImpl<'a, Derived> {
	type Output = syntax::tr_impl::rdf::FromRdfImpl;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(N::Id)).unwrap());

		let type_iri = self
			.ty_ref
			.id()
			.as_iri()
			.map(|iri| context.vocabulary().iri(iri).unwrap().to_owned());

		Ok(syntax::tr_impl::rdf::FromRdfImpl::Literal(
			syntax::tr_impl::rdf::FromRdfLiteralImpl {
				type_path: self.ty_ref.generate_syntax(context, &scope)?,
				type_iri,
				base_type_path: self.ty.base().generate_syntax(context, &scope)?,
			},
		))
	}
}
