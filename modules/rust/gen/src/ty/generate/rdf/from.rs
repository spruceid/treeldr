use std::collections::HashSet;

use crate::{
	ty::{self, BuiltIn, Context, Description, Primitive, Type},
	Error, Generate,
};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

mod r#enum;
mod r#struct;

/// `FromRdf` trait implementation.
pub struct FromRdfImpl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
	FromLiteral(Primitive),
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

/// Collect the bounds necessary for the `FromRdf` implementation of the given
/// layout.
fn collect_bounds<V, M>(
	context: &crate::Context<V, M>,
	layout: TId<treeldr::Layout>,
	mut bound: impl FnMut(Bound),
) {
	let mut stack = vec![layout];
	let mut visited = HashSet::new();

	while let Some(layout) = stack.pop() {
		if visited.insert(layout) {
			let ty = context.layout_type(layout).unwrap();

			match ty.description() {
				ty::Description::Never => (),
				ty::Description::Alias(a) => stack.push(a.target()),
				ty::Description::Primitive(p) => bound(Bound::FromLiteral(*p)),
				ty::Description::BuiltIn(b) => match b {
					ty::BuiltIn::BTreeSet(item_layout)
					| ty::BuiltIn::OneOrMany(item_layout)
					| ty::BuiltIn::Option(item_layout)
					| ty::BuiltIn::Required(item_layout)
					| ty::BuiltIn::Vec(item_layout) => stack.push(*item_layout),
				},
				ty::Description::Struct(s) => {
					for field in s.fields() {
						stack.push(field.layout())
					}
				}
				ty::Description::Enum(e) => {
					for variant in e.variants() {
						if let Some(layout_ref) = variant.ty() {
							stack.push(layout_ref)
						}
					}
				}
				ty::Description::Reference(_) => (),
			}
		}
	}
}

fn primitive_from_literal<V, M>(
	_context: &Context<V, M>,
	p: Primitive,
	lit: TokenStream,
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

fn from_object<V, M>(context: &Context<V, M>, ty: &Type, object: TokenStream) -> TokenStream {
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
		Description::Alias(a) => {
			let ty = context.layout_type(a.target()).unwrap();
			from_object(context, ty, object)
		}
		Description::Reference(_) => {
			quote! {
				match #object {
					::treeldr_rust_prelude::rdf::Object::Id(id) => ::treeldr_rust_prelude::Id(id.clone()),
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
						::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
					},
					_ => return Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
				}
			}
		}
	}
}

fn from_objects<V, M>(context: &Context<V, M>, ty: &Type, objects: TokenStream) -> TokenStream {
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
		Description::Alias(_) => {
			quote! {
				todo!("alias from RDF")
			}
		}
		_ => panic!("not a collection type"),
	}
}
