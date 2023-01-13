use locspan::Meta;

use crate::{component, node, ResourceType};

pub struct Variant;

impl ResourceType for Variant {
	const TYPE: crate::Type = crate::Type::Resource(Some(node::Type::Component(Some(
		component::Type::Formatted(Some(component::formatted::Type::LayoutVariant)),
	))));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_layout_variant()
	}
}

impl<'a, M> crate::Ref<'a, Variant, M> {
	pub fn as_component(&self) -> &'a Meta<component::Definition<M>, M> {
		self.as_resource().as_component().unwrap()
	}

	pub fn as_formatted(&self) -> &'a Meta<component::formatted::Definition<M>, M> {
		self.as_resource().as_formatted().unwrap()
	}

	pub fn as_layout_variant(&self) -> &'a Meta<Definition, M> {
		self.as_resource().as_layout_variant().unwrap()
	}
}

#[derive(Debug, Clone)]
pub struct Definition;
