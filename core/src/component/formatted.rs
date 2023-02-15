use derivative::Derivative;
use locspan::Meta;

use crate::{
	layout,
	node::BindingValueRef,
	prop::{PropertyName, UnknownProperty},
	property_values,
	vocab::{self, Term},
	FunctionalPropertyValue, Id, IriIndex, Layout, MetaOption, RequiredFunctionalPropertyValue,
	TId,
};

pub struct Formatted;

#[derive(Debug)]
pub struct Data<M> {
	pub format: FunctionalPropertyValue<TId<Layout>, M>,
}

impl<M> Data<M> {
	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			format: self.format.iter(),
		}
	}
}

#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	layout_field: MetaOption<layout::field::Definition<M>, M>,
	layout_variant: MetaOption<layout::variant::Definition, M>,
}

impl<M> Definition<M> {
	pub fn new(
		data: Data<M>,
		layout_field: MetaOption<layout::field::Definition<M>, M>,
		layout_variant: MetaOption<layout::variant::Definition, M>,
	) -> Self {
		Self {
			data,
			layout_field,
			layout_variant,
		}
	}

	pub fn format(&self) -> Option<TId<Layout>> {
		self.data
			.format
			.as_required()
			.map(RequiredFunctionalPropertyValue::value)
			.cloned()
	}

	pub fn is_layout_field(&self) -> bool {
		self.layout_field.is_some()
	}

	pub fn is_layout_variant(&self) -> bool {
		self.layout_variant.is_some()
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::field::Definition<M>, M>> {
		self.layout_field.as_ref()
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::variant::Definition, M>> {
		self.layout_variant.as_ref()
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.data.bindings(),
			layout_field: self
				.layout_field
				.as_ref()
				.map(|f| f.bindings())
				.unwrap_or_default(),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	LayoutField,
	LayoutVariant,
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, _other: Self) -> bool {
		false
	}

	pub fn term(&self) -> Term {
		match self {
			Self::LayoutField => Term::TreeLdr(vocab::TreeLdr::Field),
			Self::LayoutVariant => Term::TreeLdr(vocab::TreeLdr::Variant),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Format(Option<TId<UnknownProperty>>),
	LayoutField(layout::field::Property),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::TreeLdr;
		match self {
			Self::Format(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Format))),
			Self::Format(Some(p)) => p.id(),
			Self::LayoutField(p) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::TreeLdr;
		match self {
			Self::Format(None) => Some(Term::TreeLdr(TreeLdr::Format)),
			Self::Format(Some(_)) => None,
			Self::LayoutField(p) => p.term(),
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::Format(None) => PropertyName::Resource("format"),
			Self::Format(Some(p)) => PropertyName::Other(*p),
			Self::LayoutField(p) => p.name(),
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::LayoutField(p) => p.expect_type(),
			_ => false,
		}
	}

	pub fn expect_layout(&self) -> bool {
		match self {
			Self::Format(_) => true,
			Self::LayoutField(p) => p.expect_layout(),
		}
	}
}

pub enum ClassBinding {
	Format(Option<TId<UnknownProperty>>, TId<Layout>),
}

impl ClassBinding {
	pub fn into_binding(self) -> Binding {
		match self {
			Self::Format(p, id) => Binding::Format(p, id),
		}
	}
}

pub enum Binding {
	Format(Option<TId<UnknownProperty>>, TId<Layout>),
	LayoutField(layout::field::ClassBinding),
}

impl Binding {
	pub fn domain(&self) -> Option<Type> {
		match self {
			Self::LayoutField(_) => Some(Type::LayoutField),
			_ => None,
		}
	}

	pub fn property(&self) -> Property {
		match self {
			Self::Format(p, _) => Property::Format(*p),
			Self::LayoutField(b) => Property::LayoutField(b.property()),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Format(_, v) => BindingValueRef::Layout(*v),
			Self::LayoutField(b) => b.value(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	format: property_values::functional::Iter<'a, TId<Layout>, M>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.format
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::Format))
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	layout_field: crate::layout::field::ClassBindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<Binding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBinding::into_binding))
			.or_else(|| {
				self.layout_field
					.next()
					.map(|m| m.map(Binding::LayoutField))
			})
	}
}
