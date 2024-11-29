use core::fmt;
use std::fmt::Write;

use crate::{Registry, TypeRef};

pub struct DisplayContext<'a, G: Registry> {
	pub registry: &'a G,
	pub self_: Option<TypeRef<G::PackageId>>,
	pub indent: Indent,
}

impl<'a, G: Registry> DisplayContext<'a, G> {
	pub fn inner(&self) -> Self {
		Self {
			registry: self.registry,
			self_: self.self_,
			indent: Indent(self.indent.0 + 1),
		}
	}

	pub fn inner_self(&self, self_: TypeRef<G::PackageId>) -> Self {
		Self {
			registry: self.registry,
			self_: Some(self_),
			indent: Indent(self.indent.0 + 1),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Indent(pub usize);

impl fmt::Display for Indent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for _ in 0..self.0 {
			f.write_char('\t')?;
		}

		Ok(())
	}
}

pub trait DisplaySignature<P, R> {
	fn display_signature<'a, G>(&'a self, registry: &'a G) -> DisplayedSignature<'a, G, Self>
	where
		G: Registry<PackageId = P, Resource = R>,
	{
		self.display_signature_with(DisplayContext {
			registry,
			self_: None,
			indent: Indent(0),
		})
	}

	fn display_signature_with<'a, G>(
		&'a self,
		context: DisplayContext<'a, G>,
	) -> DisplayedSignature<'a, G, Self>
	where
		G: Registry<PackageId = P, Resource = R>,
	{
		DisplayedSignature { context, t: self }
	}

	fn fmt_signature(
		&self,
		context: &DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result;
}

pub struct DisplayedSignature<'a, G: Registry, T: ?Sized> {
	context: DisplayContext<'a, G>,
	t: &'a T,
}

impl<'a, G, T> fmt::Display for DisplayedSignature<'a, G, T>
where
	G: Registry,
	T: ?Sized + DisplaySignature<G::PackageId, G::Resource>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.t.fmt_signature(&self.context, f)
	}
}
