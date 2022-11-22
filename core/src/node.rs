use crate::{error, layout, prop, ty, Error, Id, MetaOption, component, Multiple, ResourceType, vocab::{self, Term}, list};
use locspan::Meta;

#[derive(Debug, Clone)]
pub struct AnonymousData<M> {
	pub type_: Multiple<crate::Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: Multiple<crate::Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self { id, metadata, type_: Multiple::default(), label: Multiple::default(), comment: Multiple::default() }
	}

	pub fn clone_anonymous(&self) -> AnonymousData<M> where M: Clone {
		AnonymousData {
			type_: self.type_.clone(),
			label: self.label.clone(),
			comment: self.comment.clone()
		}
	}
}

/// Resource.
pub struct Resource;

impl ResourceType for Resource {
	const TYPE: crate::Type = crate::Type::Resource(None);

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

	pub fn id(&self) -> Id {
		self.data.id
	}

	pub fn type_(&self) -> &Multiple<crate::Type, M> {
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
				expected: crate::Type::Resource(Some(Type::Component(Some(component::Type::Layout)))),
				found: self.type_().clone()
			}
			.into()
		})
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Class(Option<ty::SubClass>),
	DatatypeRestriction,
	Property(Option<prop::Type>),
	Component(Option<component::Type>),
	LayoutRestriction,
	List,
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, other: Self) -> bool {
		match (self, other) {
			(Self::Class(Some(_)), Self::Class(None)) => true,
			(Self::Class(Some(a)), Self::Class(Some(b))) => a.is_subclass_of(b),
			(Self::Property(Some(_)), Self::Property(None)) => true,
			(Self::Property(Some(a)), Self::Property(Some(b))) => a.is_subclass_of(b),
			(Self::Component(Some(_)), Self::Component(None)) => true,
			(Self::Component(Some(a)), Self::Component(Some(b))) => a.is_subclass_of(b),
			_ => false
		}
	}

	pub fn term(&self) -> Term {
		match self {
			Self::Class(None) => Term::Rdfs(vocab::Rdfs::Class),
			Self::Class(Some(ty)) => ty.term(),
			Self::DatatypeRestriction => Term::Rdfs(vocab::Rdfs::Resource),
			Self::Property(None) => Term::Rdf(vocab::Rdf::Property),
			Self::Property(Some(ty)) => ty.term(),
			Self::Component(None) => Term::TreeLdr(vocab::TreeLdr::Component),
			Self::Component(Some(ty)) => ty.term(),
			Self::LayoutRestriction => Term::TreeLdr(vocab::TreeLdr::LayoutRestriction),
			Self::List => Term::Rdf(vocab::Rdf::List)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Type,
	Label,
	Comment,
	Class(ty::Property),
	DatatypeRestriction(ty::data::restriction::Property),
	Property(prop::RdfProperty),
	Component(component::Property),
	LayoutRestriction(layout::restriction::Property),
	List(list::Property)
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Rdfs, Rdf};
		match self {
			Self::Type => Term::Rdf(Rdf::Type),
			Self::Label => Term::Rdfs(Rdfs::Label),
			Self::Comment => Term::Rdfs(Rdfs::Comment),
			Self::Class(p) => p.term(),
			Self::DatatypeRestriction(p) => p.term(),
			Self::Property(p) => p.term(),
			Self::Component(p) => p.term(),
			Self::LayoutRestriction(p) => p.term(),
			Self::List(p) => p.term()
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Type => "type",
			Self::Label => "label",
			Self::Comment => "comment",
			Self::Class(p) => p.name(),
			Self::DatatypeRestriction(p) => p.name(),
			Self::Property(p) => p.name(),
			Self::Component(p) => p.name(),
			Self::LayoutRestriction(p) => p.name(),
			Self::List(p) => p.name()
		}
	}
}