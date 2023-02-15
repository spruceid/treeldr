use derivative::Derivative;
use locspan::Meta;

use crate::{
	component,
	node::{self, BindingValueRef},
	prop::{PropertyName, UnknownProperty},
	property_values, vocab, FunctionalPropertyValue, Id, IriIndex, Layout, Name,
	RequiredFunctionalPropertyValue, ResourceType, TId,
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

	pub fn name(&self) -> Option<&'a Name> {
		self.as_component().name()
	}

	pub fn format(&self) -> TId<Layout> {
		self.as_formatted().format().unwrap()
	}

	pub fn is_required(&self, model: &crate::MutableModel<M>) -> bool {
		let layout = model.get(self.format()).unwrap().as_layout();
		layout.is_required()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	For(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::FieldFor))),
			Self::For(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Term, TreeLdr};
		match self {
			Self::For(None) => Some(Term::TreeLdr(TreeLdr::FieldFor)),
			Self::For(Some(_)) => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::For(None) => PropertyName::Resource("field property"),
			Self::For(Some(p)) => PropertyName::Other(*p),
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
	prop: FunctionalPropertyValue<TId<crate::Property>, M>,
}

impl<M> Definition<M> {
	pub fn new(prop: FunctionalPropertyValue<TId<crate::Property>, M>) -> Self {
		Self { prop }
	}

	pub fn property(&self) -> Option<&TId<crate::Property>> {
		self.prop
			.as_required()
			.map(RequiredFunctionalPropertyValue::value)
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			prop: self.prop.iter(),
		}
	}
}

pub enum ClassBinding {
	For(Option<TId<UnknownProperty>>, TId<crate::Property>),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::For(p, _) => Property::For(*p),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::For(_, v) => BindingValueRef::Property(*v),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	prop: property_values::functional::Iter<'a, TId<crate::Property>, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.prop
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::For))
	}
}
