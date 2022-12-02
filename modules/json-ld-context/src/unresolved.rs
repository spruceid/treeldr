use std::{
	collections::{BTreeMap, HashMap, HashSet},
	ops::{Deref, DerefMut},
};

use contextual::WithContext;
use iref::{IriBuf, IriRefBuf};
use json_ld::{
	context::Nest,
	syntax::context::term_definition::{Index, TypeKeyword},
	Direction, LenientLanguageTagBuf, Nullable,
};
use rdf_types::{BlankIdBuf, IriVocabularyMut, Vocabulary, VocabularyMut};
use shelves::{Ref, Shelf};
use treeldr::{BlankIdIndex, Id, IriIndex};

use crate::resolved;

#[derive(Default, Clone)]
pub struct Prefixes {
	map: HashMap<String, String>,
}

impl Prefixes {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn insert(&mut self, term: String, value: String) {
		self.map.insert(term, value);
	}

	pub fn remove(&mut self, term: &str) {
		self.map.remove(term);
	}

	pub fn intersect_with(&mut self, other: &Self) {
		let map = std::mem::take(&mut self.map);

		for (k, v) in map {
			match other.map.get(&k) {
				Some(w) if *w == v => {
					self.map.insert(k, v);
				}
				_ => (),
			}
		}
	}

	pub fn resolve(
		&self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		term: String,
	) -> Result<Id, String> {
		match term.split_once(':') {
			Some(("_", _)) => match BlankIdBuf::new(term) {
				Ok(b) => Ok(Id::Blank(vocabulary.insert_blank_id(b.as_blank_id_ref()))),
				Err(e) => Err(e.0),
			},
			Some((prefix, suffix)) if !prefix.starts_with("//") => match self.map.get(prefix) {
				Some(value) => {
					let iri = value.clone() + suffix;

					match IriBuf::from_string(iri) {
						Ok(iri) => Ok(Id::Iri(vocabulary.insert(iri.as_iri()))),
						Err(_) => Err(term),
					}
				}
				None => Err(term),
			},
			_ => Err(term),
		}
	}
}

pub trait Resolvable {
	type Unresolved: Resolve<Target = Self>;
}

impl Resolvable for IriIndex {
	type Unresolved = IriRefBuf;
}

impl Resolvable for json_ld::Term<IriIndex, BlankIdIndex> {
	type Unresolved = String;
}

impl Resolvable for json_ld::Type<IriIndex> {
	type Unresolved = json_ld::syntax::context::term_definition::Type;
}

pub trait Resolve {
	type Target;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target;
}

impl<T: Resolvable> Resolve for Unresolved<T> {
	type Target = T;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		match self {
			Self::Resolved(t) => t,
			Self::Unresolved(u) => u.resolve(vocabulary, prefixes, base_iri),
		}
	}
}

impl Resolve for IriRefBuf {
	type Target = IriIndex;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		_prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		match self.as_iri() {
			Ok(iri) => vocabulary.insert(iri),
			Err(_) => match base_iri {
				Some(base) => {
					let base = vocabulary.iri(&base).unwrap();

					let iri = self.resolved(base);
					vocabulary.insert(iri.as_iri())
				}
				None => panic!("missing base IRI"),
			},
		}
	}
}

impl Resolve for String {
	type Target = json_ld::Term<IriIndex, BlankIdIndex>;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		match prefixes.resolve(vocabulary, self) {
			Ok(id) => json_ld::Term::Ref(json_ld::Id::Valid(id)),
			Err(this) => match json_ld::syntax::Keyword::try_from(this.as_str()) {
				Ok(kw) => json_ld::Term::Keyword(kw),
				Err(_) => match IriRefBuf::from_string(this) {
					Ok(iri_ref) => json_ld::Term::Ref(json_ld::Id::Valid(treeldr::Id::Iri(
						iri_ref.resolve(vocabulary, prefixes, base_iri),
					))),
					Err((_, s)) => json_ld::Term::Ref(json_ld::Id::Invalid(s)),
				},
			},
		}
	}
}

impl Resolve for json_ld::syntax::context::term_definition::Type {
	type Target = json_ld::Type<IriIndex>;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		match self {
			Self::Keyword(TypeKeyword::Id) => json_ld::Type::Id,
			Self::Keyword(TypeKeyword::Json) => json_ld::Type::Json,
			Self::Keyword(TypeKeyword::None) => json_ld::Type::None,
			Self::Keyword(TypeKeyword::Vocab) => json_ld::Type::Vocab,
			Self::Term(t) => {
				let iri = match prefixes.resolve(vocabulary, t) {
					Ok(Id::Blank(_)) => {
						panic!("unexpected blank node id")
					}
					Ok(Id::Iri(iri)) => iri,
					Err(t) => match IriRefBuf::from_string(t) {
						Ok(iri_ref) => iri_ref.resolve(vocabulary, prefixes, base_iri),
						Err(_) => {
							panic!("invalid IRI")
						}
					},
				};

				json_ld::Type::Ref(iri)
			}
		}
	}
}

impl<T: Resolve> Resolve for Option<T> {
	type Target = Option<T::Target>;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		self.map(|t| t.resolve(vocabulary, prefixes, base_iri))
	}
}

impl<T: Resolve> Resolve for Nullable<T> {
	type Target = Nullable<T::Target>;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		self.map(|t| t.resolve(vocabulary, prefixes, base_iri))
	}
}

impl<T: Resolve> Resolve for Vec<T> {
	type Target = Vec<T::Target>;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		self.into_iter()
			.map(|t| t.resolve(vocabulary, prefixes, base_iri))
			.collect()
	}
}

pub enum Unresolved<T: Resolvable> {
	Resolved(T),
	Unresolved(T::Unresolved),
}

pub struct TermDefinition {
	pub id: Option<Unresolved<json_ld::Term<IriIndex, BlankIdIndex>>>,
	pub type_: Option<Nullable<Unresolved<json_ld::Type<IriIndex>>>>,
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

impl Default for TermDefinition {
	fn default() -> Self {
		Self {
			id: None,
			type_: None,
			container: None,
			index: None,
			language: None,
			direction: None,
			protected: false,
			prefix: false,
			reverse: false,
			nest: None,
			context: Ref::new(0),
		}
	}
}

impl Resolve for TermDefinition {
	type Target = resolved::TermDefinition;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		resolved::TermDefinition {
			id: self.id.resolve(vocabulary, prefixes, base_iri),
			type_: self.type_.resolve(vocabulary, prefixes, base_iri),
			container: self.container,
			index: self.index,
			language: self.language,
			direction: self.direction,
			protected: self.protected,
			prefix: self.prefix,
			reverse: self.reverse,
			nest: self.nest,
			context: self.context.cast(),
		}
	}
}

pub struct LocalContext {
	pub base: Option<Nullable<Unresolved<IriIndex>>>,
	pub vocab: Option<Nullable<Unresolved<json_ld::Term<IriIndex, BlankIdIndex>>>>,
	pub language: Option<Nullable<LenientLanguageTagBuf>>,
	pub direction: Option<Nullable<Direction>>,
	pub propagate: bool,

	// Internal flag signaling a local context intended as type scoped context.
	pub type_scoped: bool,
}

impl Resolve for LocalContext {
	type Target = resolved::LocalContext;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		resolved::LocalContext {
			base: self.base.resolve(vocabulary, prefixes, base_iri),
			vocab: self.vocab.resolve(vocabulary, prefixes, base_iri),
			language: self.language,
			direction: self.direction,
			propagate: self.propagate,
			type_scoped: self.type_scoped,
		}
	}
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
pub struct Bindings(BTreeMap<String, TermDefinitions>);

impl Bindings {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn insert(&mut self, term: String, def: Nullable<TermDefinition>) {
		self.0.entry(term).or_default().insert(def)
	}
}

impl Resolve for Bindings {
	type Target = resolved::Bindings;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		resolved::Bindings::new(
			self.0
				.into_iter()
				.map(|(k, v)| (k, v.resolve(vocabulary, prefixes, base_iri)))
				.collect(),
		)
	}
}

impl Deref for Bindings {
	type Target = BTreeMap<String, TermDefinitions>;

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
	type Item = (&'a String, &'a TermDefinitions);
	type IntoIter = std::collections::btree_map::Iter<'a, String, TermDefinitions>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

#[derive(Default)]
pub struct TermDefinitions {
	removed: bool,
	added: Vec<TermDefinition>,
}

impl TermDefinitions {
	pub fn insert(&mut self, def: Nullable<TermDefinition>) {
		match def {
			Nullable::Null => self.removed = true,
			Nullable::Some(d) => self.added.push(d),
		}
	}
}

impl Resolve for TermDefinitions {
	type Target = resolved::TermBindings;

	fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		prefixes: &Prefixes,
		base_iri: Option<IriIndex>,
	) -> Self::Target {
		resolved::TermBindings {
			removed: self.removed,
			added: self
				.added
				.into_iter()
				.map(|d| d.resolve(vocabulary, prefixes, base_iri))
				.collect(),
		}
	}
}

#[derive(Default)]
pub struct LocalContexts {
	contexts: Shelf<Vec<LocalContext>>,
	definitions: HashMap<Ref<LocalContext>, Bindings>,
	base_context: Option<(Ref<LocalContext>, HashSet<Ref<LocalContext>>)>,
}

impl LocalContexts {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn empty_context(&mut self) -> Ref<LocalContext> {
		self.insert(LocalContext::default())
	}

	pub fn insert(&mut self, lc: LocalContext) -> Ref<LocalContext> {
		let r = self.contexts.insert(lc);
		self.definitions.insert(r, Bindings::default());
		r
	}

	pub fn set_bindings(&mut self, r: Ref<LocalContext>, bindings: Bindings) {
		self.definitions.insert(r, bindings);
	}

	pub fn add_iri_prefixes(
		&mut self,
		prefixes: &HashMap<String, IriIndex>,
		root: Ref<LocalContext>,
	) {
		let definitions: Vec<_> = prefixes
			.iter()
			.map(|(prefix, value)| {
				(
					prefix.clone(),
					Nullable::Some(TermDefinition {
						id: Some(Unresolved::Resolved(json_ld::Term::Ref(
							json_ld::Id::Valid(Id::Iri(*value)),
						))),
						context: self.empty_context(),
						..Default::default()
					}),
				)
			})
			.collect();

		let bindings = self.definitions.get_mut(&root).unwrap();
		for (prefix, def) in definitions {
			bindings.insert(prefix, def)
		}
	}

	pub fn propagated_sub_contexts(
		&self,
		r: Ref<LocalContext>,
		stop_term: &str,
	) -> HashSet<Ref<LocalContext>> {
		let mut result = HashSet::new();
		self.collect_propagated_sub_contexts(r, stop_term, &mut result);
		result
	}

	pub fn collect_propagated_sub_contexts(
		&self,
		r: Ref<LocalContext>,
		stop_term: &str,
		result: &mut HashSet<Ref<LocalContext>>,
	) {
		if result.insert(r) {
			let context = self.contexts.get(r).unwrap();
			if context.propagate {
				if let Some(bindings) = self.definitions.get(&r) {
					if !bindings.contains_key(stop_term) {
						for (_term, defs) in bindings {
							for def in &defs.added {
								self.collect_propagated_sub_contexts(def.context, stop_term, result)
							}
						}
					}
				}
			}
		}
	}

	pub fn compute_relations(&self) -> Relations {
		let mut result = Relations::default();
		for (parent_ref, context) in &self.contexts {
			if context.propagate {
				if let Some(bindings) = self.definitions.get(&parent_ref) {
					for (term, defs) in bindings {
						for def in &defs.added {
							result.insert(parent_ref, Some(term), def.context);

							// for propagated_ref in self.propagated_sub_contexts(def.context, term) {
							// 	result.insert(propagated_ref, Some(term), def.context);
							// }
						}
					}
				}
			}
		}

		if let Some((base, roots)) = &self.base_context {
			for root in roots {
				result.insert(*base, None, *root);
			}
		}

		// result.close();
		result
	}

	pub fn set_base_context(
		&mut self,
		base: Ref<LocalContext>,
		roots: impl IntoIterator<Item = Ref<LocalContext>>,
	) {
		self.base_context = Some((base, roots.into_iter().collect()))
	}

	fn compute_base_iris_for(
		&self,
		vocabulary: &mut impl IriVocabularyMut<Iri = IriIndex>,
		relations: &Relations,
		state: &mut HashMap<Ref<LocalContext>, Option<Nullable<IriIndex>>>,
		r: Ref<LocalContext>,
	) -> Option<Nullable<IriIndex>> {
		match state.get(&r) {
			Some(Some(iri)) => Some(*iri),
			Some(None) => None,
			None => {
				state.insert(r, None);

				let context = self.contexts.get(r).unwrap();
				let iri = match &context.base {
					Some(Nullable::Null) => Nullable::Null,
					Some(Nullable::Some(Unresolved::Resolved(iri))) => Nullable::Some(*iri),
					Some(Nullable::Some(Unresolved::Unresolved(value))) => match value.as_iri() {
						Ok(iri) => Nullable::Some(vocabulary.insert(iri)),
						Err(_) => {
							let mut parent_iri = None;

							for p in relations.parents(r) {
								if let Some(p_iri) =
									self.compute_base_iris_for(vocabulary, relations, state, p)
								{
									match parent_iri {
										Some(iri) => {
											if iri != p_iri {
												panic!("inconsistent base IRI")
											}
										}
										None => parent_iri = Some(p_iri),
									}
								}
							}

							match parent_iri.unwrap_or(Nullable::Null) {
								Nullable::Null => {
									panic!("no base iri")
								}
								Nullable::Some(parent_iri) => {
									let parent_iri = vocabulary.iri(&parent_iri).unwrap();
									let iri = value.resolved(parent_iri);
									Nullable::Some(vocabulary.insert(iri.as_iri()))
								}
							}
						}
					},
					None => {
						let mut iri = None;

						for p in relations.parents(r) {
							if let Some(p_iri) =
								self.compute_base_iris_for(vocabulary, relations, state, p)
							{
								match iri {
									Some(iri) => {
										if iri != p_iri {
											panic!("inconsistent base IRI")
										}
									}
									None => iri = Some(p_iri),
								}
							}
						}

						iri.unwrap_or(Nullable::Null)
					}
				};

				state.insert(r, Some(iri));
				Some(iri)
			}
		}
	}

	pub fn compute_base_iris(
		&self,
		vocabulary: &mut impl IriVocabularyMut<Iri = IriIndex>,
		relations: &Relations,
	) -> HashMap<Ref<LocalContext>, IriIndex> {
		let mut result = HashMap::new();

		for (r, _) in &self.contexts {
			self.compute_base_iris_for(vocabulary, relations, &mut result, r);
		}

		result
			.into_iter()
			.filter_map(|(r, v)| match v {
				Some(Nullable::Some(v)) => Some((r, v)),
				_ => None,
			})
			.collect()
	}

	fn compute_prefixes_for<'a>(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		relations: &Relations,
		state: &'a mut HashMap<Ref<LocalContext>, Option<Prefixes>>,
		r: Ref<LocalContext>,
	) -> Option<&'a Prefixes> {
		if state.contains_key(&r) {
			return state.get(&r).unwrap().as_ref();
		}

		state.insert(r, None);

		let mut prefixes: Option<Prefixes> = None;

		for p in relations.parents(r) {
			if let Some(p_prefixes) = self.compute_prefixes_for(vocabulary, relations, state, p) {
				match &mut prefixes {
					Some(prefixes) => prefixes.intersect_with(p_prefixes),
					None => prefixes = Some(p_prefixes.clone()),
				}
			}
		}

		let mut prefixes = prefixes.unwrap_or_default();

		for (t, defs) in self.definitions.get(&r).unwrap() {
			let value = 'value: {
				let mut value = None;

				for def in &defs.added {
					let def_value = match &def.id {
						Some(Unresolved::Resolved(json_ld::Term::Ref(r))) => {
							r.with(vocabulary).as_str()
						}
						Some(Unresolved::Unresolved(r)) => r.as_str(),
						_ => break 'value None,
					};

					match value {
						Some(value) => {
							if value != def_value {
								break 'value None;
							}
						}
						None => value = Some(def_value),
					}
				}

				value
			};

			match value {
				Some(value) => prefixes.insert(t.clone(), value.to_string()),
				None => prefixes.remove(t),
			}
		}

		state.insert(r, Some(prefixes));
		state.get(&r).unwrap().as_ref()
	}

	pub fn compute_prefixes(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		relations: &Relations,
	) -> HashMap<Ref<LocalContext>, Prefixes> {
		let mut result = HashMap::new();

		for (r, _) in &self.contexts {
			self.compute_prefixes_for(vocabulary, relations, &mut result, r);
		}

		result.into_iter().map(|(r, v)| (r, v.unwrap())).collect()
	}

	pub fn resolve(
		self,
		vocabulary: &mut impl VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> resolved::LocalContexts {
		let relations = self.compute_relations();
		let base_iris = self.compute_base_iris(vocabulary, &relations);
		let prefixes = self.compute_prefixes(vocabulary, &relations);

		let relations = relations.resolve();
		let contexts = Shelf::new(
			self.contexts
				.into_storage()
				.into_iter()
				.enumerate()
				.map(|(i, context)| {
					let base_iri = base_iris.get(&Ref::new(i)).copied();
					let prefixes = prefixes.get(&Ref::new(i)).unwrap();
					context.resolve(vocabulary, prefixes, base_iri)
				})
				.collect(),
		);
		let bindings = self
			.definitions
			.into_iter()
			.map(|(r, defs)| {
				let base_iri = base_iris.get(&r).copied();
				let prefixes = prefixes.get(&r).unwrap();
				(r.cast(), defs.resolve(vocabulary, prefixes, base_iri))
			})
			.collect();

		resolved::LocalContexts::new(contexts, bindings, relations)
	}
}

/// Context hierarchy relation.
///
/// For each local context `a`, stores all the contexts `b` for which
/// `a` can be a parent of `b` through a given term.
#[derive(Debug, Default)]
pub struct Relations<'a> {
	map: HashMap<Ref<LocalContext>, LocalContextRelations<'a>>,
}

impl<'a> Relations<'a> {
	pub fn insert(
		&mut self,
		parent: Ref<LocalContext>,
		term: Option<&'a str>,
		child: Ref<LocalContext>,
	) -> bool {
		parent != child
			&& match term {
				Some(term) => {
					self.map
						.entry(parent)
						.or_default()
						.children
						.entry(term)
						.or_default()
						.insert(child) && self
						.map
						.entry(child)
						.or_default()
						.parents
						.entry(term)
						.or_default()
						.insert(parent)
				}
				None => {
					self.map
						.entry(parent)
						.or_default()
						.direct_children
						.insert(child) && self
						.map
						.entry(child)
						.or_default()
						.direct_parents
						.insert(parent)
				}
			}
	}

	pub fn contains(
		&self,
		parent: Ref<LocalContext>,
		term: &'a str,
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

	/// Close the relation to include indirect parents/children.
	pub fn close(&mut self) {
		let mut derived = Vec::new();

		let mut stack: Vec<_> = self.map.keys().copied().collect();
		let mut in_stack = HashSet::new();

		while let Some(parent_ref) = stack.pop() {
			in_stack.remove(&parent_ref);

			let relations = self.map.get(&parent_ref).unwrap();
			for (t, children) in &relations.children {
				for child in children {
					if let Some(child_relations) = self.map.get(child) {
						for (u, siblings) in &relations.children {
							if u != t && !child_relations.children.contains_key(u) {
								for sibling in siblings {
									if !self.contains(*child, u, *sibling) {
										derived.push((*child, *u, *sibling))
									}
								}
							}
						}
					}
				}
			}

			while let Some((parent, term, child)) = derived.pop() {
				if self.insert(parent, Some(term), child) && !in_stack.contains(&parent) {
					stack.push(parent);
					in_stack.insert(parent);
				}
			}
		}
	}

	pub fn resolve(self) -> resolved::Relations {
		resolved::Relations::new(
			self.map
				.into_iter()
				.map(|(r, rels)| (r.cast(), rels.resolve()))
				.collect(),
		)
	}
}

#[derive(Debug, Default)]
pub struct LocalContextRelations<'a> {
	direct_children: HashSet<Ref<LocalContext>>,
	direct_parents: HashSet<Ref<LocalContext>>,
	children: HashMap<&'a str, HashSet<Ref<LocalContext>>>,
	parents: HashMap<&'a str, HashSet<Ref<LocalContext>>>,
}

impl<'a> LocalContextRelations<'a> {
	pub fn resolve(self) -> resolved::LocalContextRelations {
		resolved::LocalContextRelations {
			direct_children: self.direct_children.into_iter().map(Ref::cast).collect(),
			direct_parents: self.direct_parents.into_iter().map(Ref::cast).collect(),
			children: self
				.children
				.into_iter()
				.map(|(t, s)| (t.to_string(), s.into_iter().map(Ref::cast).collect()))
				.collect(),
			parents: self
				.parents
				.into_iter()
				.map(|(t, s)| (t.to_string(), s.into_iter().map(Ref::cast).collect()))
				.collect(),
		}
	}
}
