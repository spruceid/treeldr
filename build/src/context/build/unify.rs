use std::{
	collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
	hash::Hash,
};

use langtag::LanguageTag;
use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{
	metadata::Merge,
	ty::data::RegExp,
	utils::UnionFind,
	value,
	vocab::{Literal, Object},
	BlankIdIndex, Id, IriIndex, Name, Property,
};

use crate::{
	context::MapIds,
	resource::{self, BindingValueRef},
	Context,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingRef<'a, B> {
	property: Property,
	value: ValueRef<'a, B>,
}

impl<'a, B> BindingRef<'a, B> {
	fn strip_blank(self) -> BindingRef<'a, ()> {
		BindingRef {
			property: self.property,
			value: self.value.strip_blank(),
		}
	}
}

impl<'a, M> From<resource::BindingRef<'a, M>> for BindingRef<'a, BlankIdIndex> {
	fn from(b: resource::BindingRef<'a, M>) -> Self {
		Self {
			property: b.property(),
			value: b.value().into(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueRef<'a, B> {
	Blank(B),
	Iri(IriIndex),
	Boolean(bool),
	U64(u64),
	Integer(&'a value::Integer),
	Numeric(&'a value::Numeric),
	String(&'a str),
	LangString(&'a str, LanguageTag<'a>),
	TypedString(&'a str, IriIndex),
	Name(&'a Name),
	RegExp(&'a RegExp),
}

impl<'a, B> ValueRef<'a, B> {
	fn strip_blank(self) -> ValueRef<'a, ()> {
		match self {
			Self::Blank(_) => ValueRef::Blank(()),
			Self::Iri(i) => ValueRef::Iri(i),
			Self::Boolean(b) => ValueRef::Boolean(b),
			Self::U64(u) => ValueRef::U64(u),
			Self::Integer(i) => ValueRef::Integer(i),
			Self::Numeric(n) => ValueRef::Numeric(n),
			Self::String(s) => ValueRef::String(s),
			Self::LangString(s, t) => ValueRef::LangString(s, t),
			Self::TypedString(s, t) => ValueRef::TypedString(s, t),
			Self::Name(n) => ValueRef::Name(n),
			Self::RegExp(e) => ValueRef::RegExp(e),
		}
	}
}

impl<'a, M> From<BindingValueRef<'a, M>> for ValueRef<'a, BlankIdIndex> {
	fn from(v: BindingValueRef<'a, M>) -> Self {
		match v {
			BindingValueRef::Id(Id::Blank(b)) => Self::Blank(b),
			BindingValueRef::Id(Id::Iri(iri)) => Self::Iri(iri),
			BindingValueRef::Type(t) => match t.id() {
				Id::Blank(b) => Self::Blank(b),
				Id::Iri(i) => Self::Iri(i),
			},
			BindingValueRef::Boolean(b) => Self::Boolean(b),
			BindingValueRef::U64(u) => Self::U64(u),
			BindingValueRef::Integer(i) => Self::Integer(i),
			BindingValueRef::Numeric(n) => Self::Numeric(n),
			BindingValueRef::String(s) => Self::String(s),
			BindingValueRef::Name(n) => Self::Name(n),
			BindingValueRef::RegExp(e) => Self::RegExp(e),
			BindingValueRef::Object(o) => match o {
				Object::Blank(b) => Self::Blank(*b),
				Object::Iri(i) => Self::Iri(*i),
				Object::Literal(l) => match l {
					Literal::String(s) => Self::String(s),
					Literal::LangString(Meta(s, _), Meta(tag, _)) => {
						Self::LangString(s, tag.as_ref())
					}
					Literal::TypedString(Meta(s, _), Meta(ty, _)) => Self::TypedString(s, *ty),
				},
			},
		}
	}
}

impl<M> Context<M> {
	pub fn unify<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) where
		M: Merge,
	{
		let equivalences = self.compute_equivalences();
		let ids = equivalences.assign_ids(vocabulary, generator);
		self.map_ids(|id, _| match id {
			Id::Blank(b) => *ids.get(&b).unwrap(),
			Id::Iri(i) => Id::Iri(i),
		})
	}

	fn compute_equivalences(&self) -> Equivalences {
		#[derive(Default)]
		struct BlankData<'a> {
			is_property: bool,
			color: BTreeSet<BindingRef<'a, ()>>,
			bindings: BTreeMap<Property, BTreeSet<BlankIdIndex>>,
		}

		impl<'a> BlankData<'a> {
			fn insert(&mut self, binding: BindingRef<'a, BlankIdIndex>) {
				self.color.insert(binding.strip_blank());
				if let ValueRef::Blank(b) = binding.value {
					self.bindings.entry(binding.property).or_default().insert(b);
				}
			}
		}

		// Collect all the bindings for each blank node, independently of other
		// blank nodes. This will allow us to assign a color for each node, such
		// that two equivalent blank nodes have the same color.
		let mut blank_data: BTreeMap<BlankIdIndex, BlankData> = BTreeMap::new();
		for (id, node) in &self.nodes {
			if let Id::Blank(id) = id {
				if node.has_type(self, resource::Type::Property(None)) {
					blank_data.entry(*id).or_default().is_property = true;
				} else {
					for Meta(binding, _) in node.bindings() {
						blank_data.entry(*id).or_default().insert(binding.into())
					}
				}
			}
		}

		// Actually compute the colors.
		let mut colors = HashMap::new();
		let mut by_color = Vec::new();
		let blank_simplified_data: HashMap<_, _> = blank_data
			.into_iter()
			.map(|(b, data)| {
				let color = if data.is_property {
					let len = by_color.len();
					by_color.push(Vec::new());
					len
				} else {
					*colors.entry(data.color).or_insert_with(|| {
						let len = by_color.len();
						by_color.push(Vec::new());
						len
					})
				};

				by_color[color].push(b);

				(
					b,
					SimplifiedBlankData {
						color,
						bindings: data
							.bindings
							.into_iter()
							.map(|(_, v)| v.into_iter().collect())
							.collect(),
					},
				)
			})
			.collect();

		// Initialize the solver that will check for equivalences.
		let mut solver = Solver::default();
		for blank_ids in &by_color {
			for &a in blank_ids {
				solver.equivalences.insert(a);
			}
		}
		for blank_ids in by_color {
			for (i, &a) in blank_ids.iter().enumerate() {
				for &b in &blank_ids[i + 1..] {
					solver.check(a, b)
				}
			}
		}

		// Run the solver.
		solver.solve(&blank_simplified_data)
	}
}

struct SimplifiedBlankData {
	color: usize,
	bindings: Vec<Vec<BlankIdIndex>>,
}

#[derive(Clone, Default)]
struct Equivalences {
	uf: UnionFind<BlankIdIndex, ()>,
}

impl Equivalences {
	fn insert(&mut self, i: BlankIdIndex) {
		self.uf.insert(i, ())
	}

	fn eq(&self, a: BlankIdIndex, b: BlankIdIndex) -> bool {
		match self.uf.class_of(&a) {
			Some(a) => match self.uf.class_of(&b) {
				Some(b) => a == b,
				None => false,
			},
			None => false,
		}
	}

	fn merge(&mut self, a: BlankIdIndex, b: BlankIdIndex) {
		self.uf.merge(&a, &b, |(), ()| ())
	}

	fn assign_ids<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> UnionFind<BlankIdIndex, Id> {
		self.uf.map(|_| generator.next(vocabulary))
	}
}

#[derive(Clone)]
pub enum Item<'a> {
	Equivalence(BlankIdIndex, BlankIdIndex),
	Bijection(&'a [BlankIdIndex], &'a [BlankIdIndex]),
}

#[derive(Default)]
struct Solver<'a> {
	stacks: VecDeque<Stack<'a>>,
	equivalences: Equivalences,
}

impl<'a> Solver<'a> {
	fn is_done(&self) -> bool {
		self.stacks.is_empty()
	}

	fn step(&mut self, blanks: &'a HashMap<BlankIdIndex, SimplifiedBlankData>) {
		if let Some(stack) = self.stacks.pop_front() {
			if stack.is_proven() {
				for (a, b) in stack.statements {
					self.equivalences.merge(a, b);
					for s in &mut self.stacks {
						s.equivalences.merge(a, b)
					}
				}
			} else {
				for s in stack.next(blanks) {
					self.stacks.push_back(s)
				}
			}
		}
	}

	fn check(&mut self, a: BlankIdIndex, b: BlankIdIndex) {
		self.stacks
			.push_front(Stack::new(a, b, self.equivalences.clone()))
	}

	fn solve(mut self, blanks: &'a HashMap<BlankIdIndex, SimplifiedBlankData>) -> Equivalences {
		while !self.is_done() {
			self.step(blanks)
		}

		self.equivalences
	}
}

struct Stack<'a> {
	stack: Vec<Item<'a>>,
	equivalences: Equivalences,
	statements: Vec<(BlankIdIndex, BlankIdIndex)>,
}

impl<'a> Stack<'a> {
	fn new(a: BlankIdIndex, b: BlankIdIndex, equivalences: Equivalences) -> Self {
		Self {
			stack: vec![Item::Equivalence(a, b)],
			equivalences,
			statements: Vec::new(),
		}
	}

	fn is_proven(&self) -> bool {
		self.stack.is_empty()
	}

	fn next(mut self, blanks: &'a HashMap<BlankIdIndex, SimplifiedBlankData>) -> Vec<Self> {
		match self.stack.pop() {
			Some(item) => match item {
				Item::Equivalence(a, b) => {
					self.equivalences.merge(a, b);
					self.statements.push((a, b));

					for (a, b) in blanks[&a].bindings.iter().zip(&blanks[&b].bindings) {
						self.stack.push(Item::Bijection(a, b))
					}

					vec![self]
				}
				Item::Bijection(a, b) => {
					let mut result = Vec::new();
					let mut indexes = Vec::new();
					indexes.resize(a.len(), 0);

					fn child_environment<'a>(
						blanks: &'a HashMap<BlankIdIndex, SimplifiedBlankData>,
						mut stack: Vec<Item<'a>>,
						indexes: &[usize],
						equivalences: &Equivalences,
						statements: &[(BlankIdIndex, BlankIdIndex)],
						a: &[BlankIdIndex],
						b: &[BlankIdIndex],
					) -> Option<Stack<'a>> {
						for (i, a) in a.iter().cloned().enumerate() {
							let b = b[indexes[i]];

							if blanks[&a].color != blanks[&b].color {
								return None;
							}

							if !equivalences.eq(a, b) {
								stack.push(Item::Equivalence(a, b))
							}
						}

						Some(Stack {
							stack,
							equivalences: equivalences.clone(),
							statements: statements.to_vec(),
						})
					}

					loop {
						if let Some(env) = child_environment(
							blanks,
							self.stack.clone(),
							&indexes,
							&self.equivalences,
							&self.statements,
							a,
							b,
						) {
							result.push(env)
						}

						if !incr(&mut indexes, b.len()) {
							break result;
						}
					}
				}
			},
			None => vec![self],
		}
	}
}

fn incr(digits: &mut [usize], base: usize) -> bool {
	for d in digits.iter_mut().rev() {
		*d += 1;

		if *d == base {
			*d = 0
		} else {
			return true;
		}
	}

	false
}
