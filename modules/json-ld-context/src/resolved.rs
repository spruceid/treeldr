use std::{
	collections::{BTreeMap, HashMap, HashSet},
	ops::{Deref, DerefMut},
};

use json_ld::{
	context::Nest, syntax::context::term_definition::Index, Direction, LenientLanguageTagBuf,
	Nullable,
};
use shelves::{Ref, Shelf};
use treeldr::{BlankIdIndex, IriIndex};

mod build;
mod compare;

pub use compare::*;

#[derive(Debug, Clone)]
pub struct TermDefinition {
	pub id: Option<json_ld::Term<IriIndex, BlankIdIndex>>,
	pub type_: Option<Nullable<json_ld::Type<IriIndex>>>,
	pub container: Option<Nullable<json_ld::Container>>,
	pub index: Option<Index>,
	pub language: Option<Nullable<LenientLanguageTagBuf>>,
	pub direction: Option<Nullable<Direction>>,
	pub protected: bool,
	pub prefix: bool,
	pub reverse: bool,
	pub nest: Option<Nest>,
	pub context: Ref<LocalContext>,
}

pub struct LocalContext {
	pub base: Option<Nullable<IriIndex>>,
	pub vocab: Option<Nullable<json_ld::Term<IriIndex, BlankIdIndex>>>,
	pub language: Option<Nullable<LenientLanguageTagBuf>>,
	pub direction: Option<Nullable<Direction>>,
	pub propagate: bool,

	// Internal flag signaling a local context intended as type scoped context.
	pub type_scoped: bool,
}

impl Default for LocalContext {
	fn default() -> Self {
		Self {
			base: None,
			vocab: None,
			language: None,
			direction: None,
			propagate: true,
			type_scoped: false,
		}
	}
}
#[derive(Default)]
pub struct Bindings(BTreeMap<String, TermBindings>);

pub struct TermBindings {
	pub removed: bool,
	pub added: Vec<TermDefinition>,
}

impl Bindings {
	pub fn new(map: BTreeMap<String, TermBindings>) -> Self {
		Self(map)
	}
}

impl Deref for Bindings {
	type Target = BTreeMap<String, TermBindings>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Bindings {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'a> IntoIterator for &'a Bindings {
	type Item = (&'a String, &'a TermBindings);
	type IntoIter = std::collections::btree_map::Iter<'a, String, TermBindings>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

pub struct LocalContexts {
	contexts: Shelf<Vec<LocalContext>>,
	definitions: HashMap<Ref<LocalContext>, Bindings>,
	relations: Relations,
}

impl LocalContexts {
	pub fn new(
		contexts: Shelf<Vec<LocalContext>>,
		definitions: HashMap<Ref<LocalContext>, Bindings>,
		relations: Relations,
	) -> Self {
		Self {
			contexts,
			definitions,
			relations,
		}
	}

	pub fn definitions(&self, r: Ref<LocalContext>) -> Option<&Bindings> {
		self.definitions.get(&r)
	}

	pub fn compute_accessible_bindings_for(&self, r: Ref<LocalContext>) -> AccessibleBindings {
		let mut result = AccessibleBindings::new();

		let mut visited = HashSet::new();

		let mut stack = vec![r];
		while let Some(r) = stack.pop() {
			if visited.insert(r) {
				if let Some(bindings) = self.definitions.get(&r) {
					for (t, defs) in bindings {
						result.entry(t).or_insert(&defs.added);
					}
				}

				stack.extend(self.relations.parents(r));
			}
		}

		result
	}

	pub fn compute_accessible_bindings(&self) -> HashMap<Ref<LocalContext>, AccessibleBindings> {
		let mut result = HashMap::new();

		for (r, _) in &self.contexts {
			result.insert(r, self.compute_accessible_bindings_for(r));
		}

		result
	}
}

pub type AccessibleBindings<'a> = HashMap<&'a str, &'a [TermDefinition]>;

/// Context hierarchy relation.
///
/// For each local context `a`, stores all the contexts `b` for which
/// `a` can be a parent of `b` through a given term.
#[derive(Debug, Default)]
pub struct Relations {
	map: HashMap<Ref<LocalContext>, LocalContextRelations>,
}

impl Relations {
	pub fn new(map: HashMap<Ref<LocalContext>, LocalContextRelations>) -> Self {
		Self { map }
	}

	pub fn contains(
		&self,
		parent: Ref<LocalContext>,
		term: &str,
		child: Ref<LocalContext>,
	) -> bool {
		parent == child
			|| self
				.map
				.get(&parent)
				.and_then(|r| r.children.get(term))
				.map(|r| r.contains(&child))
				.unwrap_or(false)
	}

	pub fn parents(&self, r: Ref<LocalContext>) -> impl '_ + Iterator<Item = Ref<LocalContext>> {
		self.map
			.get(&r)
			.into_iter()
			.flat_map(|rel| {
				rel.parents
					.values()
					.flat_map(|p| p.iter())
					.chain(rel.direct_parents.iter())
			})
			.copied()
	}
}

#[derive(Debug, Default)]
pub struct LocalContextRelations {
	pub direct_children: HashSet<Ref<LocalContext>>,
	pub direct_parents: HashSet<Ref<LocalContext>>,
	pub children: HashMap<String, HashSet<Ref<LocalContext>>>,
	pub parents: HashMap<String, HashSet<Ref<LocalContext>>>,
}
