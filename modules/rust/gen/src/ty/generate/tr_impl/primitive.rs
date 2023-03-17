use proc_macro2::TokenStream;
use quote::quote;
use treeldr::{TId, vocab::Primitive};

use crate::{ty::{generate::GenerateFor, params::{ParametersValues, ParametersBounds, Parameters}}, Context, Error, GenerateIn};

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

			tokens.extend(quote! {
				impl #params #tr_path for #ty_path {}
			})
		}

		Ok(())
	}
}