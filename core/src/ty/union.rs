use crate::{prop, Causes, Model, Ref};
use std::collections::{HashMap, HashSet};

pub struct Union<F> {
	options: HashMap<Ref<super::Definition<F>>, Causes<F>>,
}

impl<F> Union<F> {
	pub fn new(options: HashMap<Ref<super::Definition<F>>, Causes<F>>) -> Self {
		Self { options }
	}

	pub fn properties_with_duplicates<'m>(
		&'m self,
		model: &'m Model<F>,
	) -> PropertiesWithDuplicates<'m, F> {
		PropertiesWithDuplicates {
			model,
			remaning_options: self.options.keys(),
			current: None,
		}
	}

	pub fn properties<'m>(&'m self, model: &'m Model<F>) -> Properties<'m, F> {
		Properties {
			visited: HashSet::new(),
			inner: self.properties_with_duplicates(model),
		}
	}
}

pub struct PropertiesWithDuplicates<'a, F> {
	model: &'a Model<F>,
	remaning_options: std::collections::hash_map::Keys<'a, Ref<super::Definition<F>>, Causes<F>>,
	current: Option<Box<super::PropertiesWithDuplicates<'a, F>>>,
}

impl<'a, F> Iterator for PropertiesWithDuplicates<'a, F> {
	type Item = (Ref<prop::Definition<F>>, &'a Causes<F>);

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.current.as_mut() {
				Some(current) => match current.next() {
					Some(next) => break Some(next),
					None => self.current = None,
				},
				None => match self.remaning_options.next() {
					Some(ty_ref) => {
						let ty = self.model.types().get(*ty_ref).unwrap();
						self.current = Some(Box::new(ty.properties_with_duplicates(self.model)))
					}
					None => break None,
				},
			}
		}
	}
}

pub struct Properties<'a, F> {
	visited: HashSet<Ref<prop::Definition<F>>>,
	inner: PropertiesWithDuplicates<'a, F>,
}

impl<'a, F> Iterator for Properties<'a, F> {
	type Item = (Ref<prop::Definition<F>>, &'a Causes<F>);

	fn next(&mut self) -> Option<Self::Item> {
		for next in self.inner.by_ref() {
			if self.visited.insert(next.0) {
				return Some(next);
			}
		}

		None
	}
}
