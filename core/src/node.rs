use crate::{error, layout, prop, ty, Error, Id, MetaOption, component, Multiple, Type, ResourceType, vocab};
use locspan::Meta;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Types {
	pub ty: bool,
	pub datatype_restriction: bool,
	pub property: bool,
	pub component: component::Types,
	pub layout_restriction: bool,
	pub list: bool,
}

impl Types {
	pub fn includes(&self, ty: Type) -> bool {
		match ty {
			Type::Resource => true,
			Type::Type => self.ty,
			Type::DatatypeRestriction => self.datatype_restriction,
			Type::Property => self.property,
			Type::Component(t) => self.component.includes(t),
			Type::LayoutRestriction => self.layout_restriction,
			Type::List => self.list,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct TypesMetadata<M> {
	pub ty: Option<M>,
	pub datatype_restriction: Option<M>,
	pub property: Option<M>,
	pub component: component::TypesMetadata<M>,
	pub layout_restriction: Option<M>,
	pub list: Option<M>,
}

impl<M> TypesMetadata<M> {
	pub fn is_empty(&self) -> bool {
		self.ty.is_none() && self.datatype_restriction.is_none() && self.property.is_none() && self.component.is_empty() && self.layout_restriction.is_none() && self.list.is_none()
	}

	pub fn includes(&self, ty: Type) -> Option<&M> {
		match ty {
			Type::Resource => None,
			Type::Type => self.ty.as_ref(),
			Type::DatatypeRestriction => self.datatype_restriction.as_ref(),
			Type::Property => self.property.as_ref(),
			Type::Component(ty) => self.component.includes(ty),
			Type::LayoutRestriction => self.layout_restriction.as_ref(),
			Type::List => self.list.as_ref(),
		}
	}

	pub fn iter(&self) -> TypesMetadataIter<M> {
		TypesMetadataIter {
			ty: self.ty.as_ref(),
			datatype_restriction: self.datatype_restriction.as_ref(),
			property: self.property.as_ref(),
			component: self.component.iter(),
			layout_restriction: self.layout_restriction.as_ref(),
			list: self.list.as_ref(),
		}
	}
}

impl<'a, M: Clone> TypesMetadata<&'a M> {
	pub fn cloned(&self) -> TypesMetadata<M> {
		TypesMetadata {
			ty: self.ty.cloned(),
			datatype_restriction: self.datatype_restriction.cloned(),
			property: self.property.cloned(),
			component: self.component.cloned(),
			layout_restriction: self.layout_restriction.cloned(),
			list: self.list.cloned(),
		}
	}
}

impl<'a, M> IntoIterator for &'a TypesMetadata<M> {
	type Item = Meta<Type, &'a M>;
	type IntoIter = TypesMetadataIter<'a, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct TypesMetadataIter<'a, M> {
	ty: Option<&'a M>,
	datatype_restriction: Option<&'a M>,
	property: Option<&'a M>,
	component: component::TypesMetadataIter<'a, M>,
	layout_restriction: Option<&'a M>,
	list: Option<&'a M>,
}

impl<'a, M> Iterator for TypesMetadataIter<'a, M> {
	type Item = Meta<Type, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.ty
			.take()
			.map(|m| Meta(Type::Type, m))
			.or_else(|| {
				self.datatype_restriction
					.take()
					.map(|m| Meta(Type::DatatypeRestriction, m))
					.or_else(|| {
						self.property
							.take()
							.map(|m| Meta(Type::Property, m))
							.or_else(|| {
								self.component
									.next()
									.map(|m| m.map(Type::Component))
									.or_else(|| {
										self.layout_restriction
											.take()
											.map(|m| Meta(Type::LayoutRestriction, m))
											.or_else(|| {
												self.list
													.take()
													.map(|m| Meta(Type::List, m))
											})
									})
							})
					})
			})
	}
}

pub struct Parts<M> {
	pub data: Data<M>,
	pub ty: MetaOption<ty::Definition<M>, M>,
	pub property: MetaOption<prop::Definition<M>, M>,
	pub component: MetaOption<component::Definition<M>, M>
}

#[derive(Debug)]
pub struct Data<M> {
	pub id: Id,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

impl<M> Data<M> {
	pub fn new(id: Id) -> Self {
		Self { id, label: Multiple::default(), comment: Multiple::default() }
	}
}

/// Resource.
pub struct Resource;

impl ResourceType for Resource {
	const TYPE: Type = Type::Resource;

	fn check<M>(_resource: &self::Definition<M>) -> bool {
		true
	}
}

/// Resource definition.
#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	ty: MetaOption<ty::Definition<M>, M>,
	property: MetaOption<prop::Definition<M>, M>,
	component: MetaOption<component::Definition<M>, M>
}

impl<M> Definition<M> {
	pub fn new(id: Id) -> Self {
		Self {
			data: Data::new(id),
			ty: MetaOption::default(),
			property: MetaOption::default(),
			component: MetaOption::default()
		}
	}

	pub fn from_parts(parts: Parts<M>) -> Self {
		Self {
			data: parts.data,
			ty: parts.ty,
			property: parts.property,
			component: parts.component
		}
	}

	pub fn into_parts(self) -> Parts<M> {
		Parts {
			data: self.data,
			ty: self.ty,
			property: self.property,
			component: self.component
		}
	}

	pub fn id(&self) -> Id {
		self.data.id
	}

	pub fn label(&self) -> &Multiple<String, M> {
		&self.data.label
	}

	pub fn comment(&self) -> &Multiple<String, M> {
		&self.data.comment
	}

	pub fn is_type(&self) -> bool {
		self.ty.is_some()
	}

	pub fn is_property(&self) -> bool {
		self.property.is_some()
	}

	pub fn is_layout(&self) -> bool {
		self.component.value().map(component::Definition::is_layout).unwrap_or(false)
	}

	pub fn as_type(&self) -> Option<&ty::Definition<M>> {
		self.ty.value()
	}

	pub fn as_property(&self) -> Option<&prop::Definition<M>> {
		self.property.value()
	}

	pub fn as_layout(&self) -> Option<&layout::Definition<M>> {
		self.component.value().and_then(component::Definition::as_layout)
	}

	pub fn types_metadata(&self) -> TypesMetadata<&M> {
		TypesMetadata {
			ty: self.ty.metadata(),
			datatype_restriction: None,
			property: self.property.metadata(),
			component: self.component.value().map(component::Definition::types_metadata).unwrap_or_default(),
			layout_restriction: None,
			list: None,
		}
	}

	pub fn require_layout(&self) -> Result<&layout::Definition<M>, Error<M>>
	where
		M: Clone,
	{
		self.as_layout().ok_or_else(|| {
			error::NodeInvalidType {
				id: self.data.id,
				expected: Type::Component(component::Type::Layout),
				found: self.types_metadata().cloned(),
			}
			.into()
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Type,
	Label,
	Comment
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Rdfs, Rdf, Term};
		match self {
			Self::Type => Term::Rdf(Rdf::Type),
			Self::Label => Term::Rdfs(Rdfs::Label),
			Self::Comment => Term::Rdfs(Rdfs::Comment)
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Type => "type",
			Self::Label => "label",
			Self::Comment => "comment"
		}
	}
}