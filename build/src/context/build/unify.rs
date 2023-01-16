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
	/// Replace any blank node with `()`.
	fn strip_blank(self) -> BindingRef<'a, ()> {
		BindingRef {
			property: self.property,
			value: self.value.strip_blank(),
		}
	}

	/// Try to replace any blank node in the binding by its color, given by the
	/// provided `color` function.
	fn color(self, color: impl FnOnce(B) -> Option<usize>) -> Option<BindingRef<'a, usize>> {
		Some(BindingRef {
			property: self.property,
			value: self.value.color(color)?,
		})
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

	fn color(self, color: impl FnOnce(B) -> Option<usize>) -> Option<ValueRef<'a, usize>> {
		Some(match self {
			Self::Blank(id) => ValueRef::Blank(color(id)?),
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
		})
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

/// Blank node color.
struct Color {
	/// Is the color exact?
	///
	/// A color is exact if all the members of the color are equivalent and can
	/// be merged without further computation.
	exact: bool,

	/// Members of the color.
	members: Vec<BlankIdIndex>,
}

impl Color {
	/// Create a new color for the given property.
	///
	/// Each property has its own color.
	fn property(member: BlankIdIndex) -> Color {
		Color {
			exact: true,
			members: vec![member],
		}
	}

	/// Create a new exact color for non recursive blank nodes.
	fn exact() -> Self {
		Color {
			exact: true,
			members: Vec::new(),
		}
	}

	/// Create a new (non exact) color for recursive blank nodes.
	fn recursive() -> Self {
		Color {
			exact: false,
			members: Vec::new(),
		}
	}
}

impl<M> Context<M> {
	/// Merge equivalent blank nodes.
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

	/// Compute the color of each blank nodes.
	fn compute_blank_colors(&self) -> Vec<Color> {
		type RecursiveColor<'a> = BTreeSet<BindingRef<'a, ()>>;

		type TreeColor<'a> = BTreeSet<BindingRef<'a, usize>>;

		/// Compute the color of the given blank node.
		fn color<'a, M>(
			context: &'a Context<M>,
			id: BlankIdIndex,
			colors: &mut Vec<Color>,
			result: &mut HashMap<BlankIdIndex, Option<usize>>,
			tree_colors: &mut HashMap<TreeColor<'a>, usize>,
			recursive_colors: &mut HashMap<RecursiveColor<'a>, usize>,
		) -> Option<usize> {
			use std::collections::hash_map::Entry;
			match result.entry(id) {
				Entry::Occupied(entry) => *entry.get(),
				Entry::Vacant(entry) => {
					let node = context.get(Id::Blank(id)).unwrap();
					if node.has_type(context, resource::Type::Property(None)) {
						let c = colors.len();
						colors.push(Color::property(id));
						entry.insert(Some(c));
						Some(c)
					} else {
						entry.insert(None);

						let mut tree_color = Some(BTreeSet::new());
						let mut recursive_color = BTreeSet::new();

						for Meta(binding, _) in node.bindings() {
							let binding: BindingRef<'a, BlankIdIndex> = binding.into();

							recursive_color.insert(binding.strip_blank());

							if let Some(tc) = &mut tree_color {
								match binding.color(|b| {
									color(context, b, colors, result, tree_colors, recursive_colors)
										.filter(|c| colors[*c].exact)
								}) {
									Some(colored_binding) => {
										tc.insert(colored_binding);
									}
									None => tree_color = None,
								}
							}
						}

						let c = match tree_color {
							Some(tc) => *tree_colors.entry(tc).or_insert_with(|| {
								let c = colors.len();
								colors.push(Color::exact());
								c
							}),
							None => *recursive_colors.entry(recursive_color).or_insert_with(|| {
								let c = colors.len();
								colors.push(Color::recursive());
								c
							}),
						};

						colors[c].members.push(id);
						result.insert(id, Some(c));
						Some(c)
					}
				}
			}
		}

		let mut colors = Vec::new();
		let mut result = HashMap::new();
		let mut tree_colors = HashMap::new();
		let mut recursive_colors = HashMap::new();

		for id in self.ids() {
			if let Id::Blank(id) = id {
				color(
					self,
					id,
					&mut colors,
					&mut result,
					&mut tree_colors,
					&mut recursive_colors,
				);
			}
		}

		colors
	}

	/// Computes the blank node equivalences.
	fn compute_equivalences(&self) -> Equivalences {
		let colors = self.compute_blank_colors();

		// Initialize the solver that will check for equivalences.
		let mut solver = Solver::default();
		let mut blank_simplified_data: HashMap<BlankIdIndex, SimplifiedBlankData> = HashMap::new();
		for (c, color) in colors.iter().enumerate() {
			for &id in &color.members {
				let mut data = SimplifiedBlankData {
					color: c,
					bindings: Vec::new(),
				};

				if !color.exact {
					let node = self.get(Id::Blank(id)).unwrap();
					let mut bindings: BTreeMap<Property, BTreeSet<BlankIdIndex>> = BTreeMap::new();

					for Meta(binding, _) in node.bindings() {
						let binding: BindingRef<BlankIdIndex> = binding.into();
						if let ValueRef::Blank(b) = binding.value {
							bindings.entry(binding.property).or_default().insert(b);
						}
					}

					data.bindings = bindings
						.into_values()
						.map(|v| v.into_iter().collect())
						.collect();
				}

				blank_simplified_data.insert(id, data);
				solver.equivalences.insert(id);
			}
		}

		for color in colors {
			if color.exact {
				if let Some((&a, rest)) = color.members.split_first() {
					for &b in rest {
						solver.equivalences.merge(a, b);
					}
				}
			} else {
				for (i, &a) in color.members.iter().enumerate() {
					for &b in &color.members[i + 1..] {
						solver.check(a, b)
					}
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
		// Make each stack inherit the initial knowledge.
		for s in &mut self.stacks {
			// FIXME: this is not efficient at all!
			// TODO: write a hierarchical union-find data structure to avoid
			//       cloning every time we want to inherit knowledge.
			s.equivalences = s.equivalences.clone();
		}

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
