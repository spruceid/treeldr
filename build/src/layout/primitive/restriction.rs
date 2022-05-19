use crate::{error, Error};
use std::collections::BTreeMap;
pub use treeldr::{
	layout::{primitive::restricted, primitive::RegExp, Primitive},
	value, Causes, Id, MaybeSet,
};

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Numeric(Numeric),
	String(String),
}

/// Numeric restriction.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Numeric {
	InclusiveMinimum(value::Numeric),
	ExclusiveMinimum(value::Numeric),
	InclusiveMaximum(value::Numeric),
	ExclusiveMaximum(value::Numeric),
}

/// String restriction.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum String {
	Pattern(RegExp),
}

#[derive(Clone, Debug)]
pub struct Restrictions<F> {
	map: BTreeMap<Restriction, Causes<F>>,
}

impl<F> Default for Restrictions<F> {
	fn default() -> Self {
		Self {
			map: BTreeMap::default(),
		}
	}
}

impl<F> PartialEq for Restrictions<F> {
	fn eq(&self, other: &Self) -> bool {
		self.map.len() == other.map.len()
			&& self.map.keys().zip(other.map.keys()).all(|(a, b)| a == b)
	}
}

impl<F> Eq for Restrictions<F> {}

impl<F: Ord> Restrictions<F> {
	pub fn insert(&mut self, restriction: Restriction, causes: impl Into<Causes<F>>) {
		self.map
			.entry(restriction)
			.or_insert_with(Causes::new)
			.extend(causes.into())
	}

	pub fn unify(&mut self, other: Self) {
		for (r, causes) in other.map {
			self.insert(r, causes)
		}
	}
}

impl<F: Clone> Restrictions<F> {
	pub fn build_boolean(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Boolean,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_integer(self, id: Id) -> Result<restricted::integer::Restrictions, Error<F>> {
		let mut r = restricted::integer::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_integer() {
						Ok(min) => match min.into_i64() {
							Ok(min) => r.add_min(min),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_integer() {
						Ok(min) => match min.into_i64() {
							Ok(min) => r.add_min(min + 1),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_integer() {
						Ok(max) => match max.into_i64() {
							Ok(max) => r.add_max(max),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_integer() {
						Ok(max) => match max.into_i64() {
							Ok(max) => r.add_max(max - 1),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes.preferred().cloned(),
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_unsigned_integer(
		self,
		id: Id,
	) -> Result<restricted::unsigned::Restrictions, Error<F>> {
		let mut r = restricted::unsigned::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_non_negative_integer() {
						Ok(min) => match min.into_u64() {
							Ok(min) => r.add_min(min),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_non_negative_integer() {
						Ok(min) => match min.into_u64() {
							Ok(min) => r.add_min(min + 1),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_non_negative_integer() {
						Ok(max) => match max.into_u64() {
							Ok(max) => r.add_max(max),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_non_negative_integer() {
						Ok(max) => match max.into_u64() {
							Ok(max) => r.add_max(max - 1),
							Err(_) => todo!(),
						},
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes.preferred().cloned(),
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_float(self, id: Id) -> Result<restricted::float::Restrictions, Error<F>> {
		use restricted::float::{Max, Min};
		let mut r = restricted::float::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_float() {
						Ok(min) => r.add_min(Min::Included(min)),
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_float() {
						Ok(min) => r.add_min(Min::Excluded(min)),
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_float() {
						Ok(max) => r.add_max(Max::Included(max)),
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_float() {
						Ok(max) => r.add_max(Max::Excluded(max)),
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes.preferred().cloned(),
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_double(self, id: Id) -> Result<restricted::double::Restrictions, Error<F>> {
		use restricted::double::{Max, Min};
		let mut r = restricted::double::Restrictions::default();

		for (restriction, causes) in self.map {
			match restriction {
				Restriction::Numeric(restriction) => match restriction {
					Numeric::InclusiveMinimum(min) => match min.into_double() {
						Ok(min) => r.add_min(Min::Included(min)),
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMinimum(min) => match min.into_double() {
						Ok(min) => r.add_min(Min::Excluded(min)),
						Err(_) => todo!(),
					},
					Numeric::InclusiveMaximum(max) => match max.into_double() {
						Ok(max) => r.add_max(Max::Included(max)),
						Err(_) => todo!(),
					},
					Numeric::ExclusiveMaximum(max) => match max.into_double() {
						Ok(max) => r.add_max(Max::Excluded(max)),
						Err(_) => todo!(),
					},
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::Integer,
							restriction: other,
						}
						.into(),
						causes.preferred().cloned(),
					))
				}
			}
		}

		Ok(r)
	}

	pub fn build_string(self, id: Id) -> Result<restricted::string::Restrictions, Error<F>> {
		let mut p = restricted::string::Restrictions::default();

		for (restriction, causes) in self.map.into_iter() {
			match restriction {
				Restriction::String(restriction) => match restriction {
					String::Pattern(regexp) => p.add_pattern(regexp),
				},
				other => {
					return Err(Error::new(
						error::LayoutDatatypeRestrictionInvalid {
							id,
							primitive: Primitive::String,
							restriction: other,
						}
						.into(),
						causes.preferred().cloned(),
					))
				}
			}
		}

		Ok(p)
	}

	pub fn build_time(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Time,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_date(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Date,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_date_time(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::DateTime,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_iri(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Iri,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_uri(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Uri,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}

	pub fn build_url(self, id: Id) -> Result<(), Error<F>> {
		match self.map.into_iter().next() {
			Some((restriction, causes)) => Err(Error::new(
				error::LayoutDatatypeRestrictionInvalid {
					id,
					primitive: Primitive::Url,
					restriction,
				}
				.into(),
				causes.preferred().cloned(),
			)),
			None => Ok(()),
		}
	}
}
