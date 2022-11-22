use crate::{Error, Single, error::NodeBindingMissing, ObjectAsRequiredId, single, Context};
use locspan::Meta;
use treeldr::{metadata::Merge, Id, ty::data::Primitive};

pub use treeldr::ty::data::Property;

pub mod restriction;

pub use restriction::{Restriction, Restrictions};

#[derive(Clone)]
pub struct Definition<M> {
	/// Derived Datatype.
	base: Single<Id, M>,

	/// List of restrictions.
	restrictions: Single<Id, M>
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self { base: Single::default(), restrictions: Single::default() }
	}
}

impl<M> Definition<M> {
	pub fn base(&self) -> &Single<Id, M> {
		&self.base
	}

	pub fn base_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.base
	}

	pub fn restrictions(&self) -> &Single<Id, M> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.restrictions
	}

	// pub fn dependencies(
	// 	&self,
	// 	nodes: &context::allocated::Nodes<M>,
	// 	id: Id,
	// ) -> Result<Vec<crate::Item<M>>, Error<M>>
	// where
	// 	M: Clone,
	// {
	// 	match self {
	// 		Self::Unknown => todo!(),
	// 		Self::Primitive(_) => Ok(Vec::new()),
	// 		Self::Derived(d) => d.dependencies(nodes, id),
	// 	}
	// }

	pub fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		meta: &M
	) -> Result<treeldr::ty::data::Definition, Error<M>>
	where
		M: Clone + Merge,
	{
		let restrictions = self.restrictions.clone().try_unwrap().map_err(|e| e.at_functional_node_property(as_resource.id, Property::WithRestrictions))?;
		let dt = match Primitive::from_id(as_resource.id) {
			Some(primitive) => {
				match restrictions.unwrap() {
					Some(_) => {
						todo!("restricted primitive datatype error")
					}
					None => {
						treeldr::ty::DataType::Primitive(primitive)
					}
				}
			}
			None => {
				match restrictions.unwrap() {
					Some(list_id) => {
						let Meta(base, _) = self.base.clone().into_required_at_node_binding(as_resource.id, Property::OnDatatype, meta)?;
						let primitive = Primitive::from_id(base).ok_or_else(|| todo!())?;

						let list = context
							.require_list(*list_id)
							.map_err(|e| e.at_node_property(as_resource.id, Property::WithRestrictions, list_id.metadata().clone()))?;

						let mut restrictions = Restrictions::new();

						for item in list.iter(context) {
							let Meta(object, restriction_meta) = item?.cloned();
							let restriction_id = object.into_required_id(&restriction_meta)?;
							let restriction = context
								.require(restriction_id)
								.map_err(|e| e.at(restriction_meta.clone()))?
								.require_datatype_restriction(context)
								.map_err(|e| e.at(restriction_meta.clone()))?;
							restrictions.insert(restriction.build()?.into_value(), restriction_meta)
						}

						let derived = match primitive {
							Primitive::Boolean => treeldr::ty::data::Derived::Boolean(base),
							Primitive::Date => treeldr::ty::data::Derived::Date(base),
							Primitive::DateTime => treeldr::ty::data::Derived::DateTime(base),
							Primitive::Double => treeldr::ty::data::Derived::Double(base, restrictions.build_double(as_resource.id)?),
							Primitive::Duration => treeldr::ty::data::Derived::Duration(base),
							Primitive::Float => treeldr::ty::data::Derived::Float(base, restrictions.build_float(as_resource.id)?),
							Primitive::Real => treeldr::ty::data::Derived::Real(base, restrictions.build_real(as_resource.id)?),
							Primitive::String => treeldr::ty::data::Derived::String(base, restrictions.build_string(as_resource.id)?),
							Primitive::Time => treeldr::ty::data::Derived::Time(base)
						};

						treeldr::ty::DataType::Derived(derived)
					}
					None => {
						return Err(Meta(
							NodeBindingMissing {
								id: as_resource.id,
								property: Property::WithRestrictions.into()
							}.into(),
							meta.clone()
						))
					}
				}
			}
		};

		Ok(treeldr::ty::data::Definition::new(dt))
	}
}

pub enum Binding {
	OnDatatype(Id),
	WithRestrictions(Id)
}

pub struct Bindings<'a, M> {
	on_datatype: single::Iter<'a, Id, M>,
	with_restrictions: single::Iter<'a, Id, M>
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<Binding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_datatype
			.next()
			.map(Meta::into_cloned_value)
			.map(|m| m.map(Binding::OnDatatype))
			.or_else(|| {
				self.with_restrictions
					.next()
					.map(Meta::into_cloned_value)
					.map(|m| m.map(Binding::WithRestrictions))
			})
	}
}