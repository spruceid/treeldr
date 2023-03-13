use crate::{path::Segment, Context, Error, Generate, Path};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use shelves::Ref;
use std::collections::HashSet;
use treeldr::{BlankIdIndex, IriIndex, TId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraitId {
	FromRdf,
	TriplesAndValues,
	IntoJsonLd,
	Defined(TId<treeldr::Type>)
}

impl TraitId {
	pub fn impl_for(self, ty: TId<treeldr::Layout>) -> TraitImpl {
		TraitImpl::new(ty, self)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitImpl {
	pub ty: TId<treeldr::Layout>,
	pub tr: TraitId,
}

impl TraitImpl {
	pub fn new(
		ty: TId<treeldr::Layout>,
		tr: TraitId,
	) -> Self {
		Self {
			ty,
			tr
		}
	}
}

pub struct Module {
	parent: Option<Ref<Self>>,
	ident: proc_macro2::Ident,
	sub_modules: HashSet<Ref<Self>>,
	layouts: HashSet<TId<treeldr::Layout>>,
	types: HashSet<TId<treeldr::Type>>,
	trait_impls: HashSet<TraitImpl>
}

impl Module {
	pub fn new(parent: Option<Ref<Self>>, ident: proc_macro2::Ident) -> Self {
		Self {
			parent,
			ident,
			sub_modules: HashSet::new(),
			layouts: HashSet::new(),
			types: HashSet::new(),
			trait_impls: HashSet::new()
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

	pub fn trait_impls(&self) -> &HashSet<TraitImpl> {
		&self.trait_impls
	}

	pub fn trait_impls_mut(&mut self) -> &mut HashSet<TraitImpl> {
		&mut self.trait_impls
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

		for trait_impl in &self.trait_impls {
			trait_impl.generate(context, scope, tokens)?
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

impl Parent {
	pub fn into_ref(self) -> Option<Ref<Module>> {
		match self {
			Self::Extern => None,
			Self::Ref(r) => Some(r)
		}
	}
}