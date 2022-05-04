use super::{error, Error};
use locspan::Location;
use treeldr::{vocab::Name, Caused, Causes, Documentation, Id, MaybeSet, Vocabulary, WithCauses};

/// Layout field definition.
#[derive(Clone)]
pub struct Definition<F> {
	id: Id,
	prop: MaybeSet<Id, F>,
	name: MaybeSet<Name, F>,
	layout: MaybeSet<Id, F>,
	required: MaybeSet<bool, F>,
	functional: MaybeSet<bool, F>,
}

impl<F> Definition<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			prop: MaybeSet::default(),
			name: MaybeSet::default(),
			layout: MaybeSet::default(),
			required: MaybeSet::default(),
			functional: MaybeSet::default(),
		}
	}

	pub fn property(&self) -> Option<&WithCauses<Id, F>> {
		self.prop.with_causes()
	}

	pub fn set_property(&mut self, prop_ref: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.prop
			.try_set(prop_ref, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutFieldMismatchProperty {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn replace_property(&mut self, prop: MaybeSet<Id, F>) {
		self.prop = prop
	}

	pub fn default_name(
		&self,
		vocabulary: &Vocabulary,
		cause: Option<Location<F>>,
	) -> Option<Caused<Name, F>>
	where
		F: Clone,
	{
		self.id
			.as_iri()
			.and_then(|term| term.iri(vocabulary))
			.and_then(|iri| {
				iri.path()
					.file_name()
					.and_then(|name| Name::try_from(name).ok())
			})
			.map(|name| Caused::new(name, cause))
	}

	pub fn name(&self) -> Option<&WithCauses<Name, F>> {
		self.name.with_causes()
	}

	pub fn set_name(&mut self, name: Name, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.name
			.try_set(name, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutFieldMismatchName {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn replace_name(&mut self, name: MaybeSet<Name, F>) {
		self.name = name
	}

	pub fn layout(&self) -> Option<&WithCauses<Id, F>> {
		self.layout.with_causes()
	}

	pub fn set_layout(&mut self, layout_ref: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.layout
			.try_set(layout_ref, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutFieldMismatchLayout {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn replace_layout(&mut self, layout: MaybeSet<Id, F>) {
		self.layout = layout
	}

	pub fn require_layout(&self, causes: &Causes<F>) -> Result<&WithCauses<Id, F>, Error<F>>
	where
		F: Clone,
	{
		self.layout.value_or_else(|| {
			Caused::new(
				error::LayoutFieldMissingLayout(self.id).into(),
				causes.preferred().cloned(),
			)
		})
	}

	pub fn is_required(&self) -> bool {
		self.required.value().cloned().unwrap_or(false)
	}

	pub fn set_required(&mut self, value: bool, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.required
			.try_set(value, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutFieldMismatchRequired {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn is_functional(&self) -> bool {
		self.functional.value().cloned().unwrap_or(true)
	}

	pub fn set_functional(
		&mut self,
		value: bool,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.functional
			.try_set(value, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutFieldMismatchFunctional {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}
}

pub trait Build<F> {
	fn require_name(&self) -> Result<WithCauses<Name, F>, Error<F>>;

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<F>,
	) -> Result<treeldr::layout::Field<F>, Error<F>>;
}

impl<F: Ord + Clone> Build<F> for WithCauses<Definition<F>, F> {
	fn require_name(&self) -> Result<WithCauses<Name, F>, Error<F>> {
		self.name.clone().ok_or_else(|| {
			Caused::new(
				error::LayoutFieldMissingName(self.id).into(),
				self.causes().preferred().cloned(),
			)
		})
	}

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<F>,
	) -> Result<treeldr::layout::Field<F>, Error<F>> {
		let prop = self.prop.clone().try_map_with_causes(|prop_id, causes| {
			Ok(**nodes.require_property(prop_id, causes.preferred().cloned())?)
		})?;

		let name = self.require_name()?;

		let layout_id = self.require_layout(self.causes())?;
		let layout = nodes
			.require_layout(*layout_id.inner(), layout_id.causes().preferred().cloned())?
			.clone_with_causes(layout_id.causes().clone());

		let required = self.required.clone().unwrap_or(false);

		Ok(treeldr::layout::Field::new(
			prop, name, label, layout, required, doc,
		))
	}
}
