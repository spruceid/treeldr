use derivative::Derivative;
use locspan::Meta;

use crate::{
	layout,
	node::BindingValueRef,
	prop::{PropertyName, UnknownProperty},
	property_values,
	vocab::{self, Term},
	FunctionalPropertyValue, Id, IriIndex, MetaOption, Name, TId,
};

pub mod formatted;

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub name: FunctionalPropertyValue<Name, M>,
}

impl<M> Data<M> {
	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			name: self.name.iter(),
		}
	}
}

#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	layout: MetaOption<layout::Definition<M>, M>,
	formatted: MetaOption<formatted::Definition<M>, M>,
}

impl<M> Definition<M> {
	pub fn new(
		data: Data<M>,
		layout: MetaOption<layout::Definition<M>, M>,
		formatted: MetaOption<formatted::Definition<M>, M>,
	) -> Self {
		Self {
			data,
			layout,
			formatted,
		}
	}

	pub fn is_layout(&self) -> bool {
		self.layout.is_some()
	}

	pub fn is_layout_field(&self) -> bool {
		self.formatted
			.value()
			.map(formatted::Definition::is_layout_field)
			.unwrap_or(false)
	}

	pub fn is_layout_variant(&self) -> bool {
		self.formatted
			.value()
			.map(formatted::Definition::is_layout_variant)
			.unwrap_or(false)
	}

	pub fn as_layout(&self) -> Option<&Meta<layout::Definition<M>, M>> {
		self.layout.as_ref()
	}

	pub fn as_formatted(&self) -> Option<&Meta<formatted::Definition<M>, M>> {
		self.formatted.as_ref()
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::field::Definition<M>, M>> {
		self.formatted
			.value()
			.and_then(formatted::Definition::as_layout_field)
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::variant::Definition, M>> {
		self.formatted
			.value()
			.and_then(formatted::Definition::as_layout_variant)
	}

	pub fn name(&self) -> Option<&Name> {
		self.data.name.value()
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.data.bindings(),
			layout: self
				.layout
				.as_ref()
				.map(|l| l.bindings()),
			formatted: self
				.formatted
				.as_ref()
				.map(|f| f.bindings())
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Layout,
	Formatted(Option<formatted::Type>),
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, other: Self) -> bool {
		match (self, other) {
			(Self::Formatted(Some(_)), Self::Formatted(None)) => true,
			(Self::Formatted(Some(a)), Self::Formatted(Some(b))) => a.is_subclass_of(b),
			_ => false,
		}
	}

	pub fn term(&self) -> Term {
		match self {
			Self::Layout => Term::TreeLdr(vocab::TreeLdr::Layout),
			Self::Formatted(None) => Term::TreeLdr(vocab::TreeLdr::Formatted),
			Self::Formatted(Some(ty)) => ty.term(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub enum Property {
	Name(Option<TId<UnknownProperty>>),
	Layout(layout::Property),
	Formatted(formatted::Property),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::TreeLdr;
		match self {
			Self::Name(None) => Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Name))),
			Self::Name(Some(p)) => p.id(),
			Self::Layout(p) => p.id(),
			Self::Formatted(p) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::TreeLdr;
		match self {
			Self::Name(None) => Some(Term::TreeLdr(TreeLdr::Name)),
			Self::Name(Some(_)) => None,
			Self::Layout(p) => p.term(),
			Self::Formatted(p) => p.term(),
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::Name(None) => PropertyName::Resource("name"),
			Self::Name(Some(p)) => PropertyName::Other(*p),
			Self::Layout(p) => p.name(),
			Self::Formatted(p) => p.name(),
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::Layout(p) => p.expect_type(),
			Self::Formatted(p) => p.expect_type(),
			_ => false,
		}
	}

	pub fn expect_layout(&self) -> bool {
		match self {
			Self::Layout(p) => p.expect_layout(),
			Self::Formatted(p) => p.expect_layout(),
			_ => false,
		}
	}
}

pub enum ClassBindingRef<'a> {
	Name(Option<TId<UnknownProperty>>, &'a Name),
}

impl<'a> ClassBindingRef<'a> {
	pub fn into_binding_ref<M>(self) -> BindingRef<'a, M> {
		match self {
			Self::Name(id, n) => BindingRef::Name(id, n),
		}
	}
}

pub enum BindingRef<'a, M> {
	Name(Option<TId<UnknownProperty>>, &'a Name),
	Layout(layout::BindingRef<'a, M>),
	Formatted(formatted::Binding),
}

impl<'a, M> BindingRef<'a, M> {
	pub fn domain(&self) -> Option<Type> {
		match self {
			Self::Layout(_) => Some(Type::Layout),
			Self::Formatted(b) => Some(Type::Formatted(b.domain())),
			_ => None,
		}
	}

	pub fn property(&self) -> Property {
		match self {
			Self::Name(p, _) => Property::Name(*p),
			Self::Layout(b) => Property::Layout(b.property()),
			Self::Formatted(b) => Property::Formatted(b.property()),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Name(_, v) => BindingValueRef::Name(v),
			Self::Layout(b) => b.value(),
			Self::Formatted(b) => b.value(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	name: property_values::functional::Iter<'a, Name, M>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.name
			.next()
			.map(|m| m.into_class_binding(ClassBindingRef::Name))
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	layout: Option<layout::Bindings<'a, M>>,
	formatted: Option<formatted::Bindings<'a, M>>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<BindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBindingRef::into_binding_ref))
			.or_else(|| {
				self.layout
					.as_mut()
					.and_then(|i| {
						i.next().map(|m| m.map(BindingRef::Layout))
					})
			})
			.or_else(|| {
				self.formatted
					.as_mut()
					.and_then(|i| {
						i.next().map(|m| m.map(BindingRef::Formatted))
					})
			})
	}
}
