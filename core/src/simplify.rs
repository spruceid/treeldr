use std::collections::{btree_map::Entry, BTreeMap, HashMap, VecDeque};

use rdf_types::{Generator, VocabularyMut};
use shelves::{Ref, Shelf};

use crate::{
	utils::UnionFind,
	vocab::{StrippedObject, StrippedQuad},
	BlankIdIndex, Id, IriIndex, Model, Node, ReferenceSubstitution, SubstituteReferences,
};

impl<M> Model<M> {
	/// Simplify the model to merge structurally equivalent anonymous nodes.
	///
	/// # Determinism
	///
	/// This function is deterministic.
	pub fn simplify<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) {
		let mut quads = Vec::new();
		self.to_rdf_with(
			vocabulary,
			generator,
			&mut quads,
			crate::to_rdf::Options {
				ignore_standard_vocabulary: false,
			},
		);

		let equivalences = compute_equivalences(&quads);

		self.apply_equivalences(vocabulary, generator, equivalences)
	}

	fn apply_equivalences<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		equivalences: Equivalences,
	) {
		let ids = equivalences.assign_ids(vocabulary, generator);

		struct ShelfSubstitution<T> {
			items: Vec<Option<T>>,
			new_items: Vec<T>,
			map: HashMap<Ref<T>, Ref<T>>,
		}

		impl<T> ShelfSubstitution<T> {
			fn new(shelf: Shelf<Vec<T>>) -> Self {
				Self {
					items: shelf.into_storage().into_iter().map(Option::Some).collect(),
					new_items: Vec::new(),
					map: HashMap::new(),
				}
			}

			fn insert(&mut self, r: Ref<T>, new_r: Ref<T>) {
				self.map.insert(r, new_r);
			}

			fn allocate(&mut self, r: Ref<T>) -> Ref<T> {
				let new_r = Ref::new(self.new_items.len());
				self.map.insert(r, new_r);
				let item = self.items[r.index()].take().unwrap();
				self.new_items.push(item);
				new_r
			}

			#[allow(clippy::type_complexity)]
			fn into_parts(self) -> (Shelf<Vec<T>>, HashMap<Ref<T>, Ref<T>>) {
				(Shelf::new(self.new_items), self.map)
			}
		}

		let mut types = ShelfSubstitution::new(std::mem::take(self.types_mut()));
		let mut properties = ShelfSubstitution::new(std::mem::take(self.properties_mut()));
		let mut layouts = ShelfSubstitution::new(std::mem::take(self.layouts_mut()));

		for (id, node) in std::mem::take(&mut self.nodes) {
			let new_id = match id {
				Id::Blank(b) => ids.get(&b).cloned().unwrap_or(Id::Blank(b)),
				Id::Iri(i) => Id::Iri(i),
			};

			match self.nodes.entry(new_id) {
				Entry::Vacant(entry) => {
					let mut node = node.into_parts();

					node.ty = node.ty.map(|r| types.allocate(r));
					node.property = node.property.map(|r| properties.allocate(r));
					node.layout = node.layout.map(|r| layouts.allocate(r));

					entry.insert(Node::from_parts(node));
				}
				Entry::Occupied(entry) => {
					if let Some(r) = node.as_type() {
						types.insert(r, entry.get().as_type().unwrap())
					}

					if let Some(r) = node.as_property() {
						properties.insert(r, entry.get().as_property().unwrap())
					}

					if let Some(r) = node.as_layout() {
						layouts.insert(r, entry.get().as_layout().unwrap())
					}
				}
			}
		}

		let (types, types_map) = types.into_parts();
		let (properties, properties_map) = properties.into_parts();
		let (layouts, layouts_map) = layouts.into_parts();

		*self.types_mut() = types;
		*self.properties_mut() = properties;
		*self.layouts_mut() = layouts;

		self.substitute_references(&ReferenceSubstitution::new(
			|id| match id {
				Id::Blank(b) => ids.get(&b).cloned().unwrap_or(Id::Blank(b)),
				Id::Iri(i) => Id::Iri(i),
			},
			|r| types_map[&r],
			|r| properties_map[&r],
			|r| layouts_map[&r],
		));
	}
}

/// Computes the equivalence relation between the nodes of the graph.
fn compute_equivalences(quads: &[StrippedQuad]) -> Equivalences {
	let mut blank_data: BTreeMap<BlankIdIndex, BlankData> = BTreeMap::new();

	// Collect all the occurrences for each blank node, independently of other
	// blank nodes. This will allow us to assign a color for each node, such
	// that two equivalent blank nodes have the same color.
	for quad in quads {
		if let Id::Blank(b) = quad.subject() {
			blank_data
				.entry(*b)
				.or_default()
				.add_occurence(occurrence(quad, *b));
		}
	}

	// Actually compute the colors.
	let mut colors = HashMap::new();
	let mut by_color = Vec::new();
	let blank_simplified_data: HashMap<_, _> = blank_data
		.into_iter()
		.map(|(b, mut data)| {
			data.occurrences.sort();
			let color = *colors.entry(data.occurrences).or_insert_with(|| {
				let len = by_color.len();
				by_color.push(Vec::new());
				len
			});

			by_color[color].push(b);

			(
				b,
				BlankSimplifiedData {
					color,
					quads: data.quads.into_values().collect(),
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

#[derive(Default)]
struct BlankData<'a> {
	occurrences: Vec<QuadColor<'a>>,
	quads: BTreeMap<QuadColor<'a>, ColoredQuads>,
}

impl<'a> BlankData<'a> {
	fn add_occurence(&mut self, (color, q): (QuadColor<'a>, ColoredQuad)) {
		self.occurrences.push(color);

		match q {
			ColoredQuad::Zero => (),
			ColoredQuad::One(a) => self
				.quads
				.entry(color)
				.or_insert(ColoredQuads::One(vec![]))
				.as_one_mut()
				.unwrap()
				.push(a),
			// ColoredQuad::Two(a, b) => self.quads.entry(color).or_insert(ColoredQuads::Two(vec![])).as_two_mut().unwrap().push((a, b)),
		}
	}
}

fn occurrence(quad: &StrippedQuad, x: BlankIdIndex) -> (QuadColor, ColoredQuad) {
	match quad.object() {
		StrippedObject::Blank(z) => {
			if x == *z {
				(QuadColor::This(*quad.predicate()), ColoredQuad::Zero)
			} else {
				(QuadColor::Blank(*quad.predicate()), ColoredQuad::One(*z))
			}
		}
		o => (QuadColor::Value(*quad.predicate(), o), ColoredQuad::Zero),
	}
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuadColor<'a> {
	/// The object equal to the subject.
	This(IriIndex),

	/// The object is another blank node.
	Blank(IriIndex),

	/// The object is not a blank node.
	Value(IriIndex, &'a StrippedObject),
}

enum ColoredQuad {
	Zero,
	One(BlankIdIndex),
	// Two(BlankIdIndex, BlankIdIndex)
}

/// Quad of a given color.
enum ColoredQuads {
	One(Vec<BlankIdIndex>),
	// Two(Vec<(BlankIdIndex, BlankIdIndex)>),
}

impl ColoredQuads {
	fn as_one_mut(&mut self) -> Option<&mut Vec<BlankIdIndex>> {
		match self {
			Self::One(v) => Some(v),
			// Self::Two(_) => None
		}
	}

	// fn as_two_mut(&mut self) -> Option<&mut Vec<(BlankIdIndex, BlankIdIndex)>> {
	// 	match self {
	// 		Self::One(_) => None,
	// 		Self::Two(v) => Some(v)
	// 	}
	// }
}

struct BlankSimplifiedData {
	/// Blank id color.
	color: usize,

	/// Quads in which the blank id occurs, by color.
	quads: Vec<ColoredQuads>,
}

#[derive(Clone)]
pub enum Item<'a> {
	Equivalence(BlankIdIndex, BlankIdIndex),
	ColoredQuadsOneEquiv(&'a [BlankIdIndex], &'a [BlankIdIndex]),
	// ColoredQuadsTwoEquiv(&'a [(BlankIdIndex, BlankIdIndex)], &'a [(BlankIdIndex, BlankIdIndex)]),
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

	fn step(&mut self, blanks: &'a HashMap<BlankIdIndex, BlankSimplifiedData>) {
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

	fn solve(mut self, blanks: &'a HashMap<BlankIdIndex, BlankSimplifiedData>) -> Equivalences {
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

	fn next(mut self, blanks: &'a HashMap<BlankIdIndex, BlankSimplifiedData>) -> Vec<Self> {
		match self.stack.pop() {
			Some(item) => {
				match item {
					Item::Equivalence(a, b) => {
						self.equivalences.merge(a, b);
						self.statements.push((a, b));

						for (a, b) in blanks[&a].quads.iter().zip(&blanks[&b].quads) {
							match (a, b) {
								(ColoredQuads::One(a), ColoredQuads::One(b)) => {
									self.stack.push(Item::ColoredQuadsOneEquiv(a, b))
								} // (ColoredQuads::Two(a), ColoredQuads::Two(b)) => {
								  // 	self.stack.push(Item::ColoredQuadsTwoEquiv(a, b))
								  // }
								  // _ => panic!("blank ids do not have the same color")
							}
						}

						vec![self]
					}
					Item::ColoredQuadsOneEquiv(a, b) => {
						let mut result = Vec::new();
						let mut indexes: Vec<_> = (0usize..b.len()).collect();

						fn child_environment<'a>(
							blanks: &'a HashMap<BlankIdIndex, BlankSimplifiedData>,
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

							if !crate::utils::permutation::next(&mut indexes) {
								break result;
							}
						}
					}
				}
			}
			None => vec![self],
		}
	}
}
