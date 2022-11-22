use locspan::Meta;

use crate::{TId, Layout, layout, vocab::{self, Term}, MetaOption};

pub struct Formatted;

#[derive(Debug)]
pub struct Data<M> {
	pub format: MetaOption<TId<Layout>, M>
}

#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	layout_field: MetaOption<layout::field::Definition<M>, M>,
	layout_variant: MetaOption<layout::variant::Definition, M>
}

impl<M> Definition<M> {
	pub fn new(
		data: Data<M>,
		layout_field: MetaOption<layout::field::Definition<M>, M>,
		layout_variant: MetaOption<layout::variant::Definition, M>
	) -> Self {
		Self {
			data,
			layout_field,
			layout_variant
		}
	}

	pub fn format(&self) -> &MetaOption<TId<Layout>, M> {
		&self.data.format
	}

	pub fn as_layout_field(&self) -> Option<&Meta<layout::field::Definition<M>, M>> {
		self.layout_field.as_ref()
	}

	pub fn as_layout_variant(&self) -> Option<&Meta<layout::variant::Definition, M>> {
		self.layout_variant.as_ref()
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
			Self::LayoutVariant => Term::TreeLdr(vocab::TreeLdr::Variant)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Format,
	LayoutField(layout::field::Property),
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::TreeLdr;
		match self {
			Self::Format => Term::TreeLdr(TreeLdr::Format),
			Self::LayoutField(p) => p.term()
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Format => "format",
			Self::LayoutField(p) => p.name()
		}
	}
}