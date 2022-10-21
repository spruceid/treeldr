//! RDF × Rust code generation.
use crate::ty::{BuiltIn, Context, Description, Enum, Primitive, Struct, Type};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex};

pub enum Error {
	NoDefaultImplForOrphanField,
}

fn primitive_from_literal<V, M>(
	context: &Context<V, M>,
	p: Primitive,
	lit: TokenStream,
) -> TokenStream {
	let id_ty = context.ident_type();

	match p {
		Primitive::Boolean => quote! {
			<bool as ::treeldr_rust_prelude::rdf::FromXsdLiteral<#id_ty>>::from_xsd_literal(#lit)?
		},
		Primitive::Integer => quote! {
			<i64 as ::treeldr_rust_prelude::rdf::FromXsdLiteral<#id_ty>>::from_xsd_literal(#lit)?
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
			<String as ::treeldr_rust_prelude::rdf::FromXsdLiteral<#id_ty>>::from_xsd_literal(#lit)?
		},
		Primitive::Time => {
			todo!("time")
		}
		Primitive::Date => {
			todo!("date")
		}
		Primitive::DateTime => quote! {
			<::chrono::DateTime<::chrono::Utc> as ::treeldr_rust_prelude::rdf::FromXsdLiteral<#id_ty>>::from_xsd_literal(#lit)?
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

fn from_object<V, M>(context: &Context<V, M>, ty: &Type<M>, object: TokenStream) -> TokenStream {
	match ty.description() {
		Description::BuiltIn(BuiltIn::Required(item)) => {
			let ty = context.layout_type(*item).unwrap();
			from_object(context, ty, object)
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
			from_object(context, ty, object)
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

fn from_objects<V, M>(context: &Context<V, M>, ty: &Type<M>, objects: TokenStream) -> TokenStream {
	match ty.description() {
		Description::BuiltIn(BuiltIn::Vec(item)) => {
			let object = quote! { object };
			let from_object =
				from_object(context, context.layout_type(*item).unwrap(), object.clone());
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
				from_object(context, context.layout_type(*item).unwrap(), object.clone());
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

/// Generate a function that extracts an instance of the given structure
/// from an RDF graph.
pub fn structure_reader<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
	context: &Context<V, M>,
	ty: &Struct<M>,
	ident: &proc_macro2::Ident,
) -> Result<TokenStream, Error> {
	let mut fields_init = Vec::with_capacity(ty.fields().len());

	for field in ty.fields() {
		let id = field.ident();

		let init = match field.property() {
			Some(prop_ref) => {
				let prop = context.model().properties().get(prop_ref).unwrap();

				let layout_ref = field.layout();
				let layout = context.model().layouts().get(layout_ref).unwrap();

				if prop.id()
					== treeldr::Id::Iri(IriIndex::Iri(treeldr::vocab::Term::TreeLdr(
						treeldr::vocab::TreeLdr::Self_,
					))) {
					match layout.description().value() {
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
					let prop_id = context.ident_type().for_property(context, prop_ref);
					let prop_id = quote! { ::treeldr_rust_prelude::Id::from_ref(#prop_id) }; // FIXME: this is required because `https://github.com/rust-lang/hashbrown/issues/345`
					let id = quote! { ::treeldr_rust_prelude::Id::from_ref(id) }; // FIXME: same limitation
					let objects = quote! { graph.objects(&#id, &#prop_id) };

					match layout.description().value() {
						treeldr::layout::Description::Required(_) => {
							let object = quote! { object };
							let from_object =
								from_object(context, field.ty(context), object.clone());

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
									let from_object = from_object(context, item_ty, object.clone());

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
						_ => from_objects(context, field.ty(context), objects),
					}
				}
			}
			None => {
				if field.ty(context).impl_default(context) {
					quote! { ::core::default::Default::default() }
				} else {
					return Err(Error::NoDefaultImplForOrphanField);
				}
			}
		};

		fields_init.push(quote! { #id: { #init } })
	}

	let id_ty = context.ident_type();
	let id_ty_ref = crate::Referenced(id_ty);

	Ok(quote! {
		impl ::treeldr_rust_prelude::FromRdf<#id_ty> for #ident {
			fn from_rdf<G>(id: #id_ty_ref, graph: &G) -> Result<Self, ::treeldr_rust_prelude::FromRdfError> where G: ::grdf::Graph<Subject=#id_ty, Predicate=#id_ty, Object=::treeldr_rust_prelude::rdf::Object<#id_ty>> {
				Ok(Self {
					#(#fields_init),*
				})
			}
		}
	})
}

/// Generate a function that extracts an instance of the given enumeration
/// from an RDF graph.
pub fn enum_reader<V, M>(
	context: &Context<V, M>,
	_ty: &Enum<M>,
	ident: &proc_macro2::Ident,
) -> Result<TokenStream, Error> {
	let id_ty = context.ident_type();
	let id_ty_ref = crate::Referenced(id_ty);

	Ok(quote! {
		impl ::treeldr_rust_prelude::FromRdf<#id_ty> for #ident {
			fn from_rdf<G>(id: #id_ty_ref, graph: &G) -> Result<Self, ::treeldr_rust_prelude::FromRdfError> where G: ::grdf::Graph<Subject=#id_ty, Predicate=#id_ty, Object=::treeldr_rust_prelude::rdf::Object<#id_ty>> {
				todo!()
			}
		}
	})
}
