use crate::{Context, Error, Generate, Module};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex, Name, TId};

pub struct Struct {
	ident: proc_macro2::Ident,
	fields: Vec<Field>,
}

impl Struct {
	pub fn new(ident: proc_macro2::Ident, fields: Vec<Field>) -> Self {
		Self { ident, fields }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn fields(&self) -> &[Field] {
		&self.fields
	}

	pub fn impl_default<V, M>(&self, context: &Context<V, M>) -> bool {
		self.fields
			.iter()
			.all(|f| f.ty(context).impl_default(context))
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

impl<M> Generate<M> for Field {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
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
