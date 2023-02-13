use std::cmp::Ordering;

use super::Property;
use crate::{
	context::{MapIds, MapIdsIn},
	functional_property_value,
	resource::BindingValueRef,
	Context, Error, FunctionalPropertyValue,
};
use derivative::Derivative;
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use treeldr::{metadata::Merge, vocab, Id, IriIndex};
use vocab::{Rdf, Term};

#[derive(Clone, Debug, Derivative, StrippedPartialEq)]
#[derivative(Default(bound = ""))]
#[locspan(ignore(M))]
pub struct Semantics<M> {
	first: FunctionalPropertyValue<Id, M>,

	rest: FunctionalPropertyValue<Id, M>,

	nil: FunctionalPropertyValue<Id, M>,
}

impl<M> Semantics<M> {
	pub fn rdf_list(metadata: M) -> Self
	where
		M: Clone,
	{
		Self {
			first: Meta(
				Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
				metadata.clone(),
			)
			.into(),
			rest: Meta(
				Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
				metadata.clone(),
			)
			.into(),
			nil: Meta(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Nil))), metadata).into(),
		}
	}

	pub fn first(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.first
	}

	pub fn rest(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.rest
	}

	pub fn nil(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.nil
	}

	pub fn set_first(&mut self, id: Meta<Id, M>)
	where
		M: Merge,
	{
		self.first.insert_base(id)
	}

	pub fn set_rest(&mut self, id: Meta<Id, M>)
	where
		M: Merge,
	{
		self.rest.insert_base(id)
	}

	pub fn set_nil(&mut self, id: Meta<Id, M>)
	where
		M: Merge,
	{
		self.nil.insert_base(id)
	}

	pub fn unify_with(&mut self, prop_cmp: impl Fn(Id, Id) -> Option<Ordering>, other: Self)
	where
		M: Merge,
	{
		self.first.extend_with(&prop_cmp, other.first);
		self.rest.extend_with(&prop_cmp, other.rest);
		self.nil.extend_with(prop_cmp, other.nil);
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			first: self.first.iter(),
			rest: self.rest.iter(),
			nil: self.nil.iter(),
		}
	}

	pub fn build(
		self,
		model: &Context<M>,
		id: Id,
	) -> Result<Option<treeldr::layout::array::Semantics<M>>, Error<M>>
	where
		M: Clone,
	{
		let first = self
			.first
			.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, Property::ArrayListFirst))?;
		let rest = self
			.rest
			.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, Property::ArrayListRest))?;
		let nil = self
			.nil
			.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, Property::ArrayListNil))?;

		let first = first.try_map_borrow_metadata(|id, meta| {
			let meta = meta.first().unwrap().value.into_metadata();
			model
				.require_property_id(id)
				.map_err(|e| e.at_node_property(id, Property::ArrayListFirst, meta.clone()))
		})?;
		let rest = rest.try_map_borrow_metadata(|id, meta| {
			let meta = meta.first().unwrap().value.into_metadata();
			model
				.require_property_id(id)
				.map_err(|e| e.at_node_property(id, Property::ArrayListRest, meta.clone()))
		})?;

		if first.is_some() || rest.is_some() || nil.is_some() {
			Ok(Some(treeldr::layout::array::Semantics::new(
				first, rest, nil,
			)))
		} else {
			Ok(None)
		}
	}
}

impl<M: Merge> MapIds for Semantics<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.first
			.map_ids_in(Some(Property::ArrayListFirst.into()), &f);
		self.rest
			.map_ids_in(Some(Property::ArrayListRest.into()), &f);
		self.nil.map_ids_in(Some(Property::ArrayListNil.into()), f)
	}
}

impl<M> PartialEq for Semantics<M> {
	fn eq(&self, other: &Self) -> bool {
		self.first == other.first && self.rest == other.rest && self.nil == other.nil
	}
}

#[derive(Debug)]
pub enum Binding {
	ArrayListFirst(Option<Id>, Id),
	ArrayListRest(Option<Id>, Id),
	ArrayListNil(Option<Id>, Id),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::ArrayListFirst(_, _) => Property::ArrayListFirst,
			Self::ArrayListRest(_, _) => Property::ArrayListRest,
			Self::ArrayListNil(_, _) => Property::ArrayListNil,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::ArrayListFirst(_, v) => BindingValueRef::Id(*v),
			Self::ArrayListRest(_, v) => BindingValueRef::Id(*v),
			Self::ArrayListNil(_, v) => BindingValueRef::Id(*v),
		}
	}
}

pub struct Bindings<'a, M> {
	first: functional_property_value::Iter<'a, Id, M>,
	rest: functional_property_value::Iter<'a, Id, M>,
	nil: functional_property_value::Iter<'a, Id, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<Binding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.first
			.next()
			.map(|m| m.into_cloned_class_binding(Binding::ArrayListFirst))
			.or_else(|| {
				self.rest
					.next()
					.map(|m| m.into_cloned_class_binding(Binding::ArrayListRest))
					.or_else(|| {
						self.nil
							.next()
							.map(|m| m.into_cloned_class_binding(Binding::ArrayListNil))
					})
			})
	}
}
