use super::{layout, prop, ty, list, Error};
use crate::{Causes, Caused, WithCauses, MaybeSet, Id, Documentation};
use locspan::Location;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Type,
	Property,
	Layout,
	LayoutField,
	List
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Types {
	pub ty: bool,
	pub property: bool,
	pub layout: bool,
	pub layout_field: bool,
	pub list: bool
}

impl Types {
	pub fn includes(&self, ty: Type) -> bool {
		match ty {
			Type::Type => self.ty,
			Type::Property => self.property,
			Type::Layout => self.layout,
			Type::LayoutField => self.layout_field,
			Type::List => self.list
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
			Type::List => self.list.as_ref()
		}
	}

	pub fn iter(&self) -> CausedTypesIter<F> {
		CausedTypesIter {
			ty: self.ty.as_ref(),
			property: self.property.as_ref(),
			layout: self.layout.as_ref(),
			layout_field: self.layout_field.as_ref(),
			list: self.list.as_ref()
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
					}
				}
			},
		}
	}
}

pub struct Node<T> {
	id: Id,
	doc: Documentation,
	value: T
}

pub struct Components<F> {
	pub ty: MaybeSet<ty::Definition<F>, F>,
	pub property: MaybeSet<prop::Definition<F>, F>,
	pub layout: MaybeSet<layout::Definition<F>, F>,
	pub layout_field: MaybeSet<layout::field::Definition<F>, F>,
	pub list: MaybeSet<list::Definition<F>, F>,
}

impl<T> Node<T> {
	pub fn id(&self) -> Id {
		self.id
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
	
	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Node<U> {
		Node {
			id: self.id,
			doc: self.doc,
			value: f(self.value)
		}
	}
}

impl<F> Node<Components<F>> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				list: MaybeSet::default()
			}
		}
	}

	pub fn new_type(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::new(ty::Definition::new(), causes),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				list: MaybeSet::default(),
			}
		}
	}

	pub fn new_property(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::new(prop::Definition::new(), causes),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				list: MaybeSet::default(),
			}
		}
	}

	pub fn new_layout(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::new(layout::Definition::new(), causes),
				layout_field: MaybeSet::default(),
				list: MaybeSet::default(),
			}
		}
	}

	pub fn new_layout_field(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::new(layout::field::Definition::new(), causes),
				list: MaybeSet::default(),
			}
		}
	}

	pub fn new_list(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			doc: Documentation::default(),
			value: Components {
				ty: MaybeSet::default(),
				property: MaybeSet::default(),
				layout: MaybeSet::default(),
				layout_field: MaybeSet::default(),
				list: MaybeSet::new(list::Definition::new(), causes),
			}
		}
	}

	pub fn types(&self) -> Types {
		Types {
			ty: self.value.ty.is_set(),
			property: self.value.property.is_set(),
			layout: self.value.layout.is_set(),
			layout_field: self.value.layout_field.is_set(),
			list: self.value.list.is_set(),
		}
	}

	pub fn caused_types(&self) -> CausedTypes<F> where F: Clone {
		CausedTypes {
			ty: self.value.ty.causes().map(|causes| causes.preferred().cloned()),
			property: self.value.property.causes().map(|causes| causes.preferred().cloned()),
			layout: self.value.layout.causes().map(|causes| causes.preferred().cloned()),
			layout_field: self.value.layout_field.causes().map(|causes| causes.preferred().cloned()),
			list: self.value.list.causes().map(|causes| causes.preferred().cloned()),
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

	pub fn is_list(&self) -> bool {
		self.value.list.is_set()
	}

	pub fn as_type(&self) -> Option<&WithCauses<ty::Definition<F>, F>> {
		self.value.ty.with_causes()
	}

	pub fn as_property(&self) -> Option<&WithCauses<prop::Definition<F>, F>> {
		self.value.property.with_causes()
	}

	pub fn as_layout(&self) -> Option<&WithCauses<layout::Definition<F>, F>> {
		self.value.layout.with_causes()
	}

	pub fn as_layout_field(&self) -> Option<&WithCauses<layout::field::Definition<F>, F>> {
		self.value.layout_field.with_causes()
	}

	pub fn as_list(&self) -> Option<&WithCauses<list::Definition<F>, F>> {
		self.value.list.with_causes()
	}

	pub fn as_type_mut(&mut self) -> Option<&mut WithCauses<ty::Definition<F>, F>> {
		self.value.ty.with_causes_mut()
	}

	pub fn as_property_mut(&mut self) -> Option<&mut WithCauses<prop::Definition<F>, F>> {
		self.value.property.with_causes_mut()
	}

	pub fn as_layout_mut(&mut self) -> Option<&mut WithCauses<layout::Definition<F>, F>> {
		self.value.layout.with_causes_mut()
	}

	pub fn as_layout_field_mut(&mut self) -> Option<&mut WithCauses<layout::field::Definition<F>, F>> {
		self.value.layout_field.with_causes_mut()
	}

	pub fn as_list_mut(&mut self) -> Option<&mut WithCauses<list::Definition<F>, F>> {
		self.value.list.with_causes_mut()
	}

	pub fn declare_type(&mut self, cause: Option<Location<F>>) where F: Ord {
		self.value.ty.set_once(cause, || ty::Definition::new())
	}

	pub fn declare_property(&mut self, cause: Option<Location<F>>) where F: Ord {
		self.value.property.set_once(cause, || prop::Definition::new())
	}

	pub fn declare_layout(&mut self, cause: Option<Location<F>>) where F: Ord {
		self.value.layout.set_once(cause, || layout::Definition::new())
	}

	pub fn declare_layout_field(&mut self, cause: Option<Location<F>>) where F: Ord {
		self.value.layout_field.set_once(cause, || layout::field::Definition::new())
	}

	pub fn declare_list(&mut self, cause: Option<Location<F>>) where F: Ord {
		self.value.list.set_once(cause, || list::Definition::new())
	}

	pub fn require_type_mut(&mut self, cause: Option<Location<F>>) -> Result<&mut WithCauses<ty::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		let types = self.caused_types();
		match self.value.ty.with_causes_mut() {
			Some(ty) => Ok(ty),
			None => {
				Err(Caused::new(
					Error::InvalidNodeType {
						id: self.id,
						expected: Type::Type,
						found: types
					},
					cause
				))
			}
		}
	}

	pub fn require_property_mut(&mut self, cause: Option<Location<F>>) -> Result<&mut WithCauses<prop::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		let types = self.caused_types();
		match self.value.property.with_causes_mut() {
			Some(prop) => Ok(prop),
			None => {
				Err(Caused::new(
					Error::InvalidNodeType {
						id: self.id,
						expected: Type::Property,
						found: types
					},
					cause
				))
			}
		}
	}

	pub fn require_layout_mut(&mut self, cause: Option<Location<F>>) -> Result<&mut WithCauses<layout::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		let types = self.caused_types();
		match self.value.layout.with_causes_mut() {
			Some(layout) => Ok(layout),
			None => {
				Err(Caused::new(
					Error::InvalidNodeType {
						id: self.id,
						expected: Type::Layout,
						found: types
					},
					cause
				))
			}
		}
	}

	pub fn require_layout_field_mut(&mut self, cause: Option<Location<F>>) -> Result<&mut WithCauses<layout::field::Definition<F>, F>, Caused<Error<F>, F>> where F: Clone {
		let types = self.caused_types();
		match self.value.layout_field.with_causes_mut() {
			Some(field) => Ok(field),
			None => {
				Err(Caused::new(
					Error::InvalidNodeType {
						id: self.id,
						expected: Type::LayoutField,
						found: types
					},
					cause
				))
			}
		}
	}

	pub fn require_property_or_layout_field_mut(&mut self, cause: Option<Location<F>>) -> Result<(Option<&mut WithCauses<prop::Definition<F>, F>>, Option<&mut WithCauses<layout::field::Definition<F>, F>>), Caused<Error<F>, F>> where F: Clone {
		let types = self.caused_types();

		let (prop, layout_field) = (self.value.property.with_causes_mut(), self.value.layout_field.with_causes_mut());

		if prop.is_some() || layout_field.is_some() {
			Ok((prop, layout_field))
		} else {
			Err(Caused::new(
				Error::InvalidNodeType {
					id: self.id,
					expected: Type::Property,
					found: types
				},
				cause
			))
		}
	}

	pub fn require_list_mut(&mut self, cause: Option<Location<F>>) -> Result<&mut list::Definition<F>, Caused<Error<F>, F>> where F: Clone {
		let types = self.caused_types();
		match self.value.list.with_causes_mut() {
			Some(list) => Ok(list),
			None => {
				Err(Caused::new(
					Error::InvalidNodeType {
						id: self.id,
						expected: Type::List,
						found: types
					},
					cause
				))
			}
		}
	}
}