use crate::{error, layout, prop, ty, Error, Id, MetaOption, component, Multiple, Type, ResourceType, vocab};
use locspan::Meta;

pub struct Parts<M> {
	pub data: Data<M>,
	pub ty: MetaOption<ty::Definition<M>, M>,
	pub property: MetaOption<prop::Definition<M>, M>,
	pub component: MetaOption<component::Definition<M>, M>
}

#[derive(Debug)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: Multiple<Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self { id, metadata, type_: Multiple::default(), label: Multiple::default(), comment: Multiple::default() }
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
	pub fn new(
		data: Data<M>,
		ty: MetaOption<ty::Definition<M>, M>,
		property: MetaOption<prop::Definition<M>, M>,
		component: MetaOption<component::Definition<M>, M>
	) -> Self {
		Self {
			data,
			ty,
			property,
			component
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

	pub fn type_(&self) -> &Multiple<Type, M> {
		&self.data.type_
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

	pub fn as_type(&self) -> Option<&Meta<ty::Definition<M>, M>> {
		self.ty.as_ref()
	}

	pub fn as_property(&self) -> Option<&Meta<prop::Definition<M>, M>> {
		self.property.as_ref()
	}

	pub fn as_component(&self) -> Option<&Meta<component::Definition<M>, M>> {
		self.component.as_ref()
	}

	pub fn as_layout(&self) -> Option<&Meta<layout::Definition<M>, M>> {
		self.component.value().and_then(component::Definition::as_layout)
	}

	pub fn as_formatted(&self) -> Option<&Meta<component::formatted::Definition<M>, M>> {
		self.component.value().and_then(component::Definition::as_formatted)
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::field::Definition<M>, M>> {
		self.component.value().and_then(component::Definition::as_layout_field)
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::variant::Definition, M>> {
		self.component.value().and_then(component::Definition::as_layout_variant)
	}

	pub fn require_layout(&self) -> Result<&Meta<layout::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		self.as_layout().ok_or_else(|| {
			error::NodeInvalidType {
				id: self.data.id,
				expected: Type::Component(Some(component::Type::Layout)),
				found: self.type_().clone()
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