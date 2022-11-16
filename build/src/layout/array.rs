use crate::{node, Error, Single, Context};
use derivative::Derivative;
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use treeldr::{metadata::Merge, vocab, Id, IriIndex};
use vocab::{Rdf, Term};

#[derive(Clone, Debug, Derivative, StrippedPartialEq)]
#[derivative(Default(bound = ""))]
#[locspan(ignore(M))]
pub struct Semantics<M> {
	first: Single<Id, M>,

	rest: Single<Id, M>,

	nil: Single<Id, M>,
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

	pub fn first(&self) -> &Single<Id, M> {
		&self.first
	}

	pub fn rest(&self) -> &Single<Id, M> {
		&self.rest
	}

	pub fn nil(&self) -> &Single<Id, M> {
		&self.nil
	}

	pub fn set_first(&mut self, id: Meta<Id, M>)
	where
		M: Merge,
	{
		self.first.insert(id)
	}

	pub fn set_rest(&mut self, id: Meta<Id, M>)
	where
		M: Merge,
	{
		self.rest.insert(id)
	}

	pub fn set_nil(&mut self, id: Meta<Id, M>)
	where
		M: Merge,
	{
		self.nil.insert(id)
	}

	pub fn unify(mut self, other: Self) -> Self
	where
		M: Merge,
	{
		self.first.extend(other.first);
		self.rest.extend(other.rest);
		self.nil.extend(other.nil);
		self
	}

	pub fn build(
		self,
		model: &Context<M>,
		id: Id,
	) -> Result<Option<treeldr::layout::array::Semantics<M>>, Error<M>>
	where
		M: Clone,
	{
		let first = self.first.try_unwrap().map_err(|c| {
			c.at_functional_node_property(id, node::property::Layout::ArrayListFirst)
		})?;
		let rest = self.rest.try_unwrap().map_err(|c| {
			c.at_functional_node_property(id, node::property::Layout::ArrayListRest)
		})?;
		let nil = self
			.nil
			.try_unwrap()
			.map_err(|c| c.at_functional_node_property(id, node::property::Layout::ArrayListNil))?;

		let first = first.try_map_with_causes(|Meta(id, meta)| {
			Ok(Meta(
				**model.require_property(id).map_err(|e| {
					e.at_node_property(id, node::property::Layout::ArrayListFirst, meta.clone())
				})?,
				meta,
			))
		})?;
		let rest = rest.try_map_with_causes(|Meta(id, meta)| {
			Ok(Meta(
				**model.require_property(id).map_err(|e| {
					e.at_node_property(id, node::property::Layout::ArrayListRest, meta.clone())
				})?,
				meta,
			))
		})?;

		if first.is_some() || rest.is_some() || nil.is_some() {
			Ok(Some(treeldr::layout::array::Semantics::new(first, rest, nil)))
		} else {
			Ok(None)
		}
	}
}

impl<M> PartialEq for Semantics<M> {
	fn eq(&self, other: &Self) -> bool {
		self.first == other.first && self.rest == other.rest && self.nil == other.nil
	}
}
