use crate::{layout, prop, ty, error, Caused, Documentation, Id, MaybeSet};
use locspan::Location;
use shelves::Ref;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Type,
	Property,
	Layout,
	LayoutField,
	List,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Types {
	pub ty: bool,
	pub property: bool,
	pub layout: bool,
	pub layout_field: bool,
	pub list: bool,
}

impl Types {
	pub fn includes(&self, ty: Type) -> bool {
		match ty {
			Type::Type => self.ty,
			Type::Property => self.property,
			Type::Layout => self.layout,
			Type::LayoutField => self.layout_field,
			Type::List => self.list,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct CausedTypes<F> {
	pub ty: Option<Option<Location<F>>>,
	pub property: Option<Option<Location<F>>>,
	pub layout: Option<Option<Location<F>>>,
	pub layout_field: Option<Option<Location<F>>>,
	pub list: Option<Option<Location<F>>>,
}

impl<F> CausedTypes<F> {
	pub fn is_empty(&self) -> bool {
		self.ty.is_none() && self.property.is_none() && self.layout.is_none()
	}

	pub fn includes(&self, ty: Type) -> Option<&Option<Location<F>>> {
		match ty {
			Type::Type => self.ty.as_ref(),
			Type::Property => self.property.as_ref(),
			Type::Layout => self.layout.as_ref(),
			Type::LayoutField => self.layout_field.as_ref(),
			Type::List => self.list.as_ref(),
		}
	}

	pub fn iter(&self) -> CausedTypesIter<F> {
		CausedTypesIter {
			ty: self.ty.as_ref(),
			property: self.property.as_ref(),
			layout: self.layout.as_ref(),
			layout_field: self.layout_field.as_ref(),
			list: self.list.as_ref(),
		}
	}
}

impl<'a, F: Clone> IntoIterator for &'a CausedTypes<F> {
	type Item = Caused<Type, F>;
	type IntoIter = CausedTypesIter<'a, F>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct CausedTypesIter<'a, F> {
	ty: Option<&'a Option<Location<F>>>,
	property: Option<&'a Option<Location<F>>>,
	layout: Option<&'a Option<Location<F>>>,
	layout_field: Option<&'a Option<Location<F>>>,
	list: Option<&'a Option<Location<F>>>,
}

impl<'a, F: Clone> Iterator for CausedTypesIter<'a, F> {
	type Item = Caused<Type, F>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.ty.take() {
			Some(cause) => Some(Caused::new(Type::Type, cause.clone())),
			None => match self.property.take() {
				Some(cause) => Some(Caused::new(Type::Property, cause.clone())),
				None => match self.layout.take() {
					Some(cause) => Some(Caused::new(Type::Layout, cause.clone())),
					None => match self.layout_field.take() {
						Some(cause) => Some(Caused::new(Type::LayoutField, cause.clone())),
						None => self
							.list
							.take()
							.map(|cause| Caused::new(Type::List, cause.clone())),
					},
				},
			},
		}
	}
}

#[derive(Debug)]
pub struct Node<F> {
	id: Id,
	ty: MaybeSet<Ref<ty::Definition<F>>, F>,
	property: MaybeSet<Ref<prop::Definition<F>>, F>,
	layout: MaybeSet<Ref<layout::Definition<F>>, F>,
	doc: Documentation,
}

impl<F> Node<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			ty: MaybeSet::default(),
			property: MaybeSet::default(),
			layout: MaybeSet::default(),
			doc: Documentation::default(),
		}
	}

	pub fn from_parts(
		id: Id,
		ty: MaybeSet<Ref<ty::Definition<F>>, F>,
		property: MaybeSet<Ref<prop::Definition<F>>, F>,
		layout: MaybeSet<Ref<layout::Definition<F>>, F>,
		doc: Documentation,
	) -> Self {
		Self {
			id,
			ty,
			property,
			layout,
			doc,
		}
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn documentation_mut(&mut self) -> &mut Documentation {
		&mut self.doc
	}

	pub fn is_type(&self) -> bool {
		self.ty.is_set()
	}

	pub fn is_property(&self) -> bool {
		self.property.is_set()
	}

	pub fn is_layout(&self) -> bool {
		self.layout.is_set()
	}

	pub fn as_type(&self) -> Option<Ref<ty::Definition<F>>> {
		self.ty.value().cloned()
	}

	pub fn as_property(&self) -> Option<Ref<prop::Definition<F>>> {
		self.property.value().cloned()
	}

	pub fn as_layout(&self) -> Option<Ref<layout::Definition<F>>> {
		self.layout.value().cloned()
	}

	pub fn caused_types(&self) -> CausedTypes<F>
	where
		F: Clone,
	{
		CausedTypes {
			ty: self
				.ty
				.causes()
				.map(|causes| causes.preferred().cloned()),
			property: self
				.property
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout: self
				.layout
				.causes()
				.map(|causes| causes.preferred().cloned()),
			layout_field: None,
			list: None
		}
	}

	pub fn require_layout(&self) -> Result<Ref<layout::Definition<F>>, error::Description<F>> where F: Clone {
		self.as_layout().ok_or_else(|| error::NodeInvalidType {
			id: self.id,
			expected: Type::Layout,
			found: self.caused_types(),
		}.into())
	}
}
