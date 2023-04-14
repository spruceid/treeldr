use std::collections::HashSet;

use crate::{
	doc_attribute,
	tr::{CollectContextBounds, ContextBound},
	Context, Error, GenerateIn, Module,
};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{vocab, BlankIdIndex, Id, IriIndex, Name, TId};

use super::{params::ParametersValues, Parameters};

#[derive(Debug)]
pub struct Struct {
	layout: TId<treeldr::Layout>,
	ident: proc_macro2::Ident,
	fields: Vec<Field>,
	params: Parameters,
}

impl Struct {
	pub fn new(
		layout: TId<treeldr::Layout>,
		ident: proc_macro2::Ident,
		fields: Vec<Field>,
	) -> Self {
		Self {
			layout,
			ident,
			fields,
			params: Parameters::default(),
		}
	}

	pub fn layout(&self) -> TId<treeldr::Layout> {
		self.layout
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn params(&self) -> Parameters {
		self.params
	}

	pub(crate) fn set_params(&mut self, p: Parameters) {
		self.params = p
	}

	pub fn fields(&self) -> &[Field] {
		&self.fields
	}

	pub fn self_field(&self) -> Option<&Field> {
		self.fields.iter().find(|f| f.is_self())
	}

	pub fn impl_default<V, M>(&self, context: &Context<V, M>) -> bool {
		self.fields
			.iter()
			.all(|f| f.ty(context).impl_default(context))
	}

	pub fn field_for(&self, p: TId<treeldr::Property>) -> Option<&Field> {
		self.fields.iter().find(|f| f.property() == Some(p))
	}

	pub(crate) fn compute_params(
		&self,
		mut dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		let mut result = Parameters::default();

		for f in &self.fields {
			result.append(dependency_params(f.layout))
		}

		result
	}
}

impl CollectContextBounds for Struct {
	fn collect_context_bounds_from<V, M>(
		&self,
		context: &Context<V, M>,
		tr: TId<treeldr::Type>,
		visited: &mut HashSet<TId<treeldr::Layout>>,
		f: &mut impl FnMut(ContextBound),
	) {
		for field in self.fields() {
			if let Some(p) = field.property() {
				let prop = context.model().get(p).unwrap();
				for domain in prop.as_property().domain() {
					if context.model().is_subclass_of_or_eq(**domain.value, tr) {
						for range in prop.as_property().range() {
							field.layout().collect_context_bounds_from(
								context,
								**range.value,
								visited,
								f,
							);
						}
						break;
					}
				}
			}
		}
	}
}

// #[derive(Derivative)]
// #[derivative(Clone(bound = ""), Copy(bound = ""))]
// pub struct FieldType<M> {
// 	layout: TId<treeldr::Layout>,
// }

// impl<M> FieldType<M> {
// 	pub fn new(layout: TId<treeldr::Layout>) -> Self {
// 		Self { layout }
// 	}

// 	pub fn layout(&self) -> TId<treeldr::Layout> {
// 		self.layout
// 	}

// 	pub fn ty<'c>(&self, context: &'c Context<M>) -> &'c super::Type<M> {
// 		context.layout_type(self.layout).unwrap()
// 	}

// 	pub fn impl_default(&self, context: &Context<M>) -> bool {
// 		self.ty(context).impl_default(context)
// 	}
// }

// impl<M> Generate<M> for FieldType<M> {
// 	fn generate(
// 		&self,
// 		context: &Context<M>,
// 		scope: Option<Ref<Module>>,
// 		tokens: &mut TokenStream,
// 	) -> Result<(), Error> {
// 		let layout = self.layout.with(context, scope).into_tokens()?;

// 		tokens.extend(layout);

// 		Ok(())
// 	}
// }

#[derive(Debug)]
pub struct Field {
	name: Name,
	ident: proc_macro2::Ident,
	layout: TId<treeldr::Layout>,
	prop: Option<TId<treeldr::Property>>,
	label: Option<String>,
	doc: treeldr::StrippedDocumentation,
}

impl Field {
	pub fn new(
		name: Name,
		ident: proc_macro2::Ident,
		layout: TId<treeldr::Layout>,
		prop: Option<TId<treeldr::Property>>,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
	) -> Self {
		Self {
			name,
			ident,
			layout,
			prop,
			label,
			doc,
		}
	}

	pub fn is_self(&self) -> bool {
		self.prop
			.map(|p| p.id() == Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Self_))))
			.unwrap_or(false)
	}

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn layout(&self) -> TId<treeldr::Layout> {
		self.layout
	}

	pub fn ty<'c, V, M>(&self, context: &'c Context<V, M>) -> &'c super::Type {
		context.layout_type(self.layout).unwrap()
	}

	pub fn property(&self) -> Option<TId<treeldr::Property>> {
		self.prop
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}
}

impl<M> GenerateIn<M> for Field {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ident = &self.ident;
		let ty = self
			.layout
			.generate_in_with(context, scope, params)
			.into_tokens()?;
		let doc = doc_attribute(self.label(), self.documentation());

		tokens.extend(quote! {
			#(#doc)*
			pub #ident: #ty
		});

		Ok(())
	}
}
