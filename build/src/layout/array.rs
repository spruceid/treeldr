use crate::{error, Error};
use derivative::Derivative;
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use treeldr::{vocab, Id, MetaOption, Name, metadata::Merge};
use vocab::{Rdf, Term};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq(bound = ""))]
pub struct Array<M> {
	item: Id,

	semantics: Option<Semantics<M>>,
}

impl<M> Array<M> {
	pub fn new(item: Id, semantics: Option<Semantics<M>>) -> Self {
		Self { item, semantics }
	}

	pub fn item_layout(&self) -> Id {
		self.item
	}

	pub fn semantics(&self) -> Option<&Semantics<M>> {
		self.semantics.as_ref()
	}

	pub fn set_list_first(
		&mut self,
		id: Id,
		value: Id,
		metadata: M,
	) -> Result<(), Error<M>> {
		self.semantics
			.get_or_insert_with(Semantics::default)
			.set_first(id, value, metadata)
	}

	pub fn set_list_rest(
		&mut self,
		id: Id,
		value: Id,
		metadata: M,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.semantics
			.get_or_insert_with(Semantics::default)
			.set_rest(id, value, metadata)
	}

	pub fn set_list_nil(
		&mut self,
		id: Id,
		value: Id,
		metadata: M,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.semantics
			.get_or_insert_with(Semantics::default)
			.set_nil(id, value, metadata)
	}

	pub fn try_unify(
		self,
		other: Self,
		id: Id,
		metadata: M,
		other_metadata: M,
	) -> Result<Meta<Self, M>, Error<M>> where M: Merge {
		if self.item == other.item {
			let semantics = match (self.semantics, other.semantics) {
				(Some(a), Some(b)) => Some(a.try_unify(b, id)?),
				(Some(a), None) => Some(a),
				(None, Some(b)) => Some(b),
				(None, None) => None,
			};

			Ok(Meta(
				Self {
					item: self.item,
					semantics,
				},
				metadata.merged_with(other_metadata)
			))
		} else {
			Err(Error::new(
				error::LayoutMismatchDescription {
					id,
					because: metadata,
				}
				.into(),
				other_metadata,
			))
		}
	}

	pub fn build(
		self,
		name: MetaOption<Name, M>,
		nodes: &mut crate::context::allocated::Nodes<M>,
		metadata: &M,
	) -> Result<treeldr::layout::Array<M>, Error<M>>
	where
		M: Clone,
	{
		let item_layout_ref = **nodes
			.require_layout(self.item, metadata)?;
		let semantics = self.semantics.map(|s| s.build(nodes)).transpose()?;

		Ok(treeldr::layout::Array::new(
			name,
			item_layout_ref,
			semantics,
		))
	}
}

#[derive(Clone, Debug, Derivative, StrippedPartialEq)]
#[derivative(Default(bound = ""))]
#[stripped_ignore(M)]
pub struct Semantics<M> {
	first: MetaOption<Id, M>,

	rest: MetaOption<Id, M>,

	nil: MetaOption<Id, M>,
}

fn on_err<M>(id: Id, a_meta: M, b_meta: M) -> Error<M> {
	Error::new(
		error::LayoutMismatchDescription {
			id,
			because: a_meta,
		}
		.into(),
		b_meta,
	)
}

impl<M> Semantics<M> {
	pub fn rdf_list(metadata: M) -> Self
	where
		M: Clone,
	{
		Self {
			first: MetaOption::new(Id::Iri(Term::Rdf(Rdf::First)), metadata.clone()),
			rest: MetaOption::new(Id::Iri(Term::Rdf(Rdf::Rest)), metadata.clone()),
			nil: MetaOption::new(Id::Iri(Term::Rdf(Rdf::Nil)), metadata),
		}
	}

	pub fn first(&self) -> &MetaOption<Id, M> {
		&self.first
	}

	pub fn rest(&self) -> &MetaOption<Id, M> {
		&self.rest
	}

	pub fn nil(&self) -> &MetaOption<Id, M> {
		&self.nil
	}

	pub fn set_first(
		&mut self,
		id: Id,
		value: Id,
		metadata: M,
	) -> Result<(), Error<M>> {
		self.first
			.try_set(value, metadata, |Meta(_, a_meta), Meta(_, b_meta)| {
				on_err(id, a_meta, b_meta)
			})
	}

	pub fn set_rest(
		&mut self,
		id: Id,
		value: Id,
		metadata: M,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.rest
			.try_set(value, metadata, |Meta(_, a_meta), Meta(_, b_meta)| {
				on_err(id, a_meta, b_meta)
			})
	}

	pub fn set_nil(
		&mut self,
		id: Id,
		value: Id,
		metadata: M,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.nil.try_set(value, metadata, |Meta(_, a_meta), Meta(_, b_meta)| {
			on_err(id, a_meta, b_meta)
		})
	}

	pub fn try_unify(mut self, other: Self, id: Id) -> Result<Self, Error<M>> {
		self.first
			.try_set_opt(other.first, |Meta(_, a_meta), Meta(_, b_meta)| {
				on_err(id, a_meta, b_meta)
			})?;
		self.rest
			.try_set_opt(other.rest, |Meta(_, a_meta), Meta(_, b_meta)| {
				on_err(id, a_meta, b_meta)
			})?;
		self.nil
			.try_set_opt(other.nil, |Meta(_, a_meta), Meta(_, b_meta)| {
				on_err(id, a_meta, b_meta)
			})?;

		Ok(self)
	}

	pub fn build(
		self,
		nodes: &mut crate::context::allocated::Nodes<M>,
	) -> Result<treeldr::layout::array::Semantics<M>, Error<M>>
	where
		M: Clone,
	{
		let first = self.first.try_map_with_causes(|Meta(id, metadata)| {
			Ok(Meta(**nodes.require_property(id, &metadata)?, metadata))
		})?;
		let rest = self.rest.try_map_with_causes(|Meta(id, metadata)| {
			Ok(Meta(**nodes.require_property(id, &metadata)?, metadata))
		})?;

		Ok(treeldr::layout::array::Semantics::new(
			first, rest, self.nil,
		))
	}
}

impl<M> PartialEq for Semantics<M> {
	fn eq(&self, other: &Self) -> bool {
		self.first.value() == other.first.value()
			&& self.rest.value() == other.rest.value()
			&& self.nil.value() == other.nil.value()
	}
}
