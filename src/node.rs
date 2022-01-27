use crate::{layout, prop, ty, Cause, Caused, Id, Ref};

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
pub struct CausedTypes {
	pub ty: Option<Option<Cause>>,
	pub property: Option<Option<Cause>>,
	pub layout: Option<Option<Cause>>,
}

impl CausedTypes {
	pub fn is_empty(&self) -> bool {
		self.ty.is_none() && self.property.is_none() && self.layout.is_none()
	}

	pub fn includes(&self, ty: Type) -> Option<Option<Cause>> {
		match ty {
			Type::Type => self.ty,
			Type::Property => self.property,
			Type::Layout => self.layout,
		}
	}

	pub fn iter(&self) -> CausedTypesIter {
		CausedTypesIter {
			ty: self.ty,
			property: self.property,
			layout: self.layout,
		}
	}
}

impl IntoIterator for CausedTypes {
	type Item = Caused<Type>;
	type IntoIter = CausedTypesIter;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a CausedTypes {
	type Item = Caused<Type>;
	type IntoIter = CausedTypesIter;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct CausedTypesIter {
	ty: Option<Option<Cause>>,
	property: Option<Option<Cause>>,
	layout: Option<Option<Cause>>,
}

impl Iterator for CausedTypesIter {
	type Item = Caused<Type>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.ty.take() {
			Some(cause) => Some(Caused::new(Type::Type, cause)),
			None => match self.property.take() {
				Some(cause) => Some(Caused::new(Type::Property, cause)),
				None => self
					.layout
					.take()
					.map(|cause| Caused::new(Type::Property, cause)),
			},
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Node {
	id: Id,
	ty: Option<Ref<ty::Definition>>,
	property: Option<Ref<prop::Definition>>,
	layout: Option<Ref<layout::Definition>>,
}

impl Node {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			ty: None,
			property: None,
			layout: None,
		}
	}

	pub fn new_type(id: Id, ty: Ref<ty::Definition>) -> Self {
		Self {
			id,
			ty: Some(ty),
			property: None,
			layout: None,
		}
	}

	pub fn new_property(id: Id, prop: Ref<prop::Definition>) -> Self {
		Self {
			id,
			ty: None,
			property: Some(prop),
			layout: None,
		}
	}

	pub fn new_layout(id: Id, layout: Ref<layout::Definition>) -> Self {
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

	pub fn caused_types(&self, model: &crate::Model) -> CausedTypes {
		CausedTypes {
			ty: self
				.ty
				.map(|ty_ref| model.types().get(ty_ref).unwrap().causes().preferred()),
			property: self.property.map(|prop_ref| {
				model
					.properties()
					.get(prop_ref)
					.unwrap()
					.causes()
					.preferred()
			}),
			layout: self.layout.map(|layout_ref| {
				model
					.layouts()
					.get(layout_ref)
					.unwrap()
					.causes()
					.preferred()
			}),
		}
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn as_type(&self) -> Option<Ref<ty::Definition>> {
		self.ty
	}

	pub fn as_property(&self) -> Option<Ref<prop::Definition>> {
		self.property
	}

	pub fn as_layout(&self) -> Option<Ref<layout::Definition>> {
		self.layout
	}

	pub fn declare_type(&mut self, ty_ref: Ref<ty::Definition>) {
		self.ty = Some(ty_ref)
	}

	pub fn declare_property(&mut self, prop_ref: Ref<prop::Definition>) {
		self.property = Some(prop_ref)
	}

	pub fn declare_layout(&mut self, layout_ref: Ref<layout::Definition>) {
		self.layout = Some(layout_ref)
	}
}
