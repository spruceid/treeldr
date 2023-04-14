use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::quote;
use treeldr::TId;

use crate::{
	tr::{CollectContextBounds, MethodType},
	ty::{
		generate::{GenerateFor, InContext},
		params::{ParametersBounds, ParametersValues},
		structure::Struct,
	},
	Context, Error, GenerateIn,
};

use super::{collection_iterator, context_bounds_tokens, ClassTraitImpl};

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

impl<'a, M> GenerateIn<M> for ClassTraitAssociatedTypePath<'a> {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self.ty.field_for(self.prop) {
			Some(f) => {
				if self.collection {
					let iter_expr = collection_iterator(context, scope, f.layout(), params_values)?;
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
						let ty_expr = InContext(item_layout)
							.generate_in_with(context, scope, params_values)
							.into_tokens()?;
						tokens.extend(
							quote!(::treeldr_rust_prelude::iter::Fetch <'a, C, #ty_expr, #iter_expr>),
						)
					} else {
						tokens.extend(iter_expr)
					}

					Ok(())
				} else {
					let layout = context.model().get(f.layout()).unwrap();
					let item_layout = **layout.as_layout().description().collection_item().unwrap();
					tokens.extend(quote!(&'a));
					InContext(item_layout).generate_in(context, scope, params_values, tokens)
				}
			}
			None => {
				if self.collection {
					tokens.extend(quote!(::std::iter::Empty<&'a ::std::convert::Infallible>))
				} else {
					tokens.extend(quote!(&'a ::std::convert::Infallible))
				}

				Ok(())
			}
		}
	}
}

impl<M> GenerateFor<Struct, M> for ClassTraitImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Struct,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		if let Some(tr) = context.type_trait(self.0) {
			let ident = ty.ident();
			let params_values = ParametersValues::default();
			let ty_params = ty.params().instantiate(&params_values);
			let tr_path = self
				.0
				.generate_in_with(context, scope, &params_values)
				.into_tokens()?;
			let mut context_bounds = BTreeSet::new();
			ty.collect_context_bounds(context, self.0, |b| {
				context_bounds.insert(b);
			});

			let mut associated_types = Vec::new();
			for a in tr.associated_types() {
				let a_ident = a.ident();

				let ty_expr = ClassTraitAssociatedTypePath::new(
					ty,
					// self.0,
					a.property(),
					a.bound().is_collection(),
				)
				.generate_in_with(context, scope, &params_values)
				.into_tokens()?;

				associated_types.push(quote! {
					type #a_ident <'a> = #ty_expr where Self: 'a, C: 'a;
				})
			}

			let mut methods = Vec::new();
			for m in tr.methods() {
				let m_ident = m.ident();
				let return_ty = m.return_type_expr(tr);
				let body = match ty.field_for(m.property()) {
					Some(f) => {
						let f_ident = f.ident();
						match m.type_() {
							MethodType::Required(i) => {
								if tr.associated_types()[*i].is_collection() {
									let layout = context.model().get(f.layout()).unwrap();
									let item_layout = **layout
										.as_layout()
										.description()
										.collection_item()
										.unwrap();
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

				methods.push(quote! {
					fn #m_ident <'a> (&'a self, context: &'a C) -> #return_ty {
						#body
					}
				})
			}

			let params_bounds = ParametersBounds::new_for_trait(context_bounds_tokens(
				&context_bounds,
				context,
				scope,
				&params_values,
			)?);
			let params = ty
				.params()
				.with_context()
				.instantiate(&params_values)
				.with_bounds(&params_bounds);

			let dyn_table_path = context
				.module_path(scope)
				.to(&tr.dyn_table_path(context).unwrap());
			let dyn_table_instance_path = context
				.module_path(scope)
				.to(&tr.dyn_table_instance_path(context).unwrap());

			tokens.extend(quote! {
				impl #params #tr_path for #ident #ty_params {
					#(#associated_types)*
					#(#methods)*
				}

				unsafe impl #params ::treeldr_rust_prelude::AsTraitObject<#dyn_table_path<C>> for #ident #ty_params {
					fn as_trait_object(&self) -> (*const u8, #dyn_table_instance_path<C>) {
						let table = #dyn_table_instance_path::new::<Self>();
						(self as *const Self as *const u8, table)
					}
				}
			})
		}

		Ok(())
	}
}
