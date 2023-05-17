use crate::{
	syntax,
	ty::{Enum, Struct},
	Context, Error, GenerateSyntax,
};
use quote::quote;
use treeldr::{Id, TId};

/// `IntoJsonLd` trait implementation.
pub struct IntoJsonLdImpl<'a, T> {
	ty_ref: TId<treeldr::Layout>,
	ty: &'a T,
}

impl<'a, T> IntoJsonLdImpl<'a, T> {
	pub fn new(ty_ref: TId<treeldr::Layout>, ty: &'a T) -> Self {
		Self { ty_ref, ty }
	}
}

impl<'a, M> GenerateSyntax<M> for IntoJsonLdImpl<'a, Struct> {
	type Output = syntax::tr_impl::json_ld::IntoJsonLdImpl;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(N::Id)).unwrap());

		let mut insert_field = Vec::new();
		for field in self.ty.fields() {
			if let Some(prop_id) = field.property() {
				let rust_prop_id = match prop_id.into_id() {
					Id::Iri(iri) => {
						let iri = context.vocabulary().iri(&iri).unwrap().to_string();
						quote!(::treeldr_rust_prelude::json_ld::ValidId::Iri(
							vocabulary.insert(::treeldr_rust_prelude::iref::Iri::new(#iri).unwrap())
						))
					}
					Id::Blank(blank) => {
						let blank = context.vocabulary().blank_id(&blank).unwrap().to_string();
						quote!(::treeldr_rust_prelude::json_ld::ValidId::Blank(
							vocabulary.insert_blank_id(::treeldr_rust_prelude::rdf_types::BlankId::new(#blank).unwrap())
						))
					}
				};

				// let key = field.name().as_str();
				let id = field.ident();

				let layout_ref = field.layout();
				let layout = context.model().get(layout_ref).unwrap();

				insert_field.push(match layout.as_layout().description() {
					treeldr::layout::Description::Required(_) => {
						quote! {
							result.properties_mut().insert(
								::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Id::Valid(#rust_prop_id), ()),
								::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(self.#id, vocabulary, meta)
							);
						}
					}
					treeldr::layout::Description::Option(_) => {
						quote! {
							if let Some(value) = self.#id {
								result.properties_mut().insert(
									::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Id::Valid(#rust_prop_id), ()),
									::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(value, vocabulary, meta)
								);
							}
						}
					}
					_ => {
						quote! {
							if !self.#id.is_empty() {
								let list = ::treeldr_rust_prelude::json_ld::object::List::new(
									(),
									::treeldr_rust_prelude::locspan::Meta(
										self.#id.into_iter()
											.map(|v| ::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(v, vocabulary, meta)).collect(),
										()
									)
								);

								result.properties_mut().insert(
									::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Id::Valid(#rust_prop_id), ()),
									::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Indexed::new(::treeldr_rust_prelude::json_ld::Object::List(list), None), ())
								);
							}
						}
					}
				})
			}
		}

		Ok(syntax::tr_impl::json_ld::IntoJsonLdImpl {
			type_path: self.ty_ref.generate_syntax(context, &scope)?,
			body: quote! {
				let mut result = ::treeldr_rust_prelude::json_ld::Node::new();
				#(#insert_field)*
				::treeldr_rust_prelude::locspan::Meta(
					::treeldr_rust_prelude::json_ld::Indexed::new(::treeldr_rust_prelude::json_ld::Object::Node(Box::new(result)), None),
					meta
				)
			},
		})
	}
}

impl<'a, M> GenerateSyntax<M> for IntoJsonLdImpl<'a, Enum> {
	type Output = syntax::tr_impl::json_ld::IntoJsonLdImpl;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(N::Id)).unwrap());

		let variants = self.ty.variants().iter().map(|variant| {
			let v_ident = variant.ident();
			if variant.ty().is_some() {
				quote! {
					Self::#v_ident(value) => {
						value.into_json_ld_object_meta(vocabulary, meta)
					}
				}
			} else {
				quote! {
					Self::#v_ident => {
						::treeldr_rust_prelude::locspan::Meta(
							::treeldr_rust_prelude::json_ld::Indexed::new(
								::treeldr_rust_prelude::json_ld::Object::Node(Box::new(
									::treeldr_rust_prelude::json_ld::Node::new()
								)),
								None
							),
							meta
						)
					}
				}
			}
		});

		Ok(syntax::tr_impl::json_ld::IntoJsonLdImpl {
			type_path: self.ty_ref.generate_syntax(context, &scope)?,
			body: quote! {
				match self {
					#(#variants,)*
				}
			},
		})
	}
}
