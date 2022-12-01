use derivative::Derivative;
use locspan::Meta;

use crate::{
	layout,
	node::BindingValueRef,
	vocab::{self, Term},
	Layout, MetaOption, TId,
};

pub struct Formatted;

#[derive(Debug)]
pub struct Data<M> {
	pub format: MetaOption<TId<Layout>, M>,
}

impl<M> Data<M> {
	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			format: self.format.as_ref(),
		}
	}
}

#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	layout_field: MetaOption<layout::structure::field::Definition<M>, M>,
	layout_variant: MetaOption<layout::enumeration::variant::Definition, M>,
}

impl<M> Definition<M> {
	pub fn new(
		data: Data<M>,
		layout_field: MetaOption<layout::structure::field::Definition<M>, M>,
		layout_variant: MetaOption<layout::enumeration::variant::Definition, M>,
	) -> Self {
		Self {
			data,
			layout_field,
			layout_variant,
		}
	}

	pub fn format(&self) -> &MetaOption<TId<Layout>, M> {
		&self.data.format
	}

	pub fn is_layout_field(&self) -> bool {
		self.layout_field.is_some()
	}

	pub fn is_layout_variant(&self) -> bool {
		self.layout_variant.is_some()
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::structure::field::Definition<M>, M>> {
		self.layout_field.as_ref()
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::enumeration::variant::Definition, M>> {
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
	Format,
	LayoutField(layout::structure::field::Property),
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::TreeLdr;
		match self {
			Self::Format => Term::TreeLdr(TreeLdr::Format),
			Self::LayoutField(p) => p.term(),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Format => "format",
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
			Self::Format => true,
			Self::LayoutField(p) => p.expect_layout(),
		}
	}
}

pub enum ClassBinding {
	Format(TId<Layout>),
}

impl ClassBinding {
	pub fn into_binding(self) -> Binding {
		match self {
			Self::Format(id) => Binding::Format(id),
		}
	}
}

pub enum Binding {
	Format(TId<Layout>),
	LayoutField(layout::structure::field::ClassBinding),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::Format(_) => Property::Format,
			Self::LayoutField(b) => Property::LayoutField(b.property()),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Format(v) => BindingValueRef::Layout(*v),
			Self::LayoutField(b) => b.value(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	format: Option<&'a Meta<TId<Layout>, M>>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.format
			.take()
			.map(|m| m.borrow().into_cloned_value().map(ClassBinding::Format))
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	layout_field: crate::layout::structure::field::ClassBindings<'a, M>,
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
