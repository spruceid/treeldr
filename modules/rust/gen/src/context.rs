use crate::{
	module,
	tr::Trait,
	ty::{self, params::Parameters},
	Module, Path, Type,
};
use proc_macro2::Ident;
use quote::format_ident;
use rdf_types::Vocabulary;
use shelves::{Ref, Shelf};
use std::collections::{BTreeMap, HashMap};
use treeldr::{value::Literal, IriIndex, TId};

/// Rust context.
pub struct Context<'a, V, M> {
	/// TreeLDR model.
	model: &'a treeldr::Model<M>,

	/// Vocabulary.
	vocabulary: &'a V,

	/// Rust modules.
	modules: Shelf<Vec<Module>>,

	/// Maps each TreeLDR layout to its Rust type.
	layouts: BTreeMap<TId<treeldr::Layout>, Type>,

	types: BTreeMap<TId<treeldr::Type>, Trait>,

	anonymous_types: usize,
}

impl<'a, V, M> Context<'a, V, M> {
	pub fn new(model: &'a treeldr::Model<M>, vocabulary: &'a V) -> Self {
		Self {
			model,
			vocabulary,
			modules: Shelf::default(),
			layouts: BTreeMap::default(),
			types: BTreeMap::default(),
			anonymous_types: 0,
		}
	}

	pub fn next_anonymous_type_ident(&mut self) -> Ident {
		let i = self.anonymous_types;
		self.anonymous_types += 1;
		format_ident!("Anonymous{i}")
	}

	pub fn model(&self) -> &'a treeldr::Model<M> {
		self.model
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

	pub fn type_trait(&self, r: TId<treeldr::Type>) -> Option<&Trait> {
		self.types.get(&r)
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
		let layout = self.model().get(layout_ref).expect("undefined layout");
		let label = layout.preferred_label().map(Literal::to_string);
		let doc = layout.comment().clone_stripped();

		let desc = ty::Description::new(self, layout_ref);
		self.layouts
			.insert(layout_ref, Type::new(module, desc, label, doc));
		if let Some(module::Parent::Ref(module)) = module {
			self.modules
				.get_mut(module)
				.expect("undefined module")
				.layouts_mut()
				.insert(layout_ref);
		}
	}

	pub fn add_type(&mut self, module: Option<module::Parent>, type_ref: TId<treeldr::Type>) -> bool
	where
		V: Vocabulary<Iri = IriIndex>,
	{
		match Trait::build(self, module, type_ref) {
			Some(tr) => {
				self.types.insert(type_ref, tr);
				if let Some(module::Parent::Ref(module)) = module {
					self.modules
						.get_mut(module)
						.expect("undefined module")
						.types_mut()
						.insert(type_ref);
				}
				true
			}
			None => false,
		}
	}

	pub fn run_pre_computations(&mut self) {
		self.compute_type_params()
	}

	pub fn compute_type_params(&mut self) {
		let mut map = HashMap::new();

		fn compute(
			layouts: &BTreeMap<TId<treeldr::Layout>, Type>,
			map: &mut HashMap<TId<treeldr::Layout>, Option<Parameters>>,
			layout_ref: TId<treeldr::Layout>,
		) -> Parameters {
			match map.get(&layout_ref).copied() {
				Some(p) => p.unwrap_or_default(),
				None => {
					map.insert(layout_ref, None);
					let ty = layouts.get(&layout_ref).unwrap();
					let p = ty.compute_params(|d| compute(layouts, map, d));
					map.insert(layout_ref, Some(p));
					p
				}
			}
		}

		for layout_ref in self.layouts.keys().copied() {
			compute(&self.layouts, &mut map, layout_ref);
		}

		for (layout_ref, ty) in &mut self.layouts {
			ty.set_params(map.get(layout_ref).unwrap().as_ref().copied().unwrap())
		}
	}
}
