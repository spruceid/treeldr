use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use shelves::Ref;
use treeldr::{vocab::Primitive, TId};

use crate::{
	module::{TraitId, TraitImpl},
	tr::{AssociatedTypeBound, CollectContextBounds, ContextBound, MethodType},
	ty::{
		self,
		enumeration::Enum,
		params::{Parameter, Parameters, ParametersBounds, ParametersValues},
		structure::Struct,
	},
	Context, Error, Generate, GenerateIn, Module,
};

use super::{GenerateFor, InContext};

impl<M> GenerateIn<M> for ContextBound {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let tr = context.type_trait(self.0).unwrap();
		context
			.module_path(scope)
			.to(&tr
				.context_path(context)
				.ok_or(Error::UnreachableTrait(self.0))?)
			.to_tokens(tokens);
		let id_param_value = params_values.get(Parameter::Identifier);
		tokens.extend(quote! { <#id_param_value> });
		Ok(())
	}
}

fn context_bounds_tokens<
	V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	M,
>(
	bounds: &BTreeSet<ContextBound>,
	context: &Context<V, M>,
	scope: Option<Ref<Module>>,
	params: &ParametersValues,
) -> Result<TokenStream, Error> {
	let mut tokens = quote!(?Sized);

	for b in bounds {
		tokens.extend(quote!(+));
		b.generate_in(context, scope, params, &mut tokens)?
	}

	Ok(tokens)
}

impl<M> Generate<M> for TraitImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ty = context.layout_type(self.ty).unwrap();

		match ty.description() {
			ty::Description::Struct(s) => match self.tr {
				TraitId::FromRdf => {
					super::rdf::from::FromRdfImpl.generate(context, scope, s, tokens)
				}
				TraitId::TriplesAndValues => {
					super::rdf::to::RdfTriplesImpl.generate(context, scope, s, tokens)
				}
				TraitId::IntoJsonLd => {
					super::json_ld::IntoJsonLdImpl.generate(context, scope, s, tokens)
				}
				TraitId::Defined(tr) => ClassTraitImpl(tr).generate(context, scope, s, tokens),
			},
			ty::Description::Enum(e) => match self.tr {
				TraitId::FromRdf => {
					super::rdf::from::FromRdfImpl.generate(context, scope, e, tokens)
				}
				TraitId::TriplesAndValues => {
					super::rdf::to::RdfTriplesImpl.generate(context, scope, e, tokens)
				}
				TraitId::IntoJsonLd => {
					super::json_ld::IntoJsonLdImpl.generate(context, scope, e, tokens)
				}
				TraitId::Defined(tr) => ClassTraitImpl(tr).generate(context, scope, e, tokens),
			},
			ty::Description::Primitive(p) => match self.tr {
				TraitId::Defined(tr) => ClassTraitImpl(tr).generate(context, scope, p, tokens),
				_ => Ok(()),
			},
			_ => {
				panic!("unable to implement trait for non enum/struct type")
			}
		}
	}
}

/// Class trait implementation.
pub struct ClassTraitImpl(TId<treeldr::Type>);

fn collection_iterator<V, M>(
	context: &Context<V, M>,
	scope: Option<shelves::Ref<crate::Module>>,
	collection_layout: TId<treeldr::Layout>,
	params_values: &ParametersValues,
) -> Result<TokenStream, Error>
where
	V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
{
	let ty = context.layout_type(collection_layout).unwrap();
	match ty.description() {
		ty::Description::BuiltIn(b) => match b {
			ty::BuiltIn::Vec(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::std::slice::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::Option(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::std::option::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::BTreeSet(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::std::collections::btree_set::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::OneOrMany(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::treeldr_rust_prelude::one_or_many::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::Required(_) => panic!("cannot turn required layout into iterator"),
		},
		_ => panic!("cannot turn a non-built-in layout into an iterator"),
	}
}

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

				let ty_expr = match ty.field_for(a.property()) {
					Some(f) => match a.bound() {
						AssociatedTypeBound::Types(_) => {
							let layout = context.model().get(f.layout()).unwrap();
							let item_layout =
								**layout.as_layout().description().collection_item().unwrap();
							let ty_expr = InContext(item_layout)
								.generate_in_with(context, scope, &params_values)
								.into_tokens()?;
							quote!(&'a #ty_expr)
						}
						AssociatedTypeBound::Collection(_) => {
							let iter_expr =
								collection_iterator(context, scope, f.layout(), &params_values)?;
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
								let ty_expr = InContext(item_layout)
									.generate_in_with(context, scope, &params_values)
									.into_tokens()?;
								quote!(::treeldr_rust_prelude::iter::Fetch <'a, C, #ty_expr, #iter_expr>)
							} else {
								iter_expr
							}
						}
					},
					None => match a.bound() {
						AssociatedTypeBound::Types(_) => {
							quote!(::std::convert::Infallible)
						}
						AssociatedTypeBound::Collection(_) => {
							quote!(::std::iter::Empty<::std::convert::Infallible>)
						}
					},
				};

				associated_types.push(quote! {
					type #a_ident <'a> = #ty_expr where Self: 'a, C: 'a;
				})
			}

			let mut methods = Vec::new();
			for m in tr.methods() {
				let m_ident = m.ident();
				let return_ty = match m.type_() {
					MethodType::Required(i) => {
						let a_ident = tr.associated_types()[*i].ident();
						quote!(Self::#a_ident<'a>)
					}
					MethodType::Option(i) => {
						let a_ident = tr.associated_types()[*i].ident();
						quote!(Option<Self::#a_ident<'a>>)
					}
				};

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

			tokens.extend(quote! {
				impl #params #tr_path for #ident #ty_params {
					#(#associated_types)*
					#(#methods)*
				}
			})
		}

		Ok(())
	}
}

impl<M> GenerateFor<Enum, M> for ClassTraitImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		if let Some(tr) = context.type_trait(self.0) {
			let ident = ty.ident();
			let params_values = ParametersValues::default();
			let params_bounds = ParametersBounds::new_for_trait(quote!(?Sized));
			let params = ty
				.params()
				.with_context()
				.instantiate(&params_values)
				.with_bounds(&params_bounds);
			let ty_params = ty.params().instantiate(&params_values);
			let tr_path = self
				.0
				.generate_in_with(context, scope, &params_values)
				.into_tokens()?;

			let mut associated_types = Vec::new();
			for a in tr.associated_types() {
				let a_ident = a.ident();
				associated_types.push(quote! {
					type #a_ident;
				})
			}

			let mut methods = Vec::new();
			for m in tr.methods() {
				let m_ident = m.ident();
				methods.push(quote! {
					fn #m_ident <'a> (&'a self, context: &'a C) {
						todo!()
					}
				})
			}

			tokens.extend(quote! {
				impl #params #tr_path for #ident #ty_params {
					#(#associated_types)*
					#(#methods)*
				}
			})
		}

		Ok(())
	}
}
