use crate::{
	syntax,
	ty::{Enum, Struct},
	Context, Error, GenerateSyntax,
};
use quote::quote;
use treeldr::{Id, TId};

/// `AsJsonLd` trait implementation.
pub struct AsJsonLdImpl<'a, T> {
	ty_ref: TId<treeldr::Layout>,
	ty: &'a T,
}

impl<'a, T> AsJsonLdImpl<'a, T> {
	pub fn new(ty_ref: TId<treeldr::Layout>, ty: &'a T) -> Self {
		Self { ty_ref, ty }
	}
}

impl<'a, M> GenerateSyntax<M> for AsJsonLdImpl<'a, Struct> {
	type Output = syntax::tr_impl::json_ld::AsJsonLdImpl;

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
								::treeldr_rust_prelude::AsJsonLdObjectMeta::as_json_ld_object_meta(&self.#id, vocabulary, interpretation, meta)
							);
						}
					}
					treeldr::layout::Description::Option(_) => {
						quote! {
							if let Some(value) = &self.#id {
								result.properties_mut().insert(
									::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Id::Valid(#rust_prop_id), ()),
									::treeldr_rust_prelude::AsJsonLdObjectMeta::as_json_ld_object_meta(value, vocabulary, interpretation, meta)
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
										self.#id.iter()
											.map(|v| ::treeldr_rust_prelude::AsJsonLdObjectMeta::as_json_ld_object_meta(v, vocabulary, interpretation, meta)).collect(),
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

		Ok(syntax::tr_impl::json_ld::AsJsonLdImpl {
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

impl<'a, M> GenerateSyntax<M> for AsJsonLdImpl<'a, Enum> {
	type Output = syntax::tr_impl::json_ld::AsJsonLdImpl;

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
						value.as_json_ld_object_meta(vocabulary, interpretation, meta)
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

		Ok(syntax::tr_impl::json_ld::AsJsonLdImpl {
			type_path: self.ty_ref.generate_syntax(context, &scope)?,
			body: quote! {
				match self {
					#(#variants,)*
				}
			},
		})
	}
}
