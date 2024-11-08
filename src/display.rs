use core::fmt;
use std::fmt::Write;

use crate::{Registry, TypeRef};

pub struct DisplayContext<'a, R> {
	pub registry: &'a Registry<R>,
	pub self_: Option<TypeRef>,
	pub indent: Indent,
}

impl<'a, R> DisplayContext<'a, R> {
	pub fn inner(&self) -> Self {
		Self {
			registry: self.registry,
			self_: self.self_,
			indent: Indent(self.indent.0 + 1),
		}
	}

	pub fn inner_self(&self, self_: TypeRef) -> Self {
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

pub trait DisplaySignature<R> {
	fn display_signature<'a>(
		&'a self,
		registry: &'a Registry<R>,
	) -> DisplayedSignature<'a, R, Self> {
		self.display_signature_with(DisplayContext {
			registry,
			self_: None,
			indent: Indent(0),
		})
	}

	fn display_signature_with<'a>(
		&'a self,
		context: DisplayContext<'a, R>,
	) -> DisplayedSignature<'a, R, Self> {
		DisplayedSignature { context, t: self }
	}

	fn fmt_signature(&self, context: &DisplayContext<R>, f: &mut fmt::Formatter) -> fmt::Result;
}

pub struct DisplayedSignature<'a, R, T: ?Sized> {
	context: DisplayContext<'a, R>,
	t: &'a T,
}

impl<'a, R: fmt::Display, T: ?Sized + DisplaySignature<R>> fmt::Display
	for DisplayedSignature<'a, R, T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.t.fmt_signature(&self.context, f)
	}
}
