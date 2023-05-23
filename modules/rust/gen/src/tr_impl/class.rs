use quote::quote;
use treeldr::TId;

use crate::{
	module::{TraitId, TraitImpl},
	syntax,
	tr::ContextBound,
	ty, Context, Error, GenerateSyntax,
};

mod r#enum;
mod primitive;
mod r#struct;

impl<M> GenerateSyntax<M> for ContextBound {
	type Output = syn::TraitBound;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let tr = context.type_trait(self.0).unwrap();
		Ok(syn::TraitBound {
			paren_token: None,
			modifier: syn::TraitBoundModifier::None,
			lifetimes: None,
			path: context
				.module_path(scope.module)
				.to(&tr
					.context_path(context)
					.ok_or(Error::UnreachableTrait(self.0))?)
				.generate_syntax(context, scope)?,
		})
	}
}

impl<M> GenerateSyntax<M> for TraitImpl {
	type Output = Option<syntax::TraitImplementation>;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let ty = context.layout_type(self.ty).unwrap();

		match ty.description() {
			ty::Description::Struct(s) => match self.tr {
				TraitId::FromRdf => super::rdf::from::FromRdfImpl::new(self.ty, s)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::FromRdf)
					.map(Some),
				TraitId::TriplesAndValues => super::rdf::to::RdfQuadsImpl::new(self.ty, s)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::RdfQuads)
					.map(Some),
				TraitId::IntoJsonLd => super::json_ld::IntoJsonLdImpl::new(self.ty, s)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::IntoJsonLd)
					.map(Some),
				TraitId::IntoJsonLdSyntax => super::json_ld::IntoJsonLdSyntaxImpl::new(self.ty, s)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::IntoJsonLdSyntax)
					.map(Some),
				TraitId::Class(tr) => ClassTraitImpl::new(tr, self.ty, s)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::ClassTrait)
					.map(Some),
			},
			ty::Description::Enum(e) => match self.tr {
				TraitId::FromRdf => super::rdf::from::FromRdfImpl::new(self.ty, e)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::FromRdf)
					.map(Some),
				TraitId::TriplesAndValues => super::rdf::to::RdfQuadsImpl::new(self.ty, e)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::RdfQuads)
					.map(Some),
				TraitId::IntoJsonLd => super::json_ld::IntoJsonLdImpl::new(self.ty, e)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::IntoJsonLd)
					.map(Some),
				TraitId::IntoJsonLdSyntax => super::json_ld::IntoJsonLdSyntaxImpl::new(self.ty, e)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::IntoJsonLdSyntax)
					.map(Some),
				TraitId::Class(tr) => ClassTraitImpl::new(tr, self.ty, e)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::ClassTrait)
					.map(Some),
			},
			ty::Description::Primitive(p) => match self.tr {
				TraitId::Class(tr) => ClassTraitImpl::new(tr, self.ty, p)
					.generate_syntax(context, scope)
					.map(syntax::TraitImplementation::ClassTrait)
					.map(Some),
				_ => Ok(None),
			},
			_ => {
				panic!("unable to implement trait for non enum/struct type")
			}
		}
	}
}

/// Class trait implementation.
pub struct ClassTraitImpl<'a, T> {
	pub tr_ref: TId<treeldr::Type>,
	pub ty_ref: TId<treeldr::Layout>,
	pub ty: &'a T,
}

impl<'a, T> ClassTraitImpl<'a, T> {
	pub fn new(tr_ref: TId<treeldr::Type>, ty_ref: TId<treeldr::Layout>, ty: &'a T) -> Self {
		Self { tr_ref, ty_ref, ty }
	}
}

fn collection_iterator<V, M>(
	context: &Context<V, M>,
	scope: &crate::Scope,
	collection_layout: TId<treeldr::Layout>,
) -> Result<syn::Type, Error>
where
	V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
{
	let ty = context.layout_type(collection_layout).unwrap();
	match ty.description() {
		ty::Description::BuiltIn(b) => match b {
			ty::BuiltIn::Vec(item) => {
				let item_expr = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(::std::slice::Iter<'r, #item_expr>)).unwrap())
			}
			ty::BuiltIn::Option(item) => {
				let item_expr = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(::std::option::Iter<'r, #item_expr>)).unwrap())
			}
			ty::BuiltIn::BTreeSet(item) => {
				let item_expr = item.generate_syntax(context, scope)?;
				Ok(
					syn::parse2(quote!(::std::collections::btree_set::Iter<'r, #item_expr>))
						.unwrap(),
				)
			}
			ty::BuiltIn::BTreeMap(key, value) => {
				let key_expr = key.generate_syntax(context, scope)?;
				let value_expr = value.generate_syntax(context, scope)?;
				Ok(syn::parse2(
					quote!(::std::collections::btree_map::Iter<'r, #key_expr, #value_expr>),
				)
				.unwrap())
			}
			ty::BuiltIn::OneOrMany(item) => {
				let item_expr = item.generate_syntax(context, scope)?;
				Ok(
					syn::parse2(quote!(::treeldr_rust_prelude::one_or_many::Iter<'r, #item_expr>))
						.unwrap(),
				)
			}
			ty::BuiltIn::Required(_) => panic!("cannot turn required layout into iterator"),
		},
		_ => panic!("cannot turn a non-built-in layout into an iterator"),
	}
}