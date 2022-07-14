use crate::{path::Segment, Context, Error, Generate, Path};
use derivative::Derivative;
use proc_macro2::TokenStream;
use quote::quote;
use shelves::Ref;
use std::collections::HashSet;

pub struct Module<F> {
	parent: Option<Ref<Self>>,
	ident: proc_macro2::Ident,
	sub_modules: HashSet<Ref<Self>>,
	layouts: HashSet<Ref<treeldr::layout::Definition<F>>>,
}

impl<F> Module<F> {
	pub fn new(parent: Option<Ref<Self>>, ident: proc_macro2::Ident) -> Self {
		Self {
			parent,
			ident,
			sub_modules: HashSet::new(),
			layouts: HashSet::new(),
		}
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn path(&self, context: &Context<F>) -> Path {
		let mut path = context.module_path(self.parent);
		path.push(Segment::Ident(self.ident.clone()));
		path
	}

	pub fn sub_modules(&self) -> &HashSet<Ref<Self>> {
		&self.sub_modules
	}

	pub fn sub_modules_mut(&mut self) -> &mut HashSet<Ref<Self>> {
		&mut self.sub_modules
	}

	pub fn layouts(&self) -> &HashSet<Ref<treeldr::layout::Definition<F>>> {
		&self.layouts
	}

	pub fn layouts_mut(&mut self) -> &mut HashSet<Ref<treeldr::layout::Definition<F>>> {
		&mut self.layouts
	}
}

impl<F> Generate<F> for Module<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		for module_ref in &self.sub_modules {
			module_ref.generate(context, scope, tokens)?;
		}

		for layout_ref in &self.layouts {
			let ty = context.layout_type(*layout_ref).expect("undefined layout");
			ty.generate(context, scope, tokens)?
		}

		Ok(())
	}
}

impl<F> Generate<F> for Ref<Module<F>> {
	fn generate(
		&self,
		context: &Context<F>,
		_scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		let module = context.module(*self).expect("undefined module");
		let ident = module.ident();
		let content = module.with(context, Some(*self)).into_tokens()?;

		tokens.extend(quote! {
			pub mod #ident {
				#content
			}
		});

		Ok(())
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum Parent<F> {
	/// The parent module is unreachable.
	Extern,
	Ref(Ref<Module<F>>),
}
