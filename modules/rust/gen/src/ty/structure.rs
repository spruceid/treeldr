use crate::{Context, Error, Generate, Module};
use proc_macro2::TokenStream;
use quote::quote;
use shelves::Ref;
use treeldr::Name;

pub struct Struct<F> {
	ident: proc_macro2::Ident,
	fields: Vec<Field<F>>,
}

impl<F> Struct<F> {
	pub fn new(ident: proc_macro2::Ident, fields: Vec<Field<F>>) -> Self {
		Self { ident, fields }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn fields(&self) -> &[Field<F>] {
		&self.fields
	}

	pub fn impl_default(&self, context: &Context<F>) -> bool {
		self.fields
			.iter()
			.all(|f| f.ty(context).impl_default(context))
	}
}

// #[derive(Derivative)]
// #[derivative(Clone(bound = ""), Copy(bound = ""))]
// pub struct FieldType<F> {
// 	layout: Ref<treeldr::layout::Definition<F>>,
// }

// impl<F> FieldType<F> {
// 	pub fn new(layout: Ref<treeldr::layout::Definition<F>>) -> Self {
// 		Self { layout }
// 	}

// 	pub fn layout(&self) -> Ref<treeldr::layout::Definition<F>> {
// 		self.layout
// 	}

// 	pub fn ty<'c>(&self, context: &'c Context<F>) -> &'c super::Type<F> {
// 		context.layout_type(self.layout).unwrap()
// 	}

// 	pub fn impl_default(&self, context: &Context<F>) -> bool {
// 		self.ty(context).impl_default(context)
// 	}
// }

// impl<F> Generate<F> for FieldType<F> {
// 	fn generate(
// 		&self,
// 		context: &Context<F>,
// 		scope: Option<Ref<Module<F>>>,
// 		tokens: &mut TokenStream,
// 	) -> Result<(), Error<F>> {
// 		let layout = self.layout.with(context, scope).into_tokens()?;

// 		tokens.extend(layout);

// 		Ok(())
// 	}
// }

pub struct Field<F> {
	name: Name,
	ident: proc_macro2::Ident,
	layout: Ref<treeldr::layout::Definition<F>>,
	prop: Option<Ref<treeldr::prop::Definition<F>>>,
	label: Option<String>,
	doc: treeldr::Documentation,
}

impl<F> Field<F> {
	pub fn new(
		name: Name,
		ident: proc_macro2::Ident,
		layout: Ref<treeldr::layout::Definition<F>>,
		prop: Option<Ref<treeldr::prop::Definition<F>>>,
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

	pub fn layout(&self) -> Ref<treeldr::layout::Definition<F>> {
		self.layout
	}

	pub fn ty<'c>(&self, context: &'c Context<F>) -> &'c super::Type<F> {
		context.layout_type(self.layout).unwrap()
	}

	pub fn property(&self) -> Option<Ref<treeldr::prop::Definition<F>>> {
		self.prop
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::Documentation {
		&self.doc
	}
}

impl<F> Generate<F> for Field<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
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
