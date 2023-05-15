use std::collections::HashSet;

use crate::{
	syntax,
	tr::{CollectContextBounds, ContextBound},
	Context, Error, GenerateSyntax, Scope,
};
use proc_macro2::Ident;
use quote::format_ident;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

use super::Parameters;

/// Rust `enum` type.
#[derive(Debug)]
pub struct Enum {
	layout: TId<treeldr::Layout>,
	ident: Ident,
	variants: Vec<Variant>,
	params: Parameters,
}

impl Enum {
	pub fn new(layout: TId<treeldr::Layout>, ident: Ident, variants: Vec<Variant>) -> Self {
		Self {
			layout,
			ident,
			variants,
			params: Parameters::default(),
		}
	}

	pub fn layout(&self) -> TId<treeldr::Layout> {
		self.layout
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn params(&self) -> Parameters {
		self.params
	}

	pub(crate) fn set_params(&mut self, p: Parameters) {
		self.params = p
	}

	pub fn variants(&self) -> &[Variant] {
		&self.variants
	}

	pub(crate) fn compute_params(
		&self,
		mut dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		let mut result = Parameters::default();

		for v in &self.variants {
			if let Some(layout_ref) = v.ty() {
				result.append(dependency_params(layout_ref))
			}
		}

		result
	}
}

impl CollectContextBounds for Enum {
	fn collect_context_bounds_from<V, M>(
		&self,
		context: &Context<V, M>,
		tr: TId<treeldr::Type>,
		visited: &mut HashSet<TId<treeldr::Layout>>,
		f: &mut impl FnMut(ContextBound),
	) {
		for variant in self.variants() {
			if let Some(l) = variant.ty() {
				l.collect_context_bounds_from(context, tr, visited, f)
			}
		}
	}
}

#[derive(Debug)]
pub struct Variant {
	ident: Ident,
	ty: Option<TId<treeldr::Layout>>,
}

impl Variant {
	pub fn new(ident: Ident, ty: Option<TId<treeldr::Layout>>) -> Self {
		Self { ident, ty }
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn ty(&self) -> Option<TId<treeldr::Layout>> {
		self.ty
	}
}

impl<M> GenerateSyntax<M> for Enum {
	type Output = syntax::Enum;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let ident = self.ident().clone();

		let params = syntax::LayoutParameters {
			identifier: if self.params().identifier {
				Some(format_ident!("I"))
			} else {
				None
			},
		};

		let mut scope = scope.clone();
		scope.params.identifier = params.identifier.clone().map(|i| {
			syn::Type::Path(syn::TypePath {
				qself: None,
				path: i.into(),
			})
		});

		let derives = syntax::Derives {
			clone: true,
			partial_eq: true,
			eq: true,
			ord: true,
			debug: true,
			..Default::default()
		};

		let mut variants = Vec::with_capacity(self.variants().len());

		for variant in self.variants() {
			variants.push(variant.generate_syntax(context, &scope)?)
		}

		Ok(syntax::Enum {
			derives,
			ident,
			params,
			variants,
		})
	}
}

impl<M> GenerateSyntax<M> for Variant {
	type Output = syntax::Variant;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		Ok(syntax::Variant {
			ident: self.ident.clone(),
			type_: self
				.ty
				.as_ref()
				.map(|ty| ty.generate_syntax(context, scope))
				.transpose()?,
		})
	}
}
