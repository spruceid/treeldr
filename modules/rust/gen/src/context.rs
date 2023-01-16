use crate::{module, ty, Module, Path, Referenced, Type};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use shelves::{Ref, Shelf};
use std::collections::BTreeMap;
use treeldr::{BlankIdIndex, IriIndex, TId};

#[derive(Clone, Copy)]
pub enum IdentType {
	RdfSubject,
}

impl IdentType {
	pub fn for_property<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		context: &Context<V, M>,
		prop_ref: TId<treeldr::Property>,
	) -> TokenStream {
		match self {
			Self::RdfSubject => {
				let prop = context.model().get(prop_ref).unwrap();
				match prop.id() {
					treeldr::Id::Iri(id) => {
						let iri = context.vocabulary().iri(&id).unwrap();
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

pub struct Context<'a, V, M> {
	model: &'a treeldr::MutableModel<M>,
	vocabulary: &'a V,
	modules: Shelf<Vec<Module>>,
	layouts: BTreeMap<TId<treeldr::Layout>, Type>,
	ident_type: IdentType,
}

impl<'a, V, M> Context<'a, V, M> {
	pub fn new(model: &'a treeldr::MutableModel<M>, vocabulary: &'a V) -> Self {
		Self {
			model,
			vocabulary,
			modules: Shelf::default(),
			layouts: BTreeMap::default(),
			ident_type: IdentType::RdfSubject,
		}
	}

	pub fn model(&self) -> &'a treeldr::MutableModel<M> {
		self.model
	}

	pub fn ident_type(&self) -> IdentType {
		self.ident_type
	}

	pub fn vocabulary(&self) -> &'a V {
		self.vocabulary
	}

	pub fn module(&self, r: Ref<Module>) -> Option<&Module> {
		self.modules.get(r)
	}

	pub fn module_path(&self, r: Option<Ref<Module>>) -> Path {
		match r {
			Some(module_ref) => self
				.module(module_ref)
				.expect("undefined module")
				.path(self),
			None => Path::new(),
		}
	}

	pub fn parent_module_path(&self, r: Option<module::Parent>) -> Option<Path> {
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

	pub fn layout_type(&self, r: TId<treeldr::Layout>) -> Option<&Type> {
		self.layouts.get(&r)
	}

	pub fn add_module(
		&mut self,
		parent: Option<Ref<Module>>,
		ident: proc_macro2::Ident,
	) -> Ref<Module> {
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

	pub fn add_layout(&mut self, module: Option<module::Parent>, layout_ref: TId<treeldr::Layout>) {
		let layout = self
			.model()
			.get(layout_ref)
			.expect("undefined described layout");
		let label = layout.preferred_label().map(String::from);
		let doc = layout.comment().clone_stripped();

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
