use crate::{module, ty, Module, Path, Referenced, Type};
use proc_macro2::TokenStream;
use quote::quote;
use shelves::{Ref, Shelf};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum IdentType {
	RdfSubject,
}

impl IdentType {
	pub fn for_property<M>(
		&self,
		context: &Context<M>,
		prop_ref: Ref<treeldr::prop::Definition<M>>,
	) -> TokenStream {
		match self {
			Self::RdfSubject => {
				let prop = context.model().properties().get(prop_ref).unwrap();
				match prop.id() {
					treeldr::Id::Iri(id) => {
						let iri = id.iri(context.vocabulary()).unwrap();
						let iri_literal = iri.as_str();
						quote! { ::treeldr_rust_prelude::rdf::SubjectRef::Iri(::treeldr_rust_prelude::static_iref::iri!(#iri_literal)) }
					}
					treeldr::Id::Blank(_) => panic!("cannot generate static id for blank property"),
				}
			}
		}
	}

	pub fn path(&self) -> TokenStream {
		quote! {
			::treeldr_rust_prelude::rdf::Subject
		}
	}
}

impl Referenced<IdentType> {
	pub fn path(&self) -> TokenStream {
		quote! {
			::treeldr_rust_prelude::rdf::SubjectRef
		}
	}
}

impl quote::ToTokens for IdentType {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		tokens.extend(self.path())
	}
}

impl quote::ToTokens for Referenced<IdentType> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		tokens.extend(self.path())
	}
}

pub struct Context<'a, M> {
	model: &'a treeldr::Model<M>,
	vocabulary: &'a treeldr::Vocabulary,
	modules: Shelf<Vec<Module<M>>>,
	layouts: shelves::Map<treeldr::layout::Definition<M>, HashMap<usize, Type<M>>>,
	ident_type: IdentType,
}

impl<'a, M> Context<'a, M> {
	pub fn new(model: &'a treeldr::Model<M>, vocabulary: &'a treeldr::Vocabulary) -> Self {
		Self {
			model,
			vocabulary,
			modules: Shelf::default(),
			layouts: shelves::Map::default(),
			ident_type: IdentType::RdfSubject,
		}
	}

	pub fn model(&self) -> &'a treeldr::Model<M> {
		self.model
	}

	pub fn ident_type(&self) -> IdentType {
		self.ident_type
	}

	pub fn vocabulary(&self) -> &'a treeldr::Vocabulary {
		self.vocabulary
	}

	pub fn module(&self, r: Ref<Module<M>>) -> Option<&Module<M>> {
		self.modules.get(r)
	}

	pub fn module_path(&self, r: Option<Ref<Module<M>>>) -> Path {
		match r {
			Some(module_ref) => self
				.module(module_ref)
				.expect("undefined module")
				.path(self),
			None => Path::new(),
		}
	}

	pub fn parent_module_path(&self, r: Option<module::Parent<M>>) -> Option<Path> {
		match r {
			Some(module::Parent::Extern) => None,
			Some(module::Parent::Ref(module_ref)) => Some(
				self.module(module_ref)
					.expect("undefined module")
					.path(self),
			),
			None => Some(Path::new()),
		}
	}

	pub fn layout_type(&self, r: Ref<treeldr::layout::Definition<M>>) -> Option<&Type<M>> {
		self.layouts.get(r)
	}

	pub fn add_module(
		&mut self,
		parent: Option<Ref<Module<M>>>,
		ident: proc_macro2::Ident,
	) -> Ref<Module<M>> {
		let r = self.modules.insert(Module::new(parent, ident));
		if let Some(parent) = parent {
			self.modules
				.get_mut(parent)
				.expect("undefined parent module")
				.sub_modules_mut()
				.insert(r);
		}
		r
	}

	pub fn add_layout(
		&mut self,
		module: Option<module::Parent<M>>,
		layout_ref: Ref<treeldr::layout::Definition<M>>,
	) {
		let layout = self
			.model()
			.layouts()
			.get(layout_ref)
			.expect("undefined described layout");
		let label = layout.preferred_label(self.model()).map(String::from);
		let doc = layout.preferred_documentation(self.model()).clone();

		self.layouts.insert(
			layout_ref,
			Type::new(module, ty::Description::new(self, layout_ref), label, doc),
		);
		if let Some(module::Parent::Ref(module)) = module {
			self.modules
				.get_mut(module)
				.expect("undefined module")
				.layouts_mut()
				.insert(layout_ref);
		}
	}
}
