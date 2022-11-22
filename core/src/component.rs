use locspan::Meta;

use crate::{Name, MetaOption, layout, vocab::{self, Term}};

pub mod formatted;

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub name: MetaOption<Name, M>
}

#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	layout: MetaOption<layout::Definition<M>, M>,
	formatted: MetaOption<formatted::Definition<M>, M>
}

impl<M> Definition<M> {
	pub fn new(
		data: Data<M>,
		layout: MetaOption<layout::Definition<M>, M>,
		formatted: MetaOption<formatted::Definition<M>, M>
	) -> Self {
		Self { data, layout, formatted }
	}

	pub fn is_layout(&self) -> bool {
		self.layout.is_some()
	}

	pub fn as_layout(&self) -> Option<&Meta<layout::Definition<M>, M>> {
		self.layout.as_ref()
	}

	pub fn as_formatted(&self) -> Option<&Meta<formatted::Definition<M>, M>> {
		self.formatted.as_ref()
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::field::Definition<M>, M>> {
		self.formatted.value().and_then(formatted::Definition::as_layout_field)
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::variant::Definition, M>> {
		self.formatted.value().and_then(formatted::Definition::as_layout_variant)
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		self.data.name.as_ref()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Layout,
	Formatted(Option<formatted::Type>)
}

impl Type {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, other: Self) -> bool {
		match (self, other) {
			(Self::Formatted(Some(_)), Self::Formatted(None)) => true,
			(Self::Formatted(Some(a)), Self::Formatted(Some(b))) => a.is_subclass_of(b),
			_ => false
		}
	}

	pub fn term(&self) -> Term {
		match self {
			Self::Layout => Term::TreeLdr(vocab::TreeLdr::Layout),
			Self::Formatted(None) => Term::TreeLdr(vocab::TreeLdr::Formatted),
			Self::Formatted(Some(ty)) => ty.term()
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub enum Property {
	Name,
	Layout(layout::Property),
	Formatted(formatted::Property)
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::TreeLdr;
		match self {
			Self::Name => Term::TreeLdr(TreeLdr::Name),
			Self::Layout(p) => p.term(),
			Self::Formatted(p) => p.term()
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Name => "format",
			Self::Layout(p) => p.name(),
			Self::Formatted(p) => p.name()
		}
	}
}