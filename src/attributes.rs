use educe::Educe;
use serde_seeded::{DeserializeSeeded, SerializeSeeded};
use std::collections::BTreeMap;
use std::hash::Hash;

use crate::{
	rdf::{
		serde::serde_indexed_resource, ResourceBackwardIndex, ResourceForwardIndex, VisitResources,
	},
	Value,
};

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(R: Clone + Ord))
)]
#[educe(Default)]
pub struct Attributes<R> {
	map: BTreeMap<Attribute<R>, Vec<Value<R>>>,
}

impl<R> Attributes<R> {
	pub fn new() -> Self {
		Self {
			map: BTreeMap::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}
}

impl<R: Ord> Attributes<R> {
	pub fn insert(&mut self, attr: Attribute<R>, value: Value<R>) {
		self.map.entry(attr).or_default().push(value)
	}

	pub fn extend_attribute(
		&mut self,
		attr: Attribute<R>,
		values: impl IntoIterator<Item = Value<R>>,
	) {
		self.map.entry(attr).or_default().extend(values)
	}

	pub fn label(&self) -> Option<&str> {
		let label = self
			.map
			.get(&Attribute::Documentation)?
			.first()?
			.as_str()?
			.lines()
			.next()?
			.trim();
		Some(label.strip_suffix(".").unwrap_or(label))
	}

	pub fn description(&self) -> String {
		let mut result = String::new();

		if let Some(attrs) = self.map.get(&Attribute::Documentation) {
			for (i, doc) in attrs.iter().enumerate() {
				if let Some(s) = doc.as_str() {
					for (j, line) in s.lines().enumerate() {
						if i > 0 || j > 0 {
							let line = line.trim();
							if !result.is_empty() || !line.is_empty() {
								if !result.is_empty() {
									result.push('\n');
								}

								result.push_str(line);
							}
						}
					}
				}
			}
		}

		result
	}

	pub fn function_arguments_documentation(&self) -> Vec<ArgumentDocumentation> {
		let mut result = Vec::new();

		if let Some(attr) = self.map.get(&Attribute::Documentation) {
			enum State {
				Initial,
				List,
				Argument(ArgumentDocumentation),
			}

			let mut state = State::Initial;

			impl State {
				fn next(self, result: &mut Vec<ArgumentDocumentation>, line: &str) -> Self {
					match self {
						Self::Initial => {
							if line == "# Arguments" {
								Self::List
							} else {
								Self::Initial
							}
						}
						Self::List => {
							if let Some(rest) = line.strip_prefix('-') {
								if let Some(a) = ArgumentDocumentation::from_li(rest.trim()) {
									return State::Argument(a);
								}
							}

							Self::List
						}
						Self::Argument(mut arg) => match line.strip_prefix('-') {
							Some(rest) => {
								result.push(arg);
								match ArgumentDocumentation::from_li(rest.trim()) {
									Some(a) => State::Argument(a),
									None => Self::List,
								}
							}
							None => {
								if !arg.description.is_empty() {
									arg.description.push('\n');
								}

								arg.description.push_str(line);
								State::Argument(arg)
							}
						},
					}
				}
			}

			for doc in attr.iter() {
				if let Some(s) = doc.as_str() {
					for line in s.lines() {
						state = state.next(&mut result, line.trim());
					}
				}
			}

			if let State::Argument(arg) = state {
				result.push(arg);
			}
		}

		result
	}
}

#[derive(Clone)]
pub struct ArgumentDocumentation {
	pub id: String,
	pub label: String,
	pub description: String,
}

impl ArgumentDocumentation {
	fn from_li(s: &str) -> Option<Self> {
		let (id, label) = s.split_once(':')?;
		Some(ArgumentDocumentation {
			id: id.trim().trim_matches('`').to_string(),
			label: label.trim().to_string(),
			description: String::new(),
		})
	}
}

impl<R> VisitResources<R> for Attributes<R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.map.visit_resources_ref_mut(f)
	}
}

pub type IntoIter<R> = std::collections::btree_map::IntoIter<Attribute<R>, Vec<Value<R>>>;

impl<R> IntoIterator for Attributes<R> {
	type IntoIter = IntoIter<R>;
	type Item = (Attribute<R>, Vec<Value<R>>);

	fn into_iter(self) -> Self::IntoIter {
		self.map.into_iter()
	}
}

impl<R, V> Extend<(Attribute<R>, V)> for Attributes<R>
where
	R: Ord,
	V: IntoIterator<Item = Value<R>>,
{
	fn extend<T: IntoIterator<Item = (Attribute<R>, V)>>(&mut self, iter: T) {
		for (attr, values) in iter {
			self.extend_attribute(attr, values);
		}
	}
}

impl<R, V> FromIterator<(Attribute<R>, V)> for Attributes<R>
where
	R: Ord,
	V: IntoIterator<Item = Value<R>>,
{
	fn from_iter<T: IntoIterator<Item = (Attribute<R>, V)>>(iter: T) -> Self {
		let mut result = Self::new();
		result.extend(iter);
		result
	}
}

impl<R> Extend<(Attribute<R>, Value<R>)> for Attributes<R>
where
	R: Ord,
{
	fn extend<T: IntoIterator<Item = (Attribute<R>, Value<R>)>>(&mut self, iter: T) {
		for (attr, value) in iter {
			self.insert(attr, value);
		}
	}
}

impl<R> FromIterator<(Attribute<R>, Value<R>)> for Attributes<R>
where
	R: Ord,
{
	fn from_iter<T: IntoIterator<Item = (Attribute<R>, Value<R>)>>(iter: T) -> Self {
		let mut result = Self::new();
		result.extend(iter);
		result
	}
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(R: Clone + Ord))
)]
pub enum Attribute<R> {
	Documentation,
	Custom(#[seeded(with(serde_indexed_resource))] R),
}

impl<R> VisitResources<R> for Attribute<R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::Documentation => (),
			Self::Custom(r) => f(r),
		}
	}
}
