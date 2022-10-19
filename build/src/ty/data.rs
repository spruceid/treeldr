use crate::{context, Error};
use locspan::Meta;
use treeldr::{metadata::Merge, Id};

pub mod restriction;

pub use restriction::{Restriction, Restrictions};
pub use treeldr::ty::data::Primitive;

#[derive(Clone)]
pub enum DataType<M> {
	Unknown,
	Primitive(Meta<Primitive, M>),
	Derived(Derived<M>),
}

impl<M> Default for DataType<M> {
	fn default() -> Self {
		Self::Unknown
	}
}

impl<M> DataType<M> {
	pub fn set_primitive(&mut self, p: Primitive, cause: M) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match self {
			Self::Unknown => {
				*self = Self::Primitive(Meta(p, cause));
				Ok(())
			}
			Self::Primitive(q) => {
				if p == **q {
					q.metadata_mut().merge_with(cause);
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

	pub fn set_derivation_base(&mut self, base: Id, cause: M) -> Result<(), Error<M>>
	where
		M: Merge,
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

	pub fn as_derived_mut(&mut self) -> Option<&mut Derived<M>> {
		match self {
			Self::Derived(d) => Some(d),
			_ => None,
		}
	}

	pub fn dependencies(
		&self,
		nodes: &context::allocated::Nodes<M>,
	) -> Result<Vec<crate::Item<M>>, Error<M>>
	where
		M: Clone,
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
		nodes: &context::allocated::Nodes<M>,
		dependencies: crate::Dependencies<M>,
	) -> Result<treeldr::ty::Description<M>, Error<M>>
	where
		M: Clone,
	{
		let dt = match self {
			Self::Unknown => todo!(),
			Self::Primitive(p) => treeldr::ty::DataType::Primitive(*p),
			Self::Derived(d) => treeldr::ty::DataType::Derived(d.build(id, nodes, dependencies)?),
		};

		Ok(treeldr::ty::Description::Data(dt))
	}
}

#[derive(Clone)]
pub struct Derived<M> {
	base: Meta<Id, M>,
	restrictions: Restrictions<M>,
}

impl<M> Derived<M> {
	pub fn new(base: Id, metadata: M) -> Self {
		Self {
			base: Meta(base, metadata),
			restrictions: Restrictions::new(),
		}
	}

	pub fn set_base(&mut self, base: Id, cause: M) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		if *self.base == base {
			self.base.metadata_mut().merge_with(cause);
			Ok(())
		} else {
			todo!()
		}
	}

	pub fn restrictions_mut(&mut self) -> &mut Restrictions<M> {
		&mut self.restrictions
	}

	pub fn primitive(
		&self,
		nodes: &context::allocated::Nodes<M>,
		dependencies: crate::Dependencies<M>,
	) -> Result<Primitive, Error<M>>
	where
		M: Clone,
	{
		let base_id = *self.base;
		let base_ref = **nodes.require_type(base_id, self.base.metadata())?;
		let base = dependencies.ty(base_ref);
		match base.description() {
			treeldr::ty::Description::Data(dt) => Ok(dt.primitive()),
			_other => todo!(),
		}
	}

	pub fn dependencies(
		&self,
		nodes: &context::allocated::Nodes<M>,
	) -> Result<Vec<crate::Item<M>>, Error<M>>
	where
		M: Clone,
	{
		Ok(vec![crate::Item::Type(
			**nodes.require_type(*self.base, self.base.metadata())?,
		)])
	}

	pub fn build(
		self,
		id: Id,
		nodes: &context::allocated::Nodes<M>,
		dependencies: crate::Dependencies<M>,
	) -> Result<treeldr::ty::data::Derived, Error<M>>
	where
		M: Clone,
	{
		let base_id = *self.base;
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
