use quote::quote;
use treeldr::TId;

use crate::{syntax, GenerateSyntax};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderOf(pub TId<treeldr::Type>);

impl<M> GenerateSyntax<M> for ProviderOf {
	type Output = syntax::ClassProviderTraitDefinition;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, crate::Error> {
		let mut scope = scope.clone();
		scope.params.context = Some(syn::parse2(quote!(Self)).unwrap());

		let tr = context.type_trait(self.0).unwrap();

		Ok(syntax::ClassProviderTraitDefinition {
			class_ident: tr.ident().clone(),
			ident: tr.context_ident().clone(),
			trait_path: self.0.generate_syntax(context, &scope)?,
		})
	}
}
