use crate::{context, Error};
use locspan::Location;
use treeldr::{Causes, Id, WithCauses};

pub mod restriction;

pub use restriction::{Restriction, Restrictions};
pub use treeldr::ty::data::Primitive;

#[derive(Clone)]
pub enum DataType<F> {
	Unknown,
	Primitive(WithCauses<Primitive, F>),
	Derived(Derived<F>),
}

impl<F> Default for DataType<F> {
	fn default() -> Self {
		Self::Unknown
	}
}

impl<F> DataType<F> {
	pub fn set_primitive(
		&mut self,
		p: Primitive,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord,
	{
		match self {
			Self::Unknown => {
				*self = Self::Primitive(WithCauses::new(p, cause));
				Ok(())
			}
			Self::Primitive(q) => {
				if p == *q.inner() {
					q.add_opt_cause(cause);
					Ok(())
				} else {
					todo!()
				}
			}
			Self::Derived(_d) => {
				todo!()
			}
		}
	}

	pub fn set_derivation_base(
		&mut self,
		base: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord,
	{
		match self {
			Self::Unknown => {
				*self = Self::Derived(Derived::new(base, cause));
				Ok(())
			}
			Self::Derived(d) => d.set_base(base, cause),
			Self::Primitive(_) => {
				todo!()
			}
		}
	}

	pub fn as_derived_mut(&mut self) -> Option<&mut Derived<F>> {
		match self {
			Self::Derived(d) => Some(d),
			_ => None,
		}
	}

	pub fn dependencies(
		&self,
		nodes: &context::allocated::Nodes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>>
	where
		F: Clone + Ord,
	{
		match self {
			Self::Unknown => todo!(),
			Self::Primitive(_) => Ok(Vec::new()),
			Self::Derived(d) => d.dependencies(nodes),
		}
	}

	pub fn build(
		self,
		id: Id,
		nodes: &context::allocated::Nodes<F>,
		dependencies: crate::Dependencies<F>,
	) -> Result<treeldr::ty::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let dt = match self {
			Self::Unknown => todo!(),
			Self::Primitive(p) => treeldr::ty::DataType::Primitive(p.into_inner()),
			Self::Derived(d) => treeldr::ty::DataType::Derived(d.build(id, nodes, dependencies)?),
		};

		Ok(treeldr::ty::Description::Data(dt))
	}
}

#[derive(Clone)]
pub struct Derived<F> {
	base: WithCauses<Id, F>,
	restrictions: Restrictions<F>,
}

impl<F> Derived<F> {
	pub fn new(base: Id, causes: impl Into<Causes<F>>) -> Self
	where
		F: Ord,
	{
		Self {
			base: WithCauses::new(base, causes),
			restrictions: Restrictions::new(),
		}
	}

	pub fn set_base(&mut self, base: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord,
	{
		if *self.base.inner() == base {
			self.base.add_opt_cause(cause);
			Ok(())
		} else {
			todo!()
		}
	}

	pub fn restrictions_mut(&mut self) -> &mut Restrictions<F> {
		&mut self.restrictions
	}

	pub fn primitive(
		&self,
		nodes: &context::allocated::Nodes<F>,
		dependencies: crate::Dependencies<F>,
	) -> Result<Primitive, Error<F>>
	where
		F: Clone + Ord,
	{
		let base_id = *self.base.inner();
		let base_ref = **nodes.require_type(base_id, self.base.causes().preferred().cloned())?;
		let base = dependencies.ty(base_ref);
		match base.description() {
			treeldr::ty::Description::Data(dt) => Ok(dt.primitive()),
			_other => todo!(),
		}
	}

	pub fn dependencies(
		&self,
		nodes: &context::allocated::Nodes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>>
	where
		F: Clone + Ord,
	{
		Ok(vec![crate::Item::Type(**nodes.require_type(
			*self.base.inner(),
			self.base.causes().preferred().cloned(),
		)?)])
	}

	pub fn build(
		self,
		id: Id,
		nodes: &context::allocated::Nodes<F>,
		dependencies: crate::Dependencies<F>,
	) -> Result<treeldr::ty::data::Derived, Error<F>>
	where
		F: Clone + Ord,
	{
		let base_id = *self.base.inner();
		match self.primitive(nodes, dependencies)? {
			Primitive::Boolean => {
				self.restrictions.build_boolean(id)?;
				Ok(treeldr::ty::data::Derived::Boolean(base_id))
			}
			Primitive::Real => Ok(treeldr::ty::data::Derived::Real(
				base_id,
				self.restrictions.build_real(id)?,
			)),
			Primitive::Float => Ok(treeldr::ty::data::Derived::Float(
				base_id,
				self.restrictions.build_float(id)?,
			)),
			Primitive::Double => Ok(treeldr::ty::data::Derived::Double(
				base_id,
				self.restrictions.build_double(id)?,
			)),
			Primitive::String => Ok(treeldr::ty::data::Derived::String(
				base_id,
				self.restrictions.build_string(id)?,
			)),
			Primitive::Date => {
				self.restrictions.build_date(id)?;
				Ok(treeldr::ty::data::Derived::Date(base_id))
			}
			Primitive::Time => {
				self.restrictions.build_time(id)?;
				Ok(treeldr::ty::data::Derived::Time(base_id))
			}
			Primitive::DateTime => {
				self.restrictions.build_datetime(id)?;
				Ok(treeldr::ty::data::Derived::DateTime(base_id))
			}
			Primitive::Duration => {
				self.restrictions.build_duration(id)?;
				Ok(treeldr::ty::data::Derived::Duration(base_id))
			}
		}
	}
}
