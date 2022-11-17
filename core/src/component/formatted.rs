use locspan::Meta;

use crate::{TId, Layout, layout, vocab, MetaOption};

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Types {
	pub layout_field: bool,
	pub layout_variant: bool,
}

impl Types {
	pub fn includes(&self, ty: Type) -> bool {
		match ty {
			Type::LayoutField => self.layout_field,
			Type::LayoutVariant => self.layout_variant,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct TypesMetadata<M> {
	pub layout_field: Option<M>,
	pub layout_variant: Option<M>,
}

impl<M> Default for TypesMetadata<M> {
	fn default() -> Self {
		Self {
			layout_field: None,
			layout_variant: None
		}
	}
}

impl<M> TypesMetadata<M> {
	pub fn is_empty(&self) -> bool {
		self.layout_field.is_none() && self.layout_variant.is_none()
	}

	pub fn includes(&self, ty: Type) -> Option<&M> {
		match ty {
			Type::LayoutField => self.layout_field.as_ref(),
			Type::LayoutVariant => self.layout_variant.as_ref()
		}
	}

	pub fn iter(&self) -> TypesMetadataIter<M> {
		TypesMetadataIter {
			layout_field: self.layout_field.as_ref(),
			layout_variant: self.layout_variant.as_ref()
		}
	}
}

impl<'a, M: Clone> TypesMetadata<&'a M> {
	pub fn cloned(&self) -> TypesMetadata<M> {
		TypesMetadata {
			layout_field: self.layout_field.cloned(),
			layout_variant: self.layout_variant.cloned()
		}
	}
}

pub struct TypesMetadataIter<'a, M> {
	layout_field: Option<&'a M>,
	layout_variant: Option<&'a M>
}

impl<'a, M> Iterator for TypesMetadataIter<'a, M> {
	type Item = Meta<Type, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.layout_field
			.take()
			.map(|m| Meta(Type::LayoutField, m))
			.or_else(|| {
				self.layout_variant
					.take()
					.map(|m| Meta(Type::LayoutVariant, m))
			})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Format,
	LayoutField(layout::field::Property),
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, TreeLdr};
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