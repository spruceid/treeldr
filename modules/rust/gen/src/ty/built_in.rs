use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

use crate::{Context, Error, GenerateSyntax, Referenced, Scope};

use super::Parameters;

#[derive(Debug, Clone, Copy)]
pub enum BuiltIn {
	/// Required type, erased.
	Required(TId<treeldr::Layout>),

	/// Option.
	Option(TId<treeldr::Layout>),

	/// Vec.
	Vec(TId<treeldr::Layout>),

	/// BTreeSet.
	BTreeSet(TId<treeldr::Layout>),

	/// BTreeMap.
	BTreeMap(TId<treeldr::Layout>, TId<treeldr::Layout>),

	/// OneOrMany, for non empty sets.
	OneOrMany(TId<treeldr::Layout>),
}

impl BuiltIn {
	pub fn impl_default(&self) -> bool {
		!matches!(self, Self::Required(_))
	}

	pub(crate) fn compute_params(
		&self,
		mut dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		match self {
			Self::BTreeSet(i) => dependency_params(*i),
			Self::BTreeMap(k, v) => dependency_params(*k).union_with(dependency_params(*v)),
			Self::OneOrMany(i) => dependency_params(*i),
			Self::Option(i) => dependency_params(*i),
			Self::Required(i) => dependency_params(*i),
			Self::Vec(i) => dependency_params(*i),
		}
	}
}

impl<M> GenerateSyntax<M> for BuiltIn {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		match self {
			Self::Required(item) => item.generate_syntax(context, scope),
			Self::Option(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(Option<#item>)).unwrap())
			}
			Self::Vec(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(Vec<#item>)).unwrap())
			}
			Self::BTreeSet(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(std::collections::BTreeSet<#item>)).unwrap())
			}
			Self::BTreeMap(key, value) => {
				let key = key.generate_syntax(context, scope)?;
				let value = value.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(std::collections::BTreeMap<#key, #value>)).unwrap())
			}
			Self::OneOrMany(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(::treeldr_rust_prelude::OneOrMany<#item>)).unwrap())
			}
		}
	}
}

impl<M> GenerateSyntax<M> for Referenced<BuiltIn> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		match &self.0 {
			BuiltIn::Required(item) => Referenced(*item).generate_syntax(context, scope),
			BuiltIn::Option(item) => {
				let item = Referenced(*item).generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(Option<#item>)).unwrap())
			}
			BuiltIn::Vec(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(&[#item])).unwrap())
			}
			BuiltIn::BTreeSet(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(&std::collections::BTreeSet<#item>)).unwrap())
			}
			BuiltIn::BTreeMap(key, value) => {
				let key = key.generate_syntax(context, scope)?;
				let value = value.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(&std::collections::BTreeMap<#key, #value>)).unwrap())
			}
			BuiltIn::OneOrMany(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(&::treeldr_rust_prelude::OneOrMany<#item>)).unwrap())
			}
		}
	}
}
