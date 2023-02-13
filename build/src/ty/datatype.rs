use std::cmp::Ordering;

use crate::{
	context::{MapIds, MapIdsIn},
	functional_property_value::{self, FunctionalPropertyValue},
	rdf,
	resource::BindingValueRef,
	Context, Error, ObjectAsRequiredId,
};
use locspan::Meta;
use treeldr::{metadata::Merge, ty::data::Primitive, vocab::Object, Id};

pub use treeldr::ty::data::Property;

pub mod restriction;

pub use restriction::{Restriction, Restrictions};

#[derive(Clone)]
pub struct Definition<M> {
	/// Derived Datatype.
	base: FunctionalPropertyValue<Id, M>,

	/// List of restrictions.
	restrictions: FunctionalPropertyValue<Id, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			base: FunctionalPropertyValue::default(),
			restrictions: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn base(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.base
	}

	pub fn base_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.base
	}

	pub fn restrictions(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.restrictions
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			on_datatype: self.base.iter(),
			with_restrictions: self.restrictions.iter(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(Id, Id) -> Option<Ordering>,
		prop: Property,
		value: Meta<Object<M>, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::OnDatatype => self
				.base
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			Property::WithRestrictions => {
				self.restrictions
					.insert(None, prop_cmp, rdf::from::expect_id(value)?)
			}
		}

		Ok(())
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
		_meta: &M,
	) -> Result<treeldr::ty::data::Definition<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let restrictions = self.restrictions.clone().try_unwrap().map_err(|e| {
			e.at_functional_node_property(as_resource.id, Property::WithRestrictions)
		})?;

		let base = self
			.base
			.clone()
			.try_unwrap()
			.map_err(|e| e.at_functional_node_property(as_resource.id, Property::OnDatatype))?;

		let dt = match base.into_required() {
			None => match restrictions.into_required() {
				Some(_) => {
					todo!("restricted primitive datatype error")
				}
				None => treeldr::ty::DataType::Primitive(Primitive::from_id(as_resource.id)),
			},
			Some(base) => treeldr::ty::DataType::Derived(base.try_map_borrow_metadata(
				|base_id, base_meta| {
					let base = Meta(
						context.require_datatype_id(base_id).map_err(|e| {
							e.at_node_property(
								as_resource.id,
								Property::OnDatatype,
								base_meta.first().unwrap().value.into_metadata().clone(),
							)
						})?,
						base_meta.first().unwrap().value.into_metadata().clone(),
					);

					let primitive =
						Primitive::from_id(base.id()).expect("unknown primitive base datatype");

					let restrictions = restrictions
						.into_required()
						.map(|list_id| {
							let Meta(list_id, meta) = list_id.into_meta_value();

							let list = context.require_list(list_id).map_err(|e| {
								e.at_node_property(
									as_resource.id,
									Property::WithRestrictions,
									meta.clone(),
								)
							})?;

							let mut restrictions = Restrictions::new();

							for item in list.iter(context) {
								let Meta(object, restriction_meta) = item?.cloned();
								let restriction_id = object.into_required_id(&restriction_meta)?;
								let restriction = context
									.require(restriction_id)
									.map_err(|e| e.at(restriction_meta.clone()))?
									.require_datatype_restriction(context)
									.map_err(|e| e.at(restriction_meta.clone()))?;
								restrictions
									.insert(restriction.build()?.into_value(), restriction_meta)
							}

							Ok(Meta(restrictions, meta))
						})
						.transpose()?;

					Ok(match primitive {
						Primitive::Boolean => treeldr::ty::data::Derived::Boolean(base),
						Primitive::Date => treeldr::ty::data::Derived::Date(base),
						Primitive::DateTime => treeldr::ty::data::Derived::DateTime(base),
						Primitive::Double => treeldr::ty::data::Derived::Double(
							base,
							restrictions
								.map(|Meta(r, m)| r.build_double(as_resource.id, m))
								.transpose()?
								.into(),
						),
						Primitive::Duration => treeldr::ty::data::Derived::Duration(base),
						Primitive::Float => treeldr::ty::data::Derived::Float(
							base,
							restrictions
								.map(|Meta(r, m)| r.build_float(as_resource.id, m))
								.transpose()?
								.into(),
						),
						Primitive::Real => treeldr::ty::data::Derived::Real(
							base,
							restrictions
								.map(|Meta(r, m)| r.build_real(as_resource.id, m))
								.transpose()?
								.into(),
						),
						Primitive::String => treeldr::ty::data::Derived::String(
							base,
							restrictions
								.map(|Meta(r, m)| r.build_string(as_resource.id, m))
								.transpose()?
								.into(),
						),
						Primitive::Time => treeldr::ty::data::Derived::Time(base),
					})
				},
			)?),
		};

		Ok(treeldr::ty::data::Definition::new(dt))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.base.map_ids_in(Some(Property::OnDatatype.into()), &f);
		self.restrictions
			.map_ids_in(Some(Property::WithRestrictions.into()), f)
	}
}

#[derive(Debug)]
pub enum ClassBinding {
	OnDatatype(Option<Id>, Id),
	WithRestrictions(Option<Id>, Id),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::OnDatatype(_, _) => Property::OnDatatype,
			Self::WithRestrictions(_, _) => Property::WithRestrictions,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::OnDatatype(_, v) => BindingValueRef::Id(*v),
			Self::WithRestrictions(_, v) => BindingValueRef::Id(*v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	on_datatype: functional_property_value::Iter<'a, Id, M>,
	with_restrictions: functional_property_value::Iter<'a, Id, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_datatype
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::OnDatatype))
			.or_else(|| {
				self.with_restrictions
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBinding::WithRestrictions))
			})
	}
}
