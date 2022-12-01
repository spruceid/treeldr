use derivative::Derivative;
use locspan::Meta;

use crate::{
	component,
	node::{self, BindingValueRef},
	vocab, Layout, MetaOption, Name, ResourceType, TId,
};

pub struct Field;

impl ResourceType for Field {
	const TYPE: crate::Type = crate::Type::Resource(Some(node::Type::Component(Some(
		component::Type::Formatted(Some(component::formatted::Type::LayoutField)),
	))));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_layout_field()
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

	pub fn name(&self) -> Option<&'a Meta<Name, M>> {
		self.as_component().name()
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

	pub fn expect_type(&self) -> bool {
		false
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}

/// Layout field.
#[derive(Debug, Clone)]
pub struct Definition<M> {
	prop: MetaOption<TId<crate::Property>, M>,
}

impl<M> Definition<M> {
	pub fn new(prop: MetaOption<TId<crate::Property>, M>) -> Self {
		Self { prop }
	}

	pub fn property(&self) -> Option<&Meta<TId<crate::Property>, M>> {
		self.prop.as_ref()
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			prop: self.prop.as_ref(),
		}
	}
}

pub enum ClassBinding {
	For(TId<crate::Property>),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::For(_) => Property::For,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::For(v) => BindingValueRef::Property(*v),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	prop: Option<&'a Meta<TId<crate::Property>, M>>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.prop
			.take()
			.map(|m| m.borrow().into_cloned_value().map(ClassBinding::For))
	}
}
