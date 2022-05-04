use crate::{error, Error};
use derivative::Derivative;
use treeldr::{vocab, Causes, Id, MaybeSet};
use vocab::{Rdf, Term};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq(bound = ""))]
pub struct Array<F> {
	item: Id,

	semantics: Option<Semantics<F>>,
}

impl<F> Array<F> {
	pub fn new(item: Id, semantics: Option<Semantics<F>>) -> Self {
		Self { item, semantics }
	}

	pub fn item_layout(&self) -> Id {
		self.item
	}

	pub fn semantics(&self) -> Option<&Semantics<F>> {
		self.semantics.as_ref()
	}

	pub fn set_list_first(
		&mut self,
		id: Id,
		value: Id,
		causes: impl Into<Causes<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.semantics
			.get_or_insert_with(Semantics::default)
			.set_first(id, value, causes)
	}

	pub fn set_list_rest(
		&mut self,
		id: Id,
		value: Id,
		causes: impl Into<Causes<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.semantics
			.get_or_insert_with(Semantics::default)
			.set_rest(id, value, causes)
	}

	pub fn set_list_nil(
		&mut self,
		id: Id,
		value: Id,
		causes: impl Into<Causes<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.semantics
			.get_or_insert_with(Semantics::default)
			.set_nil(id, value, causes)
	}

	pub fn try_unify(
		self,
		other: Self,
		id: Id,
		causes: &Causes<F>,
		other_causes: &Causes<F>,
	) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		if self.item == other.item {
			let semantics = match (self.semantics, other.semantics) {
				(Some(a), Some(b)) => Some(a.try_unify(b, id)?),
				(Some(a), None) => Some(a),
				(None, Some(b)) => Some(b),
				(None, None) => None,
			};

			Ok(Self {
				item: self.item,
				semantics,
			})
		} else {
			Err(Error::new(
				error::LayoutMismatchDescription {
					id,
					because: causes.preferred().cloned(),
				}
				.into(),
				other_causes.preferred().cloned(),
			))
		}
	}

	pub fn build(
		self,
		name: MaybeSet<vocab::Name, F>,
		nodes: &mut crate::context::allocated::Nodes<F>,
		causes: &Causes<F>,
	) -> Result<treeldr::layout::Array<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let item_layout_ref = *nodes
			.require_layout(self.item, causes.preferred().cloned())?
			.inner();
		let semantics = self.semantics.map(|s| s.build(nodes)).transpose()?;

		Ok(treeldr::layout::Array::new(
			name,
			item_layout_ref,
			semantics,
		))
	}
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Semantics<F> {
	first: MaybeSet<Id, F>,
	rest: MaybeSet<Id, F>,
	nil: MaybeSet<Id, F>,
}

fn on_err<F: Clone + Ord>(id: Id, a_causes: &Causes<F>, b_causes: &Causes<F>) -> Error<F> {
	Error::new(
		error::LayoutMismatchDescription {
			id,
			because: a_causes.preferred().cloned(),
		}
		.into(),
		b_causes.preferred().cloned(),
	)
}

impl<F> Semantics<F> {
	pub fn rdf_list(causes: impl Into<Causes<F>>) -> Self
	where
		F: Clone,
	{
		let causes = causes.into();
		Self {
			first: MaybeSet::new(Id::Iri(Term::Rdf(Rdf::First)), causes.clone()),
			rest: MaybeSet::new(Id::Iri(Term::Rdf(Rdf::Rest)), causes.clone()),
			nil: MaybeSet::new(Id::Iri(Term::Rdf(Rdf::Nil)), causes),
		}
	}

	pub fn first(&self) -> &MaybeSet<Id, F> {
		&self.first
	}

	pub fn rest(&self) -> &MaybeSet<Id, F> {
		&self.rest
	}

	pub fn nil(&self) -> &MaybeSet<Id, F> {
		&self.nil
	}

	pub fn set_first(
		&mut self,
		id: Id,
		value: Id,
		causes: impl Into<Causes<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.first
			.try_set(value, causes, |_, _, a_causes, b_causes| {
				on_err(id, a_causes, b_causes)
			})
	}

	pub fn set_rest(
		&mut self,
		id: Id,
		value: Id,
		causes: impl Into<Causes<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.rest
			.try_set(value, causes, |_, _, a_causes, b_causes| {
				on_err(id, a_causes, b_causes)
			})
	}

	pub fn set_nil(
		&mut self,
		id: Id,
		value: Id,
		causes: impl Into<Causes<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.nil.try_set(value, causes, |_, _, a_causes, b_causes| {
			on_err(id, a_causes, b_causes)
		})
	}

	pub fn try_unify(mut self, other: Self, id: Id) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		self.first
			.try_set_opt(other.first, |_, _, a_causes, b_causes| {
				on_err(id, a_causes, b_causes)
			})?;
		self.rest
			.try_set_opt(other.rest, |_, _, a_causes, b_causes| {
				on_err(id, a_causes, b_causes)
			})?;
		self.nil
			.try_set_opt(other.nil, |_, _, a_causes, b_causes| {
				on_err(id, a_causes, b_causes)
			})?;

		Ok(self)
	}

	pub fn build(
		self,
		nodes: &mut crate::context::allocated::Nodes<F>,
	) -> Result<treeldr::layout::array::Semantics<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let first = self.first.try_map_with_causes(|id, causes| {
			Ok(**nodes.require_property(id, causes.preferred().cloned())?)
		})?;
		let rest = self.rest.try_map_with_causes(|id, causes| {
			Ok(**nodes.require_property(id, causes.preferred().cloned())?)
		})?;

		Ok(treeldr::layout::array::Semantics::new(
			first, rest, self.nil,
		))
	}
}

impl<F> PartialEq for Semantics<F> {
	fn eq(&self, other: &Self) -> bool {
		self.first.value() == other.first.value()
			&& self.rest.value() == other.rest.value()
			&& self.nil.value() == other.nil.value()
	}
}
