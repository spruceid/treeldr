use crate::{layout, prop, ty, Caused, Id, Ref};
use locspan::Location;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Type,
	Property,
	Layout,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Types {
	pub ty: bool,
	pub property: bool,
	pub layout: bool,
}

impl Types {
	pub fn includes(&self, ty: Type) -> bool {
		match ty {
			Type::Type => self.ty,
			Type::Property => self.property,
			Type::Layout => self.layout,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct CausedTypes<F> {
	pub ty: Option<Option<Location<F>>>,
	pub property: Option<Option<Location<F>>>,
	pub layout: Option<Option<Location<F>>>,
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
		}
	}

	pub fn iter(&self) -> CausedTypesIter<F> {
		CausedTypesIter {
			ty: self.ty.as_ref(),
			property: self.property.as_ref(),
			layout: self.layout.as_ref(),
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
}

impl<'a, F: Clone> Iterator for CausedTypesIter<'a, F> {
	type Item = Caused<Type, F>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.ty.take() {
			Some(cause) => Some(Caused::new(Type::Type, cause.clone())),
			None => match self.property.take() {
				Some(cause) => Some(Caused::new(Type::Property, cause.clone())),
				None => self
					.layout
					.take()
					.map(|cause| Caused::new(Type::Property, cause.clone())),
			},
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Node<F> {
	id: Id,
	ty: Option<Ref<ty::Definition<F>>>,
	property: Option<Ref<prop::Definition<F>>>,
	layout: Option<Ref<layout::Definition<F>>>,
}

impl<F> Node<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			ty: None,
			property: None,
			layout: None,
		}
	}

	pub fn new_type(id: Id, ty: Ref<ty::Definition<F>>) -> Self {
		Self {
			id,
			ty: Some(ty),
			property: None,
			layout: None,
		}
	}

	pub fn new_property(id: Id, prop: Ref<prop::Definition<F>>) -> Self {
		Self {
			id,
			ty: None,
			property: Some(prop),
			layout: None,
		}
	}

	pub fn new_layout(id: Id, layout: Ref<layout::Definition<F>>) -> Self {
		Self {
			id,
			ty: None,
			property: None,
			layout: Some(layout),
		}
	}

	pub fn types(&self) -> Types {
		Types {
			ty: self.ty.is_some(),
			property: self.property.is_some(),
			layout: self.layout.is_some(),
		}
	}

	pub fn caused_types(&self, model: &crate::Model<F>) -> CausedTypes<F> where F: Clone {
		CausedTypes {
			ty: self
				.ty
				.map(|ty_ref| model.types().get(ty_ref).unwrap().causes().preferred().cloned()),
			property: self.property.map(|prop_ref| {
				model
					.properties()
					.get(prop_ref)
					.unwrap()
					.causes()
					.preferred().cloned()
			}),
			layout: self.layout.map(|layout_ref| {
				model
					.layouts()
					.get(layout_ref)
					.unwrap()
					.causes()
					.preferred().cloned()
			}),
		}
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn as_type(&self) -> Option<Ref<ty::Definition<F>>> {
		self.ty
	}

	pub fn as_property(&self) -> Option<Ref<prop::Definition<F>>> {
		self.property
	}

	pub fn as_layout(&self) -> Option<Ref<layout::Definition<F>>> {
		self.layout
	}

	pub fn declare_type(&mut self, ty_ref: Ref<ty::Definition<F>>) {
		self.ty = Some(ty_ref)
	}

	pub fn declare_property(&mut self, prop_ref: Ref<prop::Definition<F>>) {
		self.property = Some(prop_ref)
	}

	pub fn declare_layout(&mut self, layout_ref: Ref<layout::Definition<F>>) {
		self.layout = Some(layout_ref)
	}
}
