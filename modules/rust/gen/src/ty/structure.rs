use crate::{Context, Error, Generate, Module};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex, Name};

pub struct Struct<M> {
	ident: proc_macro2::Ident,
	fields: Vec<Field<M>>,
}

impl<M> Struct<M> {
	pub fn new(ident: proc_macro2::Ident, fields: Vec<Field<M>>) -> Self {
		Self { ident, fields }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn fields(&self) -> &[Field<M>] {
		&self.fields
	}

	pub fn impl_default<V>(&self, context: &Context<V, M>) -> bool {
		self.fields
			.iter()
			.all(|f| f.ty(context).impl_default(context))
	}
}

// #[derive(Derivative)]
// #[derivative(Clone(bound = ""), Copy(bound = ""))]
// pub struct FieldType<M> {
// 	layout: Ref<treeldr::layout::Definition<M>>,
// }

// impl<M> FieldType<M> {
// 	pub fn new(layout: Ref<treeldr::layout::Definition<M>>) -> Self {
// 		Self { layout }
// 	}

// 	pub fn layout(&self) -> Ref<treeldr::layout::Definition<M>> {
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
// 		scope: Option<Ref<Module<M>>>,
// 		tokens: &mut TokenStream,
// 	) -> Result<(), Error<M>> {
// 		let layout = self.layout.with(context, scope).into_tokens()?;

// 		tokens.extend(layout);

// 		Ok(())
// 	}
// }

pub struct Field<M> {
	name: Name,
	ident: proc_macro2::Ident,
	layout: Ref<treeldr::layout::Definition<M>>,
	prop: Option<Ref<treeldr::prop::Definition<M>>>,
	label: Option<String>,
	doc: treeldr::Documentation,
}

impl<M> Field<M> {
	pub fn new(
		name: Name,
		ident: proc_macro2::Ident,
		layout: Ref<treeldr::layout::Definition<M>>,
		prop: Option<Ref<treeldr::prop::Definition<M>>>,
		label: Option<String>,
		doc: treeldr::Documentation,
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

	pub fn name(&self) -> &Name {
		&self.name
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn layout(&self) -> Ref<treeldr::layout::Definition<M>> {
		self.layout
	}

	pub fn ty<'c, V>(&self, context: &'c Context<V, M>) -> &'c super::Type<M> {
		context.layout_type(self.layout).unwrap()
	}

	pub fn property(&self) -> Option<Ref<treeldr::prop::Definition<M>>> {
		self.prop
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::Documentation {
		&self.doc
	}
}

impl<M> Generate<M> for Field<M> {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module<M>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<M>> {
		let ident = &self.ident;
		let ty = self.layout.with(context, scope).into_tokens()?;
		let doc = super::generate::doc_attribute(self.label(), self.documentation());

		tokens.extend(quote! {
			#(#doc)*
			pub #ident: #ty
		});

		Ok(())
	}
}
