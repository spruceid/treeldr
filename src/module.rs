use core::fmt;
use std::{collections::BTreeMap, fmt::Write, hash::Hash, ops::Deref};

use educe::Educe;
use iref::{Iri, IriBuf};
use serde::{Deserialize, Serialize};
use serde_seeded::{unseeded, DeserializeSeeded, SerializeSeeded};

use crate::{
	display::{self, DisplaySignature},
	function::FunctionRef,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	Attributes, Registry, TypeRef,
};

#[derive(Debug, Clone, Educe, SerializeSeeded, DeserializeSeeded)]
#[educe(Default)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>), bounds(T: Ord))
)]
pub struct Module<P, R = rdf_types::Term, T = TypeRef<P>> {
	pub parent: Option<ModuleRef<P>>,
	pub attributes: Attributes<R>,
	pub items: BTreeMap<String, Item<P, T>>,
}

impl<P, R, T> Module<P, R, T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get_by_segment(&self, segment: &ItemPathSegment) -> Option<ItemRef<P, T>>
	where
		P: Copy,
		T: Clone,
	{
		match segment {
			ItemPathSegment::Super => self.parent.map(ItemRef::Module),
			ItemPathSegment::Item(name) => self.get_by_name(name),
		}
	}

	pub fn get_by_name(&self, name: &str) -> Option<ItemRef<P, T>>
	where
		P: Copy,
		T: Clone,
	{
		Some(self.items.get(name)?.as_ref())
	}

	pub fn insert(&mut self, name: impl Into<String>, item: Item<P, T>) -> Option<Item<P, T>> {
		self.items.insert(name.into(), item)
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Module<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Module<P, R, U> {
		Module {
			parent: self.parent,
			attributes: self.attributes,
			items: self
				.items
				.into_iter()
				.map(|(name, i)| (name, i.map_types_ref_mut(f)))
				.collect(),
		}
	}
}

impl<P: Copy, R> Module<P, R> {
	pub fn get<'a>(
		&'a self,
		registry: &'a impl Registry<PackageId = P, Resource = R>,
		this: ModuleRef<P>,
		path: &ItemPath,
	) -> Option<ItemRef<'a, P>> {
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_segment(first)
				.and_then(|i| i.get(registry, ItemPath::from_slice(rest))),
			None => Some(ItemRef::Module(this)),
		}
	}
}

impl<P, R, T> VisitResources<R> for Module<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.attributes.visit_resources_ref_mut(f)
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for Module<P, R> {
	fn fmt_signature(
		&self,
		context: &display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		for (name, item) in &self.items {
			match item {
				Item::Prefix(item) => writeln!(f, "{}prefix = <{item}>", context.indent)?,
				Item::Type(item) => writeln!(
					f,
					"{}type {} = {}",
					context.indent,
					name,
					item.display_signature_with(context.inner())
				)?,
				Item::Function(item) => {
					let function = context.registry.get_function(*item).unwrap();

					write!(f, "{}let ({}", context.indent, name)?;

					for a in &function.signature.args {
						write!(f, " {}", a.display_signature_with(context.inner()))?
					}

					writeln!(
						f,
						"): {}",
						function
							.signature
							.return_type
							.display_signature_with(context.inner())
					)?
				}
				Item::Module(m) => {
					writeln!(f, "{}mod {} {{", context.indent, name)?;
					m.fmt_signature(&context.inner(), f)?;
					writeln!(f, "{}}}", context.indent)?
				}
			}
		}

		Ok(())
	}
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(params(Q), seed(Q), override_bounds(P: Serialize)),
	de(params(Q), seed(Q), override_bounds(P: Deserialize<'de>))
)]
pub struct ModuleRef<P> {
	#[seeded(with(unseeded))]
	pub package: P,
	pub index: u32,
}

impl<P> ModuleRef<P> {
	pub fn new(package: P, index: u32) -> Self {
		Self { package, index }
	}
}

impl<P: Copy> ModuleRef<P> {
	pub fn get<'a>(
		&self,
		registry: &'a impl Registry<PackageId = P>,
		path: &ItemPath,
	) -> Option<ItemRef<'a, P>>
	where
		P: 'a,
	{
		registry.get_module(*self)?.get(registry, *self, path)
	}

	pub fn get_by_segment<'a>(
		&self,
		registry: &'a impl Registry<PackageId = P>,
		segment: &ItemPathSegment,
	) -> Option<ItemRef<'a, P>>
	where
		P: 'a,
	{
		registry.get_module(*self)?.get_by_segment(segment)
	}

	pub fn get_by_name<'a>(
		&self,
		registry: &'a impl Registry<PackageId = P>,
		name: &str,
	) -> Option<ItemRef<'a, P>>
	where
		P: 'a,
	{
		registry.get_module(*self)?.get_by_name(name)
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for ModuleRef<P> {
	fn fmt_signature(
		&self,
		context: &display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		context
			.registry
			.get_module(*self)
			.unwrap()
			.fmt_signature(context, f)
	}
}

#[derive(Debug, Clone, SerializeSeeded, DeserializeSeeded)]
#[seeded(
	ser(params(Q), seed(Q), override_bounds(P: Serialize, T: Serialize)),
	de(params(Q), seed(Q), override_bounds(P: Deserialize<'de>, T: Deserialize<'de>))
)]
pub enum Item<P, T = TypeRef<P>> {
	Prefix(#[seeded(with(unseeded))] IriBuf),
	Type(#[seeded(with(unseeded))] T),
	Function(FunctionRef<P>),
	Module(ModuleRef<P>),
}

impl<P, T> Item<P, T> {
	pub fn as_ref(&self) -> ItemRef<P, T>
	where
		P: Copy,
		T: Clone,
	{
		match self {
			Self::Prefix(i) => ItemRef::Prefix(i),
			Self::Type(t) => ItemRef::Type(t.clone()),
			Self::Function(f) => ItemRef::Function(*f),
			Self::Module(m) => ItemRef::Module(*m),
		}
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Item<P, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Item<P, U> {
		match self {
			Self::Prefix(i) => Item::Prefix(i),
			Self::Type(t) => Item::Type(f(t)),
			Self::Function(f) => Item::Function(f),
			Self::Module(m) => Item::Module(m),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ItemRef<'a, P, T = TypeRef<P>> {
	Prefix(&'a Iri),
	Type(T),
	Function(FunctionRef<P>),
	Module(ModuleRef<P>),
}

impl<P: Copy, T> ItemRef<'_, P, T> {
	pub fn as_function(&self) -> Option<FunctionRef<P>> {
		match self {
			Self::Function(f) => Some(*f),
			_ => None,
		}
	}

	pub fn as_module(&self) -> Option<ModuleRef<P>> {
		match self {
			Self::Module(m) => Some(*m),
			_ => None,
		}
	}
}

impl<'a, P: 'a + Copy> ItemRef<'a, P> {
	pub fn get(&self, registry: &'a impl Registry<PackageId = P>, path: &ItemPath) -> Option<Self> {
		if path.is_empty() {
			Some(*self)
		} else {
			self.as_module()?.get(registry, path)
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemPathSegment {
	Super,
	Item(String),
}

impl fmt::Display for ItemPathSegment {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Super => write!(f, "super"),
			Self::Item(name) => f.write_str(name),
		}
	}
}

#[derive(Debug)]
pub struct ItemPath(pub [ItemPathSegment]);

impl ItemPath {
	pub fn from_slice(slice: &[ItemPathSegment]) -> &Self {
		unsafe { std::mem::transmute(slice) }
	}
}

impl Deref for ItemPath {
	type Target = [ItemPathSegment];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl fmt::Display for ItemPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_empty() {
			write!(f, "self")
		} else {
			for (i, s) in self.iter().enumerate() {
				if i > 0 {
					f.write_char('.')?;
				}

				s.fmt(f)?;
			}

			Ok(())
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemPathBuf(pub Vec<ItemPathSegment>);

impl Deref for ItemPathBuf {
	type Target = ItemPath;

	fn deref(&self) -> &Self::Target {
		ItemPath::from_slice(&self.0)
	}
}

impl fmt::Display for ItemPathBuf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		ItemPath::from_slice(&self.0).fmt(f)
	}
}
