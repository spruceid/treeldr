use crate::{error, layout, list, prop, ty, Descriptions, Error, TryMap};
use derivative::Derivative;
use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{BlankIdIndex, Documentation, Id, IriIndex, MetaOption};

pub use treeldr::node::{Type, Types, TypesMetadata};

#[derive(Clone)]
pub struct Node<T> {
	id: Id,
	label: Option<String>,
	doc: Documentation,
	value: T,
}

#[derive(Derivative)]
#[derivative(Clone(bound = "M: Clone"))]
pub struct Components<M, D: Descriptions<M> = crate::StandardDescriptions> {
	pub ty: MetaOption<ty::Definition<M, D::Type>, M>,
	pub property: MetaOption<prop::Definition<M>, M>,
	pub layout: MetaOption<layout::Definition<M, D::Layout>, M>,
	pub layout_field: MetaOption<layout::field::Definition<M>, M>,
	pub layout_variant: MetaOption<layout::variant::Definition<M>, M>,
	pub list: MetaOption<list::Definition<M>, M>,
}

impl<M, D: Descriptions<M>> Components<M, D> {
	pub fn try_map<
		G: Descriptions<M>,
		E,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		self,
		map: &impl TryMap<M, E, D, G>,
		source: &crate::Context<M, D>,
		target: &mut crate::Context<M, G>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Components<M, G>, E>
	where
		M: Clone,
	{
		Ok(Components {
			ty: self.ty.try_map_with_causes(|Meta(d, metadata)| {
				Ok(Meta(
					d.try_map(|d| map.ty(d, &metadata, source, target, vocabulary, generator))?,
					metadata,
				))
			})?,
			property: self.property,
			layout: self.layout.try_map_with_causes(|Meta(d, metadata)| {
				Ok(Meta(
					d.try_map(|d| map.layout(d, &metadata, source, target, vocabulary, generator))?,
					metadata,
				))
			})?,
			layout_field: self.layout_field,
			layout_variant: self.layout_variant,
			list: self.list,
		})
	}
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

	pub fn try_map<U, E>(self, f: impl FnOnce(T) -> Result<U, E>) -> Result<Node<U>, E> {
		Ok(Node {
			id: self.id,
			label: self.label,
			doc: self.doc,
			value: f(self.value)?,
		})
	}

	pub fn into_parts(self) -> (Id, Option<String>, Documentation, T) {
		(self.id, self.label, self.doc, self.value)
	}
}

pub type PropertyOrLayoutField<'a, M> = (
	Option<&'a mut Meta<prop::Definition<M>, M>>,
	Option<&'a mut Meta<layout::field::Definition<M>, M>>,
);

pub type LayoutFieldOrVariant<'a, M> = (
	Option<&'a mut Meta<layout::field::Definition<M>, M>>,
	Option<&'a mut Meta<layout::variant::Definition<M>, M>>,
);

impl<M, D: Descriptions<M>> Node<Components<M, D>> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MetaOption::default(),
				property: MetaOption::default(),
				layout: MetaOption::default(),
				layout_field: MetaOption::default(),
				layout_variant: MetaOption::default(),
				list: MetaOption::default(),
			},
		}
	}

	pub fn new_type(id: Id, metadata: M) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MetaOption::new(ty::Definition::new(id), metadata),
				property: MetaOption::default(),
				layout: MetaOption::default(),
				layout_field: MetaOption::default(),
				layout_variant: MetaOption::default(),
				list: MetaOption::default(),
			},
		}
	}

	pub fn new_property(id: Id, metadata: M) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MetaOption::default(),
				property: MetaOption::new(prop::Definition::new(id), metadata),
				layout: MetaOption::default(),
				layout_field: MetaOption::default(),
				layout_variant: MetaOption::default(),
				list: MetaOption::default(),
			},
		}
	}

	pub fn new_layout(id: Id, metadata: M) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MetaOption::default(),
				property: MetaOption::default(),
				layout: MetaOption::new(layout::Definition::new(id), metadata),
				layout_field: MetaOption::default(),
				layout_variant: MetaOption::default(),
				list: MetaOption::default(),
			},
		}
	}

	pub fn new_layout_field(id: Id, metadata: M) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MetaOption::default(),
				property: MetaOption::default(),
				layout: MetaOption::default(),
				layout_field: MetaOption::new(layout::field::Definition::new(id), metadata),
				layout_variant: MetaOption::default(),
				list: MetaOption::default(),
			},
		}
	}

	pub fn new_layout_variant(id: Id, metadata: M) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MetaOption::default(),
				property: MetaOption::default(),
				layout: MetaOption::default(),
				layout_field: MetaOption::default(),
				layout_variant: MetaOption::new(layout::variant::Definition::new(id), metadata),
				list: MetaOption::default(),
			},
		}
	}

	pub fn new_list(id: Id, metadata: M) -> Self {
		Self {
			id,
			label: None,
			doc: Documentation::default(),
			value: Components {
				ty: MetaOption::default(),
				property: MetaOption::default(),
				layout: MetaOption::default(),
				layout_field: MetaOption::default(),
				layout_variant: MetaOption::default(),
				list: MetaOption::new(list::Definition::new(id), metadata),
			},
		}
	}

	pub fn types(&self) -> Types {
		Types {
			ty: self.value.ty.is_some(),
			property: self.value.property.is_some(),
			layout: self.value.layout.is_some(),
			layout_field: self.value.layout_field.is_some(),
			layout_variant: self.value.layout_variant.is_some(),
			list: self.value.list.is_some(),
		}
	}

	pub fn caused_types(&self) -> TypesMetadata<M>
	where
		M: Clone,
	{
		TypesMetadata {
			ty: self.value.ty.metadata().cloned(),
			property: self.value.property.metadata().cloned(),
			layout: self.value.layout.metadata().cloned(),
			layout_field: self.value.layout_field.metadata().cloned(),
			layout_variant: self.value.layout_variant.metadata().cloned(),
			list: self.value.list.metadata().cloned(),
		}
	}

	pub fn is_type(&self) -> bool {
		self.value.ty.is_some()
	}

	pub fn is_property(&self) -> bool {
		self.value.property.is_some()
	}

	pub fn is_layout(&self) -> bool {
		self.value.layout.is_some()
	}

	pub fn is_layout_field(&self) -> bool {
		self.value.layout_field.is_some()
	}

	pub fn is_layout_variant(&self) -> bool {
		self.value.layout_variant.is_some()
	}

	pub fn is_list(&self) -> bool {
		self.value.list.is_some()
	}

	pub fn as_type(&self) -> Option<&Meta<ty::Definition<M, D::Type>, M>> {
		self.value.ty.as_ref()
	}

	pub fn as_property(&self) -> Option<&Meta<prop::Definition<M>, M>> {
		self.value.property.as_ref()
	}

	pub fn as_layout(&self) -> Option<&Meta<layout::Definition<M, D::Layout>, M>> {
		self.value.layout.as_ref()
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::field::Definition<M>, M>> {
		self.value.layout_field.as_ref()
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::variant::Definition<M>, M>> {
		self.value.layout_variant.as_ref()
	}

	pub fn as_list(&self) -> Option<&Meta<list::Definition<M>, M>> {
		self.value.list.as_ref()
	}

	pub fn as_type_mut(&mut self) -> Option<&mut Meta<ty::Definition<M, D::Type>, M>> {
		self.value.ty.as_mut()
	}

	pub fn as_property_mut(&mut self) -> Option<&mut Meta<prop::Definition<M>, M>> {
		self.value.property.as_mut()
	}

	pub fn as_layout_mut(&mut self) -> Option<&mut Meta<layout::Definition<M, D::Layout>, M>> {
		self.value.layout.as_mut()
	}

	pub fn as_layout_field_mut(&mut self) -> Option<&mut Meta<layout::field::Definition<M>, M>> {
		self.value.layout_field.as_mut()
	}

	pub fn as_layout_variant_mut(
		&mut self,
	) -> Option<&mut Meta<layout::variant::Definition<M>, M>> {
		self.value.layout_variant.as_mut()
	}

	pub fn as_list_mut(&mut self) -> Option<&mut Meta<list::Definition<M>, M>> {
		self.value.list.as_mut()
	}

	pub fn declare_type(&mut self, cause: M) {
		self.value
			.ty
			.set_once(cause, || ty::Definition::new(self.id))
	}

	pub fn declare_property(&mut self, cause: M) {
		self.value
			.property
			.set_once(cause, || prop::Definition::new(self.id))
	}

	pub fn declare_layout(&mut self, cause: M) {
		self.value
			.layout
			.set_once(cause, || layout::Definition::new(self.id))
	}

	pub fn declare_layout_field(&mut self, cause: M) {
		self.value
			.layout_field
			.set_once(cause, || layout::field::Definition::new(self.id))
	}

	pub fn declare_layout_variant(&mut self, cause: M) {
		self.value
			.layout_variant
			.set_once(cause, || layout::variant::Definition::new(self.id))
	}

	pub fn declare_list(&mut self, cause: M) {
		self.value
			.list
			.set_once(cause, || list::Definition::new(self.id))
	}

	#[allow(clippy::type_complexity)]
	pub fn require_type(&self, cause: &M) -> Result<&Meta<ty::Definition<M, D::Type>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.ty.as_ref() {
			Some(ty) => Ok(ty),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Type,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_type_mut(
		&mut self,
		cause: &M,
	) -> Result<&mut Meta<ty::Definition<M, D::Type>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.ty.as_mut() {
			Some(ty) => Ok(ty),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Type,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_property_mut(
		&mut self,
		cause: &M,
	) -> Result<&mut Meta<prop::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.property.as_mut() {
			Some(prop) => Ok(prop),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Property,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_layout(
		&self,
		cause: &M,
	) -> Result<&Meta<layout::Definition<M, D::Layout>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.layout.as_ref() {
			Some(layout) => Ok(layout),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Layout,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	#[allow(clippy::type_complexity)]
	pub fn require_layout_mut(
		&mut self,
		cause: &M,
	) -> Result<&mut Meta<layout::Definition<M, D::Layout>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.layout.as_mut() {
			Some(layout) => Ok(layout),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Layout,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_field(
		&self,
		cause: &M,
	) -> Result<&Meta<layout::field::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_field.as_ref() {
			Some(field) => Ok(field),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutField,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_field_mut(
		&mut self,
		cause: &M,
	) -> Result<&mut Meta<layout::field::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_field.as_mut() {
			Some(field) => Ok(field),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutField,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_variant(
		&self,
		cause: &M,
	) -> Result<&Meta<layout::variant::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_variant.as_ref() {
			Some(variant) => Ok(variant),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutVariant,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_layout_variant_mut(
		&mut self,
		cause: &M,
	) -> Result<&mut Meta<layout::variant::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.layout_variant.as_mut() {
			Some(variant) => Ok(variant),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutVariant,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_property_or_layout_field_mut(
		&mut self,
		cause: &M,
	) -> Result<PropertyOrLayoutField<M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();

		let (prop, layout_field) = (
			self.value.property.as_mut(),
			self.value.layout_field.as_mut(),
		);

		if prop.is_some() || layout_field.is_some() {
			Ok((prop, layout_field))
		} else {
			Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::Property,
					found: types,
				}
				.into(),
				cause.clone(),
			))
		}
	}

	pub fn require_layout_field_or_variant_mut(
		&mut self,
		cause: &M,
	) -> Result<LayoutFieldOrVariant<M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();

		let (layout_field, layout_variant) = (
			self.value.layout_field.as_mut(),
			self.value.layout_variant.as_mut(),
		);

		if layout_field.is_some() || layout_variant.is_some() {
			Ok((layout_field, layout_variant))
		} else {
			Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::LayoutField,
					found: types,
				}
				.into(),
				cause.clone(),
			))
		}
	}

	pub fn require_list(&self, cause: &M) -> Result<&Meta<list::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.list.as_ref() {
			Some(list) => Ok(list),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::List,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_list_mut(&mut self, cause: &M) -> Result<&mut list::Definition<M>, Error<M>>
	where
		M: Clone,
	{
		let types = self.caused_types();
		match self.value.list.as_mut() {
			Some(list) => Ok(list),
			None => Err(Meta(
				error::NodeInvalidType {
					id: self.id,
					expected: Type::List,
					found: types,
				}
				.into(),
				cause.clone(),
			)),
		}
	}
}
