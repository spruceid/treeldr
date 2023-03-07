use crate::{path::Segment, Context, Error, Generate, Path};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use shelves::Ref;
use std::collections::HashSet;
use treeldr::{BlankIdIndex, IriIndex, TId};

pub struct Module {
	parent: Option<Ref<Self>>,
	ident: proc_macro2::Ident,
	sub_modules: HashSet<Ref<Self>>,
	layouts: HashSet<TId<treeldr::Layout>>,
	types: HashSet<TId<treeldr::Type>>,
}

impl Module {
	pub fn new(parent: Option<Ref<Self>>, ident: proc_macro2::Ident) -> Self {
		Self {
			parent,
			ident,
			sub_modules: HashSet::new(),
			layouts: HashSet::new(),
			types: HashSet::new(),
		}
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn path<V, M>(&self, context: &Context<V, M>) -> Path {
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

	pub fn layouts(&self) -> &HashSet<TId<treeldr::Layout>> {
		&self.layouts
	}

	pub fn layouts_mut(&mut self) -> &mut HashSet<TId<treeldr::Layout>> {
		&mut self.layouts
	}

	pub fn types(&self) -> &HashSet<TId<treeldr::Type>> {
		&self.types
	}

	pub fn types_mut(&mut self) -> &mut HashSet<TId<treeldr::Type>> {
		&mut self.types
	}
}

impl<M> Generate<M> for Module {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		for module_ref in &self.sub_modules {
			module_ref.generate(context, scope, tokens)?;
		}

		for type_ref in &self.types {
			let ty = context.type_trait(*type_ref).expect("undefined type");
			ty.generate(context, scope, tokens)?
		}

		for layout_ref in &self.layouts {
			let ty = context.layout_type(*layout_ref).expect("undefined layout");
			ty.generate(context, scope, tokens)?
		}

		Ok(())
	}
}

impl<M> Generate<M> for Ref<Module> {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		_scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let module = context.module(*self).expect("undefined module");
		let ident = module.ident();
		let content = module.generate_with(context, Some(*self)).into_tokens()?;

		tokens.extend(quote! {
			pub mod #ident {
				#content
			}
		});

		Ok(())
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Parent {
	/// The parent module is unreachable.
	Extern,
	Ref(Ref<Module>),
}
