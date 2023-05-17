use quote::quote;
use treeldr::TId;

use crate::{
	syntax,
	tr::{CollectContextBounds, MethodType},
	ty::{generate::InContext, structure::Struct},
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
				if self.collection {
					let iter_expr = collection_iterator(context, scope, f.layout())?;
					let layout = context.model().get(f.layout()).unwrap();
					let item_layout = **layout.as_layout().description().collection_item().unwrap();
					if context
						.model()
						.get(item_layout)
						.unwrap()
						.as_layout()
						.description()
						.is_reference()
					{
						let ty_expr = InContext(item_layout).generate_syntax(context, scope)?;
						Ok(syn::parse2(
							quote!(::treeldr_rust_prelude::iter::Fetch <'a, C, #ty_expr, #iter_expr>),
						)
						.unwrap())
					} else {
						Ok(iter_expr)
					}
				} else {
					let layout = context.model().get(f.layout()).unwrap();
					let item_layout = **layout.as_layout().description().collection_item().unwrap();
					let path = InContext(item_layout).generate_syntax(context, scope)?;
					Ok(syn::parse2(quote!(&'a #path)).unwrap())
				}
			}
			None => {
				if self.collection {
					Ok(
						syn::parse2(quote!(::std::iter::Empty<&'a ::std::convert::Infallible>))
							.unwrap(),
					)
				} else {
					Ok(syn::parse2(quote!(&'a ::std::convert::Infallible)).unwrap())
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

		let mut associated_types = Vec::new();
		for a in tr.associated_types() {
			let ty_expr = ClassTraitAssociatedTypePath::new(
				self.ty,
				// self.0,
				a.property(),
				a.bound().is_collection(),
			)
			.generate_syntax(context, &scope)?;

			associated_types.push((a.ident().clone(), ty_expr));
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
									quote!(::treeldr_rust_prelude::iter::Fetch::new(context, self.#f_ident.iter()))
								} else {
									quote!(self.#f_ident.iter())
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

		let dyn_table_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_path(context).unwrap())
			.generate_syntax(context, &scope)?;
		let dyn_table_instance_path = {
			let mut scope = scope.clone();
			scope.params.lifetime = Some(syn::Lifetime::new("'r", proc_macro2::Span::call_site()));
			context
				.module_path(scope.module)
				.to(&tr.dyn_table_instance_path(context).unwrap())
				.generate_syntax(context, &scope)?
		};

		Ok(syntax::tr_impl::class::TraitImpl {
			type_path,
			trait_path,
			context_bounds,
			associated_types,
			methods,
			dyn_table_path,
			dyn_table_instance_path,
		})
	}
}
