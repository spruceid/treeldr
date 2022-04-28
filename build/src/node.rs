use crate::{error, layout, list, prop, ty, Descriptions, Error};
use locspan::Location;
use treeldr::{Caused, Causes, Documentation, Id, MaybeSet, WithCauses};

pub use treeldr::node::{CausedTypes, Type, Types};

pub struct Node<T> {
	id: Id,
	label: Option<String>,
	doc: Documentation,
	value: T,
}

pub struct Components<F, D: Descriptions<F>> {
	pub ty: MaybeSet<ty::Definition<F, D::Type>, F>,
	pub property: MaybeSet<prop::Definition<F>, F>,
	pub layout: MaybeSet<layout::Definition<F, D::Layout>, F>,
	pub layout_field: MaybeSet<layout::field::Definition<F>, F>,
	pub layout_variant: MaybeSet<layout::variant::Definition<F>, F>,
	pub list: MaybeSet<list::Definition<F>, F>,
}

impl<T> Node<T> {
	pub fn new_with(id: Id, value: T) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::new(),
			value,
		}
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn add_label(&mut self, label: String) {
		self.label = Some(label)
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn documentation_mut(&mut self) -> &mut Documentation {
		&mut self.doc
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn value_mut(&mut self) -> &mut T {
		&mut self.value
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Node<U> {
		Node {
			id: self.id,
			label: self.label,
			doc: self.doc,
			value: f(self.value),
		}
	}

	pub fn into_parts(self) -> (Id, Option<String>, Documentation, T) {
		(self.id, self.label, self.doc, self.value)
	}
}

pub type PropertyOrLayoutField<'a, F> = (
	Option<&'a mut WithCauses<prop::Definition<F>, F>>,
	Option<&'a mut WithCauses<layout::field::Definition<F>, F>>,
);

pub type LayoutFieldOrVariant<'a, F> = (
	Option<&'a mut WithCauses<layout::field::Definition<F>, F>>,
	Option<&'a mut WithCauses<layout::variant::Definition<F>, F>>,
);

impl<F, D: Descriptions<F>> Node<Components<F, D>> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				layout_variant: MaybeSet::default(),
				list: MaybeSet::default(),
			},
		}
	}

	pub fn new_type(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::new(ty::Definition::new(id), causes),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				layout_variant: MaybeSet::default(),
				list: MaybeSet::default(),
			},
		}
	}

	pub fn new_property(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::new(prop::Definition::new(id), causes),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				layout_variant: MaybeSet::default(),
				list: MaybeSet::default(),
			},
		}
	}

	pub fn new_layout(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::new(layout::Definition::new(id), causes),
				layout_field: MaybeSet::default(),
				layout_variant: MaybeSet::default(),
				list: MaybeSet::default(),
			},
		}
	}

	pub fn new_layout_field(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::new(layout::field::Definition::new(id), causes),
				layout_variant: MaybeSet::default(),
				list: MaybeSet::default(),
			},
		}
	}

	pub fn new_layout_variant(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				layout_variant: MaybeSet::new(layout::variant::Definition::new(id), causes),
				list: MaybeSet::default(),
			},
		}
	}

	pub fn new_list(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				layout_variant: MaybeSet::default(),
				list: MaybeSet::new(list::Definition::new(id), causes),
			},
		}
	}

	pub fn types(&self) -> Types {
		Types {
			ty: self.value.ty.is_set(),
			property: self.value.property.is_set(),
			layout: self.value.layout.is_set(),
			layout_field: self.value.layout_field.is_set(),
			layout_variant: self.value.layout_variant.is_set(),
			list: self.value.list.is_set(),
		}
	}

	pub fn caused_types(&self) -> CausedTypes<F>
	where
		F: Clone,
	{
		CausedTypes {
			ty: self
				.value
				.ty
				.causes()
				.map(|causes| causes.preferred().cloned()),
			property: self
				.value
				.property
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout: self
				.value
				.layout
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout_field: self
				.value
				.layout_field
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout_variant: self
				.value
				.layout_variant
				.causes()
				.map(|causes| causes.preferred().cloned()),
			list: self
				.value
				.list
				.causes()
				.map(|causes| causes.preferred().cloned()),
		}
	}

	pub fn is_type(&self) -> bool {
		self.value.ty.is_set()
	}

	pub fn is_property(&self) -> bool {
		self.value.property.is_set()
	}

	pub fn is_layout(&self) -> bool {
		self.value.layout.is_set()
	}

	pub fn is_layout_field(&self) -> bool {
		self.value.layout_field.is_set()
	}

	pub fn is_layout_variant(&self) -> bool {
		self.value.layout_variant.is_set()
	}

	pub fn is_list(&self) -> bool {
		self.value.list.is_set()
	}

	pub fn as_type(&self) -> Option<&WithCauses<ty::Definition<F, D::Type>, F>> {
		self.value.ty.with_causes()
	}

	pub fn as_property(&self) -> Option<&WithCauses<prop::Definition<F>, F>> {
		self.value.property.with_causes()
	}

	pub fn as_layout(&self) -> Option<&WithCauses<layout::Definition<F, D::Layout>, F>> {
		self.value.layout.with_causes()
	}

	pub fn as_layout_field(&self) -> Option<&WithCauses<layout::field::Definition<F>, F>> {
		self.value.layout_field.with_causes()
	}

	pub fn as_layout_variant(&self) -> Option<&WithCauses<layout::variant::Definition<F>, F>> {
		self.value.layout_variant.with_causes()
	}

	pub fn as_list(&self) -> Option<&WithCauses<list::Definition<F>, F>> {
		self.value.list.with_causes()
	}

	pub fn as_type_mut(&mut self) -> Option<&mut WithCauses<ty::Definition<F, D::Type>, F>> {
		self.value.ty.with_causes_mut()
	}

	pub fn as_property_mut(&mut self) -> Option<&mut WithCauses<prop::Definition<F>, F>> {
		self.value.property.with_causes_mut()
	}

	pub fn as_layout_mut(
		&mut self,
	) -> Option<&mut WithCauses<layout::Definition<F, D::Layout>, F>> {
		self.value.layout.with_causes_mut()
	}

	pub fn as_layout_field_mut(
		&mut self,
	) -> Option<&mut WithCauses<layout::field::Definition<F>, F>> {
		self.value.layout_field.with_causes_mut()
	}

	pub fn as_layout_variant_mut(
		&mut self,
	) -> Option<&mut WithCauses<layout::variant::Definition<F>, F>> {
		self.value.layout_variant.with_causes_mut()
	}

	pub fn as_list_mut(&mut self) -> Option<&mut WithCauses<list::Definition<F>, F>> {
		self.value.list.with_causes_mut()
	}

	pub fn declare_type(&mut self, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		self.value
			.ty
			.set_once(cause, || ty::Definition::new(self.id))
	}

	pub fn declare_property(&mut self, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		self.value
			.property
			.set_once(cause, || prop::Definition::new(self.id))
	}

	pub fn declare_layout(&mut self, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		self.value
			.layout
			.set_once(cause, || layout::Definition::new(self.id))
	}

	pub fn declare_layout_field(&mut self, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		self.value
			.layout_field
			.set_once(cause, || layout::field::Definition::new(self.id))
	}

	pub fn declare_layout_variant(&mut self, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		self.value
			.layout_variant
			.set_once(cause, || layout::variant::Definition::new(self.id))
	}

	pub fn declare_list(&mut self, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		self.value
			.list
			.set_once(cause, || list::Definition::new(self.id))
	}

	#[allow(clippy::type_complexity)]
	pub fn require_type_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<ty::Definition<F, D::Type>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.ty.with_causes_mut() {
			Some(ty) => Ok(ty),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Type,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_property_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<prop::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.property.with_causes_mut() {
			Some(prop) => Ok(prop),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Property,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_layout(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::Definition<F, D::Layout>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.layout.with_causes() {
			Some(layout) => Ok(layout),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Layout,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_layout_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<layout::Definition<F, D::Layout>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.layout.with_causes_mut() {
			Some(layout) => Ok(layout),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Layout,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_field(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::field::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_field.with_causes() {
			Some(field) => Ok(field),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutField,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_field_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<layout::field::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_field.with_causes_mut() {
			Some(field) => Ok(field),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutField,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_variant(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<layout::variant::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_variant.with_causes() {
			Some(variant) => Ok(variant),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutVariant,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_layout_variant_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut WithCauses<layout::variant::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_variant.with_causes_mut() {
			Some(variant) => Ok(variant),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutVariant,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_property_or_layout_field_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<PropertyOrLayoutField<F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();

		let (prop, layout_field) = (
			self.value.property.with_causes_mut(),
			self.value.layout_field.with_causes_mut(),
		);

		if prop.is_some() || layout_field.is_some() {
			Ok((prop, layout_field))
		} else {
			Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Property,
					found: types,
				}
				.into(),
				cause,
			))
		}
	}

	pub fn require_layout_field_or_variant_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<LayoutFieldOrVariant<F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();

		let (layout_field, layout_variant) = (
			self.value.layout_field.with_causes_mut(),
			self.value.layout_variant.with_causes_mut(),
		);

		if layout_field.is_some() || layout_variant.is_some() {
			Ok((layout_field, layout_variant))
		} else {
			Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutField,
					found: types,
				}
				.into(),
				cause,
			))
		}
	}

	pub fn require_list(
		&self,
		cause: Option<Location<F>>,
	) -> Result<&WithCauses<list::Definition<F>, F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.list.with_causes() {
			Some(list) => Ok(list),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::List,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn require_list_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut list::Definition<F>, Error<F>>
	where
		F: Clone,
	{
		let types = self.caused_types();
		match self.value.list.with_causes_mut() {
			Some(list) => Ok(list),
			None => Err(Caused::new(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::List,
					found: types,
				}
				.into(),
				cause,
			)),
		}
	}
}
