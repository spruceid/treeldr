use hashbrown::HashMap;
use std::{borrow::Cow, hash::Hash};

use disjoint_collections::DisjointVec;
use iref::IriBuf;
use rdf_types::{
	dataset::{DatasetMut, IndexedBTreeDataset},
	Literal,
};

use crate::{
	rdf::{
		FallibleResourceUnifier, ResourceUnificationFailed, SubstituteResource,
		TryUnifyInterpretation,
	},
	Value,
};

use super::Class;

pub struct DeducedInterpretation<R> {
	pub parent_mapping: HashMap<R, usize>,
	pub iris: HashMap<IriBuf, usize>,
	pub literals: HashMap<Literal, usize>,
	pub classes: DisjointVec<Class<R>>,
}

impl<R> Default for DeducedInterpretation<R> {
	fn default() -> Self {
		Self {
			parent_mapping: HashMap::new(),
			iris: HashMap::new(),
			literals: HashMap::new(),
			classes: DisjointVec::new(),
		}
	}
}

impl<R> DeducedInterpretation<R> {
	pub fn insert_resource(&mut self, class: Class<R>) -> usize
	where
		R: Clone + Eq + Hash,
	{
		let r = class.canonical_resource.clone();
		let i = self.classes.push(class);
		let class = self.classes.get(i).unwrap();

		for iri in &class.local_iris {
			self.iris.insert(iri.clone(), i);
		}

		for literal in &class.local_literals {
			self.literals.insert(literal.clone(), i);
		}

		self.parent_mapping.insert(r, i);
		i
	}

	pub fn unify(&mut self, a: usize, b: usize)
	where
		R: Ord,
	{
		self.classes.merge(a, b, Class::unify);
	}
}

pub struct ContextDelta<R> {
	pub(crate) interpretation: DeducedInterpretation<R>,
	pub(crate) dataset: IndexedBTreeDataset<R>,
}

impl<R> ContextDelta<R> {
	pub fn new() -> Self {
		Self {
			interpretation: DeducedInterpretation::default(),
			dataset: IndexedBTreeDataset::new(),
		}
	}

	pub fn apply(
		&self,
		interpretation: &mut impl TryUnifyInterpretation<Resource = R>,
		dataset: &mut (impl DatasetMut<Resource = R> + SubstituteResource<R>),
		mut values: &mut [Value<R>],
	) -> Result<(), ResourceUnificationFailed>
	where
		R: Clone + Ord,
	{
		for quad in &self.dataset {
			dataset.insert(quad.cloned());
		}

		let mut unifier = interpretation.begin_unification();

		for (_, class) in self.interpretation.classes.classes() {
			let mut resources = class.resources().into_iter();
			if let Some(a) = resources.next() {
				for b in resources {
					unifier
						.try_unify(a, b)
						.map_err(|_| ResourceUnificationFailed)?;
				}

				for iri in class.iris() {
					unifier
						.insert_iri(a, Cow::Borrowed(iri.as_iri()))
						.map_err(|_| ResourceUnificationFailed)?;
				}

				for literal in class.literals() {
					unifier
						.insert_literal(a, literal)
						.map_err(|_| ResourceUnificationFailed)?;
				}
			}
		}

		dataset.substitute_resource(|r| unifier.substitute_resource(r));
		values.substitute_resource(|r| unifier.substitute_resource(r));
		Ok(())
	}
}
