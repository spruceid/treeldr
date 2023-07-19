use quote::quote;
use treeldr::TId;

use crate::{
	syntax,
	tr::{CollectContextBounds, MethodType},
	ty::structure::Struct,
	Context, Error, GenerateSyntax,
};

use super::{collection_iterator, ClassTraitImpl};

pub struct ClassTraitAssociatedTypePath<'a> {
	ty: &'a Struct,
	// tr: TId<treeldr::Type>,
	prop: TId<treeldr::Property>,
	collection: bool,
}

impl<'a> ClassTraitAssociatedTypePath<'a> {
	pub fn new(
		ty: &'a Struct,
		// tr: TId<treeldr::Type>,
		prop: TId<treeldr::Property>,
		collection: bool,
	) -> Self {
		Self {
			ty,
			// tr,
			prop,
			collection,
		}
	}
}

impl<'a, M> GenerateSyntax<M> for ClassTraitAssociatedTypePath<'a> {
	type Output = syn::Type;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		match self.ty.field_for(self.prop) {
			Some(f) => {
				let layout = context.model().get(f.layout()).unwrap();
				let item_layout = layout.as_layout().description().collection_item().unwrap();
				let is_reference = context
					.model()
					.get(**item_layout)
					.unwrap()
					.as_layout()
					.description()
					.is_reference();

				if self.collection {
					let ty = collection_iterator(context, scope, f.layout())?;
					if is_reference {
						Ok(syn::parse2(quote!(::treeldr_rust_prelude::iter::Ids<#ty>)).unwrap())
					} else {
						Ok(
							syn::parse2(quote!(::treeldr_rust_prelude::iter::Values<I, #ty>))
								.unwrap(),
						)
					}
				} else if is_reference {
					Ok(syn::parse2(quote!(::std::convert::Infallible)).unwrap())
				} else {
					let item_layout = layout.as_layout().description().collection_item().unwrap();
					Ok(item_layout.generate_syntax(context, scope)?)
				}
			}
			None => {
				if self.collection {
					Ok(syn::parse2(quote!(
						::std::iter::Empty<
							::treeldr_rust_prelude::Ref<'r, I, ::std::convert::Infallible>,
						>
					))
					.unwrap())
				} else {
					Ok(syn::parse2(quote!(::std::convert::Infallible)).unwrap())
				}
			}
		}
	}
}

impl<'a, M> GenerateSyntax<M> for ClassTraitImpl<'a, Struct> {
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

		let trait_path = self.tr_ref.generate_syntax(context, &scope)?;
		let type_path = self.ty_ref.generate_syntax(context, &scope)?;
		let context_bounds = self
			.ty
			.generate_context_bounds(context, self.tr_ref, &scope)?;

		let mut associated_types = Vec::with_capacity(tr.associated_types().len() * 2);
		for a in tr.associated_types() {
			associated_types.push(syntax::tr_impl::class::AssociatedType {
				ident: a.ident().clone(),
				lifetime: None,
				value: ClassTraitAssociatedTypePath::new(self.ty, a.property(), false)
					.generate_syntax(context, &scope)?,
			});

			if let Some(collection_ident) = a.collection_ident() {
				associated_types.push(syntax::tr_impl::class::AssociatedType {
					ident: collection_ident.clone(),
					lifetime: Some(syn::parse2(quote!('r)).unwrap()),
					value: ClassTraitAssociatedTypePath::new(self.ty, a.property(), true)
						.generate_syntax(context, &scope)?,
				});
			}
		}

		let mut methods = Vec::new();
		for m in tr.methods() {
			let body = match self.ty.field_for(m.property()) {
				Some(f) => {
					let f_ident = f.ident();
					match m.type_() {
						MethodType::Required(i) => {
							if tr.associated_types()[*i].is_collection() {
								let layout = context.model().get(f.layout()).unwrap();
								let item_layout =
									**layout.as_layout().description().collection_item().unwrap();
								if context
									.model()
									.get(item_layout)
									.unwrap()
									.as_layout()
									.description()
									.is_reference()
								{
									quote!(::treeldr_rust_prelude::iter::Ids::new(self.#f_ident.iter()))
								} else {
									quote!(::treeldr_rust_prelude::iter::Values::new(self.#f_ident.iter()))
								}
							} else {
								quote!(&self.#f_ident)
							}
						}
						MethodType::Option(_) => {
							quote!(self.#f_ident.as_ref())
						}
					}
				}
				None => match m.type_() {
					MethodType::Required(i) => {
						if tr.associated_types()[*i].is_collection() {
							quote!(::std::iter::empty())
						} else {
							panic!("missing required field")
						}
					}
					MethodType::Option(_) => {
						quote!(None)
					}
				},
			};

			methods.push(syntax::tr_impl::class::Method {
				ident: m.ident().clone(),
				return_ty: m.return_type_expr(tr),
				body,
			})
		}

		Ok(syntax::tr_impl::class::TraitImpl {
			type_path,
			trait_path,
			context_bounds,
			associated_types,
			methods,
		})
	}
}
