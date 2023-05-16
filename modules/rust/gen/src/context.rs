use crate::{
	module::{self, TraitId},
	tr::{Trait, TraitModules},
	ty::{self, params::Parameters},
	Module, Path, Type,
};
use proc_macro2::Ident;
use quote::format_ident;
use rdf_types::Vocabulary;
use shelves::{Ref, Shelf};
use std::collections::{BTreeMap, HashMap, HashSet};
use treeldr::{value::Literal, IriIndex, TId};

#[derive(Debug, Clone, Copy)]
pub struct Options {
	pub impl_rdf: bool,
}

impl Default for Options {
	fn default() -> Self {
		Self { impl_rdf: true }
	}
}

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

	options: Options,
}

impl<'a, V, M> Context<'a, V, M> {
	pub fn new(model: &'a treeldr::Model<M>, vocabulary: &'a V, options: Options) -> Self {
		Self {
			model,
			vocabulary,
			modules: Shelf::default(),
			layouts: BTreeMap::default(),
			types: BTreeMap::default(),
			anonymous_types: 0,
			options,
		}
	}

	pub fn options(&self) -> &Options {
		&self.options
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

	pub fn module_mut(&mut self, r: Ref<Module>) -> Option<&mut Module> {
		self.modules.get_mut(r)
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
		extern_path: Option<module::ExternPath>,
		ident: proc_macro2::Ident,
		visibility: impl Into<syn::Visibility>,
	) -> Ref<Module> {
		let r = self
			.modules
			.insert(Module::new(parent, extern_path, ident, visibility));
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

	pub fn add_type(&mut self, modules: TraitModules, type_ref: TId<treeldr::Type>) -> bool
	where
		V: Vocabulary<Iri = IriIndex>,
	{
		match Trait::build(self, modules, type_ref) {
			Some(tr) => {
				self.types.insert(type_ref, tr);

				if let Some(module::Parent::Ref(module)) = modules.main {
					self.modules
						.get_mut(module)
						.expect("undefined module")
						.types_mut()
						.insert(type_ref);
				}

				if let Some(module::Parent::Ref(module)) = modules.provider {
					self.modules
						.get_mut(module)
						.expect("undefined module")
						.types_providers_mut()
						.insert(crate::tr::ProviderOf(type_ref));
				}

				if let Some(module::Parent::Ref(module)) = modules.trait_object {
					self.modules
						.get_mut(module)
						.expect("undefined module")
						.types_trait_objects_mut()
						.insert(crate::tr::TraitObjectsOf(type_ref));
				}

				true
			}
			None => false,
		}
	}

	pub fn run_pre_computations(&mut self) {
		self.compute_type_params();
		self.dispatch_trait_implementations()
	}

	pub fn dispatch_trait_implementations(&mut self) {
		let mut trait_impls = HashSet::new();
		for ty in self.layouts.values() {
			ty.collect_trait_implementations(self, |i| trait_impls.insert(i))
		}

		for i in trait_impls {
			let module_ref = match i.tr {
				TraitId::Class(tr) => {
					let tr_module = self.types.get(&tr).and_then(|tr| tr.module());
					let ty_module = self.layouts.get(&i.ty).and_then(|ty| ty.module());

					match (tr_module, ty_module) {
						(Some(module::Parent::Ref(a)), Some(module::Parent::Ref(b))) => {
							Some(std::cmp::max(a, b))
						}
						(Some(module::Parent::Ref(a)), _) => Some(a),
						(_, Some(module::Parent::Ref(b))) => Some(b),
						_ => None,
					}
				}
				_ => self
					.layouts
					.get(&i.ty)
					.and_then(|ty| ty.module())
					.and_then(module::Parent::into_ref),
			};

			if let Some(module_ref) = module_ref {
				let module = self.module_mut(module_ref).unwrap();
				module.trait_impls_mut().insert(i);
			}
		}
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

pub struct ModulePathBuilder {
	root: Ref<Module>,
	by_path: HashMap<String, HashMap<Option<DedicatedSubModule>, Ref<Module>>>,
}

fn path_delimiter(c: char) -> bool {
	matches!(c, '/' | ':' | '.')
}

impl ModulePathBuilder {
	pub const DELIMITER: fn(char) -> bool = path_delimiter;

	pub fn new(root: Ref<Module>) -> Self {
		Self {
			root,
			by_path: HashMap::new(),
		}
	}

	pub fn split_iri_path(iri: &str) -> (&str, &str) {
		iri.rsplit_once('#')
			.or_else(|| iri.rsplit_once(Self::DELIMITER))
			.unwrap_or(("", iri))
	}

	pub fn get<V, M>(
		&mut self,
		context: &mut Context<V, M>,
		path: &str,
		dedicated_submodule: Option<DedicatedSubModule>,
	) -> Ref<Module> {
		if let Some(s) = self.by_path.get(path) {
			if let Some(r) = s.get(&dedicated_submodule) {
				return *r;
			}
		}

		let (parent, name) = match dedicated_submodule {
			Some(d) => (self.get(context, path, None), d.name()),
			None => match path.rsplit_once(Self::DELIMITER) {
				Some((prefix, name)) => (self.get(context, prefix, None), name),
				None => (self.root, path),
			},
		};

		let r = if name.is_empty() {
			parent
		} else {
			let name = treeldr::Name::new(name).unwrap();
			let ident =
				proc_macro2::Ident::new(&name.to_snake_case(), proc_macro2::Span::call_site());

			context.add_module(Some(parent), None, ident, module::Visibility::Public)
		};

		self.by_path
			.entry(path.to_string())
			.or_default()
			.insert(dedicated_submodule, r);
		r
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DedicatedSubModule {
	ClassProviders,
	TraitObjects,
	Layouts,
}

impl DedicatedSubModule {
	fn name(&self) -> &'static str {
		match self {
			Self::ClassProviders => "provider",
			Self::TraitObjects => "trait_object",
			Self::Layouts => "layout",
		}
	}
}