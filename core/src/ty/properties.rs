use crate::{
	Ref,
	Causes,
	prop,
	prop::restriction::Contradiction
};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use locspan::Location;

#[derive(Clone)]
struct PropertyData<F> {
	causes: Causes<F>,
	restrictions: prop::Restrictions<F>
}

impl<F> PropertyData<F> {
	pub fn add_causes(&mut self, causes: impl Into<Causes<F>>) where F: Ord {
		self.causes.extend(causes.into())
	}

	pub fn add_cause(&mut self, cause: Option<Location<F>>) where F: Ord {
		if let Some(loc) = cause {
			self.causes.add(loc)
		}
	}

	pub fn restrict(&mut self, restriction: prop::Restriction<F>) -> Result<(), Contradiction> {
		self.restrictions.restrict(restriction)
	}
}

/// Type properties.
#[derive(Clone)]
pub struct Properties<F> {
	/// Included properties.
	included: HashMap<Ref<prop::Definition<F>>, PropertyData<F>>,

	/// Excluded properties.
	/// 
	/// If `None`, then all the properties not
	/// in `included` are excluded.
	excluded: Option<HashMap<Ref<prop::Definition<F>>, Causes<F>>>
}

impl<F> Properties<F> {
	pub fn none() -> Self {
		Self {
			included: HashMap::new(),
			excluded: None
		}
	}

	pub fn all() -> Self {
		Self {
			included: HashMap::new(),
			excluded: Some(HashMap::new())
		}
	}

	pub fn included(&self) -> IncludedProperties<F> {
		IncludedProperties {
			inner: self.included.iter()
		}
	}
}

impl<F> Default for Properties<F> {
	fn default() -> Self {
		Self::none()
	}
}

impl<F: Ord> Properties<F> {
	pub fn insert(&mut self, prop: Ref<prop::Definition<F>>, restrictions: Option<prop::Restrictions<F>>, causes: impl Into<Causes<F>>) {
		match self.included.entry(prop) {
			Entry::Occupied(mut entry) => {
				let data = entry.get_mut();
				data.add_causes(causes);
				match restrictions {
					Some(restrictions) => {
						data.restrictions = data.restrictions.union_with(&restrictions)
					}
					None => {
						data.restrictions.clear()
					}
				}
			}
			Entry::Vacant(entry) => {
				entry.insert(PropertyData {
					restrictions: restrictions.unwrap_or_default(),
					causes: causes.into()
				});
			}
		}

		if let Some(excluded) = &mut self.excluded {
			excluded.remove(&prop);
		}
	}

	pub fn remove(&mut self, prop: Ref<prop::Definition<F>>, cause: Option<Location<F>>) {
		self.included.remove(&prop);

		if let Some(excluded) = &mut self.excluded {
			excluded.insert(prop, cause.into());
		}
	}

	/// Further restrict `prop` if it is included in this set of properties.
	pub fn restrict(&mut self, prop: Ref<prop::Definition<F>>, restriction: prop::Restriction<F>, cause: Option<Location<F>>) -> Result<(), Contradiction> {
		if let Some(data) = self.included.get_mut(&prop) {
			data.restrict(restriction)?;
			data.add_cause(cause);
		}

		Ok(())
	}

	pub fn unite_with(&mut self, other: &Self) where F: Clone + Ord {
		for (&prop, data) in &other.included {
			self.insert(prop, Some(data.restrictions.clone()), data.causes.clone());
		}

		if let (Some(excluded), Some(other_excluded)) = (&mut self.excluded, &other.excluded) {
			for (prop, _) in other_excluded {
				excluded.remove(prop);
			}
		}
	}

	pub fn union_with(&self, other: &Self) -> Self where F: Clone + Ord {
		let mut result = self.clone();
		result.unite_with(other);
		result
	}

	pub fn intersect_with(&mut self, other: &Self) -> Result<(), Contradiction> where F: Clone + Ord {
		self.excluded = match (self.excluded.take(), &other.excluded) {
			(Some(mut excluded), Some(other_excluded)) => {
				for (prop, cause) in other_excluded {
					match excluded.entry(*prop) {
						Entry::Occupied(mut entry) => {
							entry.get_mut().extend(cause.clone())
						}
						Entry::Vacant(entry) => {
							entry.insert(cause.clone());
						}
					}
				}
				Some(excluded)
			}
			_ => None
		};

		let included = std::mem::replace(&mut self.included, HashMap::new());
		for (prop, data) in included {
			match other.included.get(&prop) {
				Some(other_data) => {
					let data = PropertyData {
						causes: data.causes.clone().with(other_data.causes.iter().cloned()),
						restrictions: data.restrictions.intersection_with(&other_data.restrictions)?
					};
	
					self.included.insert(prop, data);
				}
				None => {
					if let Some(excluded) = &mut self.excluded {
						excluded.insert(prop, data.causes.clone());
					}
				}
			}
		}

		if let Some(excluded) = &mut self.excluded {
			for (prop, data) in &other.included {
				if !self.included.contains_key(prop) {
					excluded.insert(*prop, data.causes.clone());
				}
			}
		}

		Ok(())
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction> where F: Clone + Ord {
		let mut result = self.clone();
		result.intersect_with(other)?;
		Ok(result)
	}

	pub fn iter(&self) -> Iter<F> {
		Iter {
			included: IncludedProperties {
				inner: self.included.iter()
			},
			excluded: self.excluded.as_ref().map(|excluded| {
				ExcludedProperties {
					inner: excluded.iter()
				}
			})
		}
	}
}

pub struct ExcludedProperties<'a, F> {
	inner: std::collections::hash_map::Iter<'a, Ref<prop::Definition<F>>, Causes<F>>
}

impl<'a, F> Iterator for ExcludedProperties<'a, F> {
	type Item = (Ref<prop::Definition<F>>, &'a Causes<F>);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|(prop, causes)| (*prop, causes))
	}
}

pub struct RestrictedProperty<'a, F> {
	prop: Ref<prop::Definition<F>>,
	restrictions: &'a prop::Restrictions<F>,
	causes: &'a Causes<F>
}

impl<'a, F> RestrictedProperty<'a, F> {
	pub fn property(&self) -> Ref<prop::Definition<F>> {
		self.prop
	}

	pub fn restrictions(&self) -> &'a prop::Restrictions<F> {
		self.restrictions
	}

	pub fn causes(&self) -> &'a Causes<F> {
		self.causes
	}
}

pub enum PseudoProperty<'a, F> {
	Property(RestrictedProperty<'a, F>),
	Complement(ExcludedProperties<'a, F>)
}

pub struct Iter<'a, F> {
	included: IncludedProperties<'a, F>,
	excluded: Option<ExcludedProperties<'a, F>>
}

impl<'a, F> Iterator for Iter<'a, F> {
	type Item = PseudoProperty<'a, F>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.included.next() {
			Some(item) => Some(PseudoProperty::Property(item)),
			None => self.excluded.take().map(PseudoProperty::Complement)
		}
	}
}

pub struct IncludedProperties<'a, F> {
	inner: std::collections::hash_map::Iter<'a, Ref<prop::Definition<F>>, PropertyData<F>>
}

impl<'a, F> Iterator for IncludedProperties<'a, F> {
	type Item = RestrictedProperty<'a, F>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|(prop, data)| RestrictedProperty {
			prop: *prop,
			restrictions: &data.restrictions,
			causes: &data.causes
		})
	}
}