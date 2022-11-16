use locspan::Meta;

use crate::{Name, MetaOption, layout, vocab};

pub mod formatted;

#[derive(Debug)]
pub struct Data<M> {
	pub name: MetaOption<Name, M>
}

#[derive(Debug)]
pub struct Definition<M> {
	data: Data<M>,
	layout: MetaOption<layout::Definition<M>, M>
}

impl<M> Definition<M> {
	pub fn is_layout(&self) -> bool {
		self.layout.is_some()
	}

	pub fn as_layout(&self) -> Option<&layout::Definition<M>> {
		self.layout.value()
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		self.data.name.as_ref()
	}

	pub fn types_metadata(&self) -> TypesMetadata<&M> {
		TypesMetadata { layout: self.layout.metadata(), formatted: formatted::TypesMetadata::default() }
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Layout,
	Formatted(formatted::Type)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Types {
	pub layout: bool,
	pub formatted: formatted::Types
}

impl Types {
	pub fn includes(&self, ty: Type) -> bool {
		match ty {
			Type::Layout => self.layout,
			Type::Formatted(ty) => self.formatted.includes(ty)
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct TypesMetadata<M> {
	pub layout: Option<M>,
	pub formatted: formatted::TypesMetadata<M>
}

impl<M> Default for TypesMetadata<M> {
	fn default() -> Self {
		Self {
			layout: None,
			formatted: formatted::TypesMetadata::default()
		}
	}
}

impl<M> TypesMetadata<M> {
	pub fn is_empty(&self) -> bool {
		self.layout.is_none() && self.formatted.is_empty()
	}

	pub fn includes(&self, ty: Type) -> Option<&M> {
		match ty {
			Type::Layout => self.layout.as_ref(),
			Type::Formatted(ty) => self.formatted.includes(ty)
		}
	}

	pub fn iter(&self) -> TypesMetadataIter<M> {
		TypesMetadataIter {
			layout: self.layout.as_ref(),
			formatted: self.formatted.iter()
		}
	}
}

impl<'a, M: Clone> TypesMetadata<&'a M> {
	pub fn cloned(&self) -> TypesMetadata<M> {
		TypesMetadata {
			layout: self.layout.cloned(),
			formatted: self.formatted.cloned()
		}
	}
}

pub struct TypesMetadataIter<'a, M> {
	layout: Option<&'a M>,
	formatted: formatted::TypesMetadataIter<'a, M>
}

impl<'a, M> Iterator for TypesMetadataIter<'a, M> {
	type Item = Meta<Type, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.layout
			.take()
			.map(|m| Meta(Type::Layout, m))
			.or_else(|| {
				self.formatted
					.next()
					.map(|m| m.map(Type::Formatted))
			})
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
		use vocab::{Term, TreeLdr};
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