use proc_macro2::Ident;
use quote::format_ident;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

use crate::{syntax, Context, Error, GenerateSyntax, Scope};

use super::Parameters;

#[derive(Debug)]
pub struct Alias {
	ident: Ident,
	layout: TId<treeldr::Layout>,
	target: TId<treeldr::Layout>,
	params: Parameters,
}

impl Alias {
	pub fn new(ident: Ident, layout: TId<treeldr::Layout>, target: TId<treeldr::Layout>) -> Self {
		Self {
			ident,
			layout,
			target,
			params: Parameters::default(),
		}
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn layout(&self) -> TId<treeldr::Layout> {
		self.layout
	}

	pub fn target(&self) -> TId<treeldr::Layout> {
		self.target
	}

	pub fn params(&self) -> Parameters {
		self.params
	}

	pub(crate) fn set_params(&mut self, p: Parameters) {
		self.params = p
	}

	pub(crate) fn compute_params(
		&self,
		mut dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		dependency_params(self.target)
	}
}

impl<M> GenerateSyntax<M> for Alias {
	type Output = syntax::Alias;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
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

		let target = self.target().generate_syntax(context, &scope)?;

		Ok(syntax::Alias {
			ident: self.ident().clone(),
			params,
			target,
		})
	}
}
