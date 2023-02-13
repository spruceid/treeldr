use std::cmp::Ordering;

use crate::{
	context::{HasType, MapIds, MapIdsIn},
	functional_property_value, property_values, rdf,
	resource::BindingValueRef,
	Error, FunctionalPropertyValue, PropertyValues,
};
use locspan::Meta;
use treeldr::{
	metadata::Merge,
	prop::{RdfProperty, UnknownProperty},
	vocab::Object,
	Id, TId,
};

pub use treeldr::prop::{Property, Type};

/// Property definition.
#[derive(Clone)]
pub struct Definition<M> {
	/// Domain.
	domain: PropertyValues<Id, M>,

	/// Range.
	range: PropertyValues<Id, M>,

	/// Is the property required.
	required: FunctionalPropertyValue<bool, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			domain: PropertyValues::default(),
			range: PropertyValues::default(),
			required: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn range(&self) -> &PropertyValues<Id, M> {
		&self.range
	}

	pub fn range_mut(&mut self) -> &mut PropertyValues<Id, M> {
		&mut self.range
	}

	pub fn domain(&self) -> &PropertyValues<Id, M> {
		&self.domain
	}

	pub fn domain_mut(&mut self) -> &mut PropertyValues<Id, M> {
		&mut self.domain
	}

	pub fn required(&self) -> &FunctionalPropertyValue<bool, M> {
		&self.required
	}

	pub fn required_mut(&mut self) -> &mut FunctionalPropertyValue<bool, M> {
		&mut self.required
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			domain: self.domain.iter(),
			range: self.range.iter(),
			required: self.required.iter(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: RdfProperty,
		value: Meta<Object<M>, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			RdfProperty::Domain(p) => self
				.domain
				.insert(p, prop_cmp, rdf::from::expect_id(value)?),
			RdfProperty::Range(p) => self.range.insert(p, prop_cmp, rdf::from::expect_id(value)?),
			RdfProperty::Required(p) => {
				self.required
					.insert(p, prop_cmp, rdf::from::expect_schema_boolean(value)?)
			}
		}

		Ok(())
	}

	pub(crate) fn build(
		&self,
		context: &crate::Context<M>,
		as_resource: &treeldr::node::Data<M>,
		meta: M,
	) -> Result<Meta<treeldr::prop::Definition<M>, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let range = self
			.range
			.try_mapped(|_, Meta(range_id, range_meta)| {
				let range_ref = context.require_type_id(*range_id).map_err(|e| {
					e.at_node_property(as_resource.id, RdfProperty::Range(None), range_meta.clone())
				})?;
				Ok(Meta(range_ref, range_meta.clone()))
			})
			.map_err(|(Meta(e, _), _)| e)?;

		let required = self.required.clone().try_unwrap().map_err(|e| {
			e.at_functional_node_property(as_resource.id, RdfProperty::Required(None))
		})?;

		let functional = match as_resource.type_metadata(context, Type::FunctionalProperty) {
			Some(meta) => Meta(true, meta.clone()),
			None => Meta(false, meta.clone()),
		};

		let domain = self
			.domain
			.try_mapped(|_, Meta(domain_id, domain_meta)| {
				let domain_ref = context.require_type_id(*domain_id).map_err(|e| {
					e.at_node_property(
						as_resource.id,
						RdfProperty::Domain(None),
						domain_meta.clone(),
					)
				})?;
				Ok(Meta(domain_ref, domain_meta.clone()))
			})
			.map_err(|(Meta(e, _), _)| e)?;

		Ok(Meta(
			treeldr::prop::Definition::new(domain, range, required, functional),
			meta,
		))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		self.domain
			.map_ids_in(Some(RdfProperty::Domain(None).into()), &f);
		self.range
			.map_ids_in(Some(RdfProperty::Range(None).into()), f);
	}
}

#[derive(Debug)]
pub enum ClassBinding {
	Domain(Option<TId<UnknownProperty>>, Id),
	Range(Option<TId<UnknownProperty>>, Id),
	Required(Option<TId<UnknownProperty>>, bool),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> RdfProperty {
		match self {
			Self::Domain(p, _) => RdfProperty::Domain(*p),
			Self::Range(p, _) => RdfProperty::Range(*p),
			Self::Required(p, _) => RdfProperty::Required(*p),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Domain(_, v) => BindingValueRef::Id(*v),
			Self::Range(_, v) => BindingValueRef::Id(*v),
			Self::Required(_, v) => BindingValueRef::Boolean(*v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	domain: property_values::non_functional::Iter<'a, Id, M>,
	range: property_values::non_functional::Iter<'a, Id, M>,
	required: functional_property_value::Iter<'a, bool, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.domain
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::Domain))
			.or_else(|| {
				self.range
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBinding::Range))
					.or_else(|| {
						self.required
							.next()
							.map(|m| m.into_cloned_class_binding(ClassBinding::Required))
					})
			})
	}
}
