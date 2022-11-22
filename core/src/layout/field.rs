use locspan::Meta;

use crate::{vocab, MetaOption, TId, ResourceType, component, Layout, Name, node};

pub struct Field;

impl ResourceType for Field {
	const TYPE: crate::Type = crate::Type::Resource(Some(node::Type::Component(Some(component::Type::Formatted(Some(component::formatted::Type::LayoutField))))));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_layout()
	}
}

impl<'a, M> crate::Ref<'a, Field, M> {
	pub fn as_component(&self) -> &'a Meta<component::Definition<M>, M> {
		self.as_resource().as_component().unwrap()
	}
	
	pub fn as_formatted(&self) -> &'a Meta<component::formatted::Definition<M>, M> {
		self.as_resource().as_formatted().unwrap()
	}

	pub fn as_layout_field(&self) -> &'a Meta<Definition<M>, M> {
		self.as_resource().as_layout_field().unwrap()
	}

	pub fn name(&self) -> &'a Meta<Name, M> {
		self.as_component().name().as_ref().unwrap()
	}

	pub fn format(&self) -> &'a Meta<TId<Layout>, M> {
		self.as_formatted().format().as_ref().unwrap()
	}

	pub fn is_required(&self, model: &crate::Model<M>) -> bool {
		let layout = model.get(**self.format()).unwrap().as_layout();
		layout.is_required()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	For,
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For => Term::TreeLdr(TreeLdr::FieldFor),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::For => "field property",
		}
	}
}

/// Layout field.
#[derive(Debug, Clone)]
pub struct Definition<M> {
	prop: MetaOption<TId<crate::Property>, M>,
}

impl<M> Definition<M> {
	pub fn new(
		prop: MetaOption<TId<crate::Property>, M>
	) -> Self {
		Self {
			prop
		}
	}

	pub fn property(&self) -> Option<&Meta<TId<crate::Property>, M>> {
		self.prop.as_ref()
	}
}