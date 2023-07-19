use crate::{
	syntax,
	ty::{Enum, Struct},
	Context, Error, GenerateSyntax,
};
use quote::quote;
use treeldr::TId;

pub struct IntoJsonLdSyntaxImpl<'a, T> {
	ty_ref: TId<treeldr::Layout>,
	ty: &'a T,
}

impl<'a, T> IntoJsonLdSyntaxImpl<'a, T> {
	pub fn new(ty_ref: TId<treeldr::Layout>, ty: &'a T) -> Self {
		Self { ty_ref, ty }
	}
}

impl<'a, M> GenerateSyntax<M> for IntoJsonLdSyntaxImpl<'a, Struct> {
	type Output = syntax::tr_impl::json_ld::IntoJsonLdSyntaxImpl;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(I::Resource)).unwrap());

		let mut insert_field = Vec::new();
		for field in self.ty.fields() {
			let key = field.name().as_str();
			let id = field.ident();

			let layout_ref = field.layout();
			let layout = context.model().get(layout_ref).unwrap();

			insert_field.push(match layout.as_layout().description() {
				treeldr::layout::Description::Required(_) => {
					quote! {
						result.insert(
							::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
							::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(self.#id, vocabulary, interpretation), ())
						);
					}
				}
				treeldr::layout::Description::Option(_) => {
					quote! {
						if let Some(value) = self.#id {
							result.insert(
								::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
								::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(value, vocabulary, interpretation), ())
							);
						}
					}
				}
				_ => {
					quote! {
						if !self.#id.is_empty() {
							result.insert(
								::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
								::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::syntax::Value::Array(
									self.#id.into_iter()
										.map(|v| ::locspan::Meta(::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(v, vocabulary, interpretation), ())).collect()
								), ())
							);
						}
					}
				}
			})
		}

		Ok(syntax::tr_impl::json_ld::IntoJsonLdSyntaxImpl {
			type_path: self.ty_ref.generate_syntax(context, &scope)?,
			body: quote! {
				let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
				#(#insert_field)*
				result.into()
			},
		})
	}
}

impl<'a, M> GenerateSyntax<M> for IntoJsonLdSyntaxImpl<'a, Enum> {
	type Output = syntax::tr_impl::json_ld::IntoJsonLdSyntaxImpl;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(I::Resource)).unwrap());

		let variants = self.ty.variants().iter().map(|variant| {
			let v_ident = variant.ident();
			if variant.ty().is_some() {
				quote! {
					Self::#v_ident(value) => {
						value.into_json_ld_syntax(vocabulary, interpretation)
					}
				}
			} else {
				quote! {
					Self::#v_ident => {
						::treeldr_rust_prelude::json_ld::syntax::Value::Null
					}
				}
			}
		});

		Ok(syntax::tr_impl::json_ld::IntoJsonLdSyntaxImpl {
			type_path: self.ty_ref.generate_syntax(context, &scope)?,
			body: quote! {
				match self {
					#(#variants,)*
				}
			},
		})
	}
}
