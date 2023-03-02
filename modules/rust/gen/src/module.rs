use crate::{path::Segment, Context, Error, Generate, Path};
use derivative::Derivative;
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
}

impl Module {
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

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum Parent {
	/// The parent module is unreachable.
	Extern,
	Ref(Ref<Module>),
}
