use std::collections::BTreeSet;

use crate::{ty::{BuiltIn, Context, Description, Enum, Primitive, Struct, Type, generate::GenerateFor, params::ParametersValues}, Error, Generate, GenerateList};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, Id};

/// `FromRdf` trait implementation.
pub struct FromRdfImpl;

impl<M> GenerateFor<Struct, M>for FromRdfImpl {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Struct,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let mut fields_init = Vec::with_capacity(ty.fields().len());
		let mut bounds = BTreeSet::new();

		for field in ty.fields() {
			let id = field.ident();

			let init = match field.property() {
				Some(prop_ref) => {
					let prop = context.model().get(prop_ref).unwrap();

					let layout_ref = field.layout();
					let layout = context.model().get(layout_ref).unwrap();

					if prop.id()
						== treeldr::Id::Iri(IriIndex::Iri(treeldr::vocab::Term::TreeLdr(
							treeldr::vocab::TreeLdr::Self_,
						))) {
						match layout.as_layout().description() {
							treeldr::layout::Description::Required(_) => {
								quote! {
									::treeldr_rust_prelude::Id::from_ref(id)
								}
							}
							treeldr::layout::Description::Option(_) => {
								quote! {
									Some(::treeldr_rust_prelude::Id::from_ref(id))
								}
							}
							_ => {
								quote! {
									Some(::treeldr_rust_prelude::Id::from_ref(id)).into_iter().collect()
								}
							}
						}
					} else {
						let prop_id = match prop_ref.id() {
							Id::Iri(i) => {
								let iri = context.vocabulary().iri(&i).unwrap().into_str();
								quote! {
									::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
										namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
											#iri
										))
									)
								}
							}
							Id::Blank(_) => return Err(Error::BlankProperty(prop_ref))
						};

						let objects = quote! { graph.objects(&id, &#prop_id) };

						match layout.as_layout().description() {
							treeldr::layout::Description::Required(_) => {
								let object = quote! { object };
								let from_object =
									from_object(context, field.ty(context), object.clone(), |b| { bounds.insert(b); });

								quote! {
									let mut objects = #objects;

									match objects.next() {
										Some(object) => {
											if objects.next().is_some() {
												panic!("multiples values on functional property")
											}

											#from_object
										}
										None => {
											return Err(::treeldr_rust_prelude::FromRdfError::MissingRequiredPropertyValue)
										}
									}
								}
							}
							treeldr::layout::Description::Option(_) => {
								match field.ty(context).description() {
									Description::BuiltIn(BuiltIn::Option(item_layout)) => {
										let item_ty = context.layout_type(*item_layout).unwrap();
										let object = quote! { object };
										let from_object = from_object(context, item_ty, object.clone(), |b| { bounds.insert(b); });

										quote! {
											let mut objects = #objects;
											let object = objects.next();
											if objects.next().is_some() {
												panic!("multiples values on functional property")
											}

											match object {
												Some(#object) => Some({
													#from_object
												}),
												None => None
											}
										}
									}
									_ => panic!("not an option"),
								}
							}
							_ => from_objects(context, field.ty(context), objects, |b| { bounds.insert(b); }),
						}
					}
				}
				None => {
					if field.ty(context).impl_default(context) {
						quote! { ::core::default::Default::default() }
					} else {
						return Err(Error::MissingDefaultImpl);
					}
				}
			};

			fields_init.push(quote! { #id: { #init } })
		}

		let ident = ty.ident();
		let params_values = ParametersValues::new(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);

		let bounds = bounds
			.separated_by(&quote!(,))
			.generate_with(context, scope)
			.into_tokens()?;

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V> for #ident #params
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: ::treeldr_rust_prelude::rdf_types::FromIri<Iri=N::Iri>,
				#bounds
			{
				fn from_rdf<G>(
					namespace: &mut N,
					id: &N::Id,
					graph: &G
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::treeldr_rust_prelude::grdf::Graph<
						Subject=N::Id,
						Predicate=N::Id,
						Object=::treeldr_rust_prelude::rdf_types::Object<N::Id, V>
					>
				{
					Ok(Self {
						#(#fields_init),*
					})
				}
			}
		});

		Ok(())
	}
}

impl<M> GenerateFor<Enum, M>for FromRdfImpl {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let ident = ty.ident();
		let params_values = ParametersValues::default();
		let params = ty.params().instantiate(&params_values);

		tokens.extend(quote! {
			impl<I, V, N> ::treeldr_rust_prelude::FromRdf<I, V, N> for #ident #params {
				fn from_rdf<G>(
					namespace: &N,
					id: &I,
					graph: &G
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::grdf::Graph<Subject=I, Predicate=I, Object=::treeldr_rust_prelude::rdf_types::Object<I, V>>
				{
					todo!()
				}
			}
		});

		Ok(())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
	FromLiteral(Primitive)
}

impl<M> Generate<M> for Bound {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self {
			Self::FromLiteral(p) => {
				let ty = p.generate_with(context, scope).into_tokens()?;
				tokens.extend(quote!(#ty: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>));
				Ok(())
			}
		}
	}
}

fn primitive_from_literal<V, M>(
	_context: &Context<V, M>,
	p: Primitive,
	lit: TokenStream
) -> TokenStream {
	match p {
		Primitive::Boolean => quote! {
			<bool as ::treeldr_rust_prelude::rdf::FromLiteral<V, N>>::from_literal(
				namespace,
				#lit
			)?
		},
		Primitive::Integer => quote! {
			<i64 as ::treeldr_rust_prelude::rdf::FromLiteral<V, N>>::from_literal(
				namespace,
				#lit
			)?
		},
		Primitive::UnsignedInteger => {
			todo!("unsigned integer")
		}
		Primitive::Float => {
			todo!("float")
		}
		Primitive::Double => {
			todo!("double")
		}
		Primitive::String => quote! {
			<String as ::treeldr_rust_prelude::rdf::FromLiteral<V, N>>::from_literal(
				namespace,
				#lit
			)?
		},
		Primitive::Time => {
			todo!("time")
		}
		Primitive::Date => {
			todo!("date")
		}
		Primitive::DateTime => quote! {
			<::chrono::DateTime<::chrono::Utc> as ::treeldr_rust_prelude::rdf::FromLiteral<V, N>>::from_literal(
				namespace,
				#lit
			)?
		},
		Primitive::Iri => {
			todo!("iri")
		}
		Primitive::Uri => {
			todo!("uri")
		}
		Primitive::Url => {
			todo!("url")
		}
	}
}

fn from_object<V, M>(
	context: &Context<V, M>,
	ty: &Type,
	object: TokenStream,
	mut bounds: impl FnMut(Bound)
) -> TokenStream {
	match ty.description() {
		Description::BuiltIn(BuiltIn::Required(item)) => {
			let ty = context.layout_type(*item).unwrap();
			from_object(context, ty, object, bounds)
		}
		Description::BuiltIn(BuiltIn::Option(_item)) => {
			todo!("option")
		}
		Description::BuiltIn(BuiltIn::BTreeSet(_item)) => {
			todo!("btreeset")
		}
		Description::BuiltIn(BuiltIn::OneOrMany(_item)) => {
			todo!("oneormany")
		}
		Description::BuiltIn(BuiltIn::Vec(_item)) => {
			todo!("vec")
		}
		Description::Never => {
			quote! {
				return Err(::treeldr_rust_prelude::FromRdfError::Never)
			}
		}
		Description::Alias(_, layout) => {
			let ty = context.layout_type(*layout).unwrap();
			from_object(context, ty, object, bounds)
		}
		Description::Reference => {
			quote! {
				match #object {
					::treeldr_rust_prelude::rdf::Object::Id(id) => id.clone(),
					_ => return Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
				}
			}
		}
		Description::Primitive(p) => {
			bounds(Bound::FromLiteral(*p));
			let lit = quote! { lit };
			let from_literal = primitive_from_literal(context, *p, lit.clone());

			quote! {
				match #object {
					::treeldr_rust_prelude::rdf::Object::Literal(#lit) => { #from_literal },
					_ => return Err(::treeldr_rust_prelude::FromRdfError::ExpectedLiteralValue)
				}
			}
		}
		Description::Struct(_) | Description::Enum(_) => {
			quote! {
				match #object {
					::treeldr_rust_prelude::rdf::Object::Id(id) => {
						::treeldr_rust_prelude::FromRdf::from_rdf(::treeldr_rust_prelude::Id::as_ref(id), graph)?
					},
					_ => return Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
				}
			}
		}
	}
}

fn from_objects<V, M>(
	context: &Context<V, M>,
	ty: &Type,
	objects: TokenStream,
	bounds: impl FnMut(Bound)
) -> TokenStream {
	match ty.description() {
		Description::BuiltIn(BuiltIn::Vec(item)) => {
			let object = quote! { object };
			let from_object =
				from_object(context, context.layout_type(*item).unwrap(), object.clone(), bounds);
			quote! {
				let mut result = ::std::vec::Vec::new();
				for #object in #objects {
					result.push(#from_object)
				}
				result
			}
		}
		Description::BuiltIn(BuiltIn::BTreeSet(item)) => {
			let object = quote! { object };
			let from_object =
				from_object(context, context.layout_type(*item).unwrap(), object.clone(), bounds);
			quote! {
				let mut result = ::std::collections::btree_set::BTreeSet::new();
				for #object in #objects {
					result.insert(#from_object);
				}
				result
			}
		}
		Description::Alias(_, _layout) => {
			quote! {
				todo!("alias from RDF")
			}
		}
		_ => panic!("not a collection type"),
	}
}