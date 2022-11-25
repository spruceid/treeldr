use crate::{
	context::{HasType, MapIds, MapIdsIn},
	multiple,
	resource::BindingValueRef,
	single, Error, Multiple, Single,
};
use locspan::Meta;
use treeldr::{metadata::Merge, prop::RdfProperty, Id};

pub use treeldr::prop::{Property, Type};

/// Property definition.
#[derive(Clone)]
pub struct Definition<M> {
	/// Domain.
	domain: Multiple<Id, M>,

	/// Range.
	range: Single<Id, M>,

	/// Is the property required.
	required: Single<bool, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			domain: Multiple::default(),
			range: Single::default(),
			required: Single::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn range(&self) -> &Single<Id, M> {
		&self.range
	}

	pub fn range_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.range
	}

	pub fn domain(&self) -> &Multiple<Id, M> {
		&self.domain
	}

	pub fn domain_mut(&mut self) -> &mut Multiple<Id, M> {
		&mut self.domain
	}

	pub fn required(&self) -> &Single<bool, M> {
		&self.required
	}

	pub fn required_mut(&mut self) -> &mut Single<bool, M> {
		&mut self.required
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			domain: self.domain.iter(),
			range: self.range.iter(),
			required: self.required.iter(),
		}
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
		let range = self.range.clone().into_required_type_at_node_binding(
			context,
			as_resource.id,
			RdfProperty::Range,
			&meta,
		)?;

		let required = self
			.required
			.clone()
			.try_unwrap()
			.map_err(|e| e.at_functional_node_property(as_resource.id, RdfProperty::Required))?
			.unwrap()
			.unwrap_or_else(|| Meta(false, meta.clone()));

		let functional = match as_resource.type_metadata(context, Type::FunctionalProperty) {
			Some(meta) => Meta(true, meta.clone()),
			None => Meta(false, meta.clone()),
		};

		let mut domain = Multiple::default();
		for Meta(domain_id, domain_causes) in &self.domain {
			let domain_ref = context.require_type_id(*domain_id).map_err(|e| {
				e.at_node_property(as_resource.id, RdfProperty::Domain, domain_causes.clone())
			})?;
			domain.insert(Meta(domain_ref, domain_causes.clone()))
		}

		Ok(Meta(
			treeldr::prop::Definition::new(domain, range, required, functional),
			meta,
		))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		self.domain.map_ids_in(Some(RdfProperty::Domain.into()), &f);
		self.range.map_ids_in(Some(RdfProperty::Range.into()), f);
	}
}

pub enum ClassBinding {
	Domain(Id),
	Range(Id),
	Required(bool),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> RdfProperty {
		match self {
			Self::Domain(_) => RdfProperty::Domain,
			Self::Range(_) => RdfProperty::Range,
			Self::Required(_) => RdfProperty::Required,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Domain(v) => BindingValueRef::Id(*v),
			Self::Range(v) => BindingValueRef::Id(*v),
			Self::Required(v) => BindingValueRef::Boolean(*v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	domain: multiple::Iter<'a, Id, M>,
	range: single::Iter<'a, Id, M>,
	required: single::Iter<'a, bool, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.domain
			.next()
			.map(Meta::into_cloned_value)
			.map(|m| m.map(ClassBinding::Domain))
			.or_else(|| {
				self.range
					.next()
					.map(Meta::into_cloned_value)
					.map(|m| m.map(ClassBinding::Range))
					.or_else(|| {
						self.required
							.next()
							.map(Meta::into_cloned_value)
							.map(|m| m.map(ClassBinding::Required))
					})
			})
	}
}
