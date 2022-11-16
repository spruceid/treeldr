use crate::{Error, Single, node, Multiple, single, multiple};
use locspan::Meta;
use treeldr::{metadata::Merge, Id};

/// Property definition.
#[derive(Clone)]
pub struct Definition<M> {
	/// `owl:FunctionalProperty` as a superclass.
	functional: Option<M>,

	/// Domain.
	domain: Multiple<Id, M>,

	/// Range.
	range: Single<Id, M>,

	/// Is the property required.
	required: Single<bool, M>,
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self {
			domain: Multiple::default(),
			range: Single::default(),
			required: Single::default(),
			functional: None,
		}
	}

	/// Checks if this property is functional,
	/// meaning that it is associated to at most one value.
	pub fn is_functional(&self) -> bool {
		self.functional.is_some()
	}

	pub fn declare_functional(&mut self, meta: M)
	where
		M: Merge,
	{
		match &mut self.functional {
			Some(m) => m.merge_with(meta),
			None => self.functional = Some(meta)
		}
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

	pub fn dependencies(
		&self,
		_nodes: &super::context::allocated::Nodes<M>,
		_causes: &M,
	) -> Result<Vec<crate::Item<M>>, Error<M>>
	where
		M: Clone,
	{
		Ok(Vec::new())
	}
}

impl<M: Clone> crate::Build<M> for Definition<M> {
	type Target = treeldr::prop::Definition<M>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<M>,
		_dependencies: crate::Dependencies<M>,
		id: Id,
		causes: M,
	) -> Result<Self::Target, Error<M>> {
		let range = self
			.range.into_required_type_at_node_binding(nodes, id, node::property::RdfProperty::Range, &causes)?;

		let required = self.required
			.try_unwrap()
			.map_err(|e| e.at_functional_node_property(id, node::property::RdfProperty::Required))?
			.unwrap()
			.unwrap_or_else(|| Meta(false, causes.clone()));
			
		let functional = match self.functional {
			Some(meta) => Meta(true, meta),
			None => Meta(false, causes.clone())
		};

		let mut result =
			treeldr::prop::Definition::new(id, range, required, functional, causes);

		for Meta(domain_id, domain_causes) in self.domain {
			let domain_ref = nodes.require_type(domain_id).map_err(|e| e.at_node_property(id, node::property::RdfProperty::Domain, domain_causes.clone()))?;
			result.insert_domain(**domain_ref, domain_causes)
		}

		Ok(result)
	}
}

pub enum BindingRef<'a, M> {
	Domain(Meta<Id, &'a M>),
	Range(Meta<Id, &'a M>),
	Required(Meta<bool, &'a M>)
}

pub struct Bindings<'a, M> {
	domain: multiple::Iter<'a, Id, M>,
	range: single::Iter<'a, Id, M>,
	required: single::Iter<'a, bool, M>
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.domain
			.next()
			.map(Meta::into_cloned_value)
			.map(BindingRef::Domain)
			.or_else(|| {
				self.range
					.next()
					.map(Meta::into_cloned_value)
					.map(BindingRef::Range)
					.or_else(|| {
						self.required
							.next()
							.map(Meta::into_cloned_value)
							.map(BindingRef::Required)
					})
			})
	}
}