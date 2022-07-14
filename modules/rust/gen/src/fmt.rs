use crate::Context;
use std::fmt;

pub trait Display<F> {
	fn fmt(&self, context: &Context<F>, f: &mut fmt::Formatter) -> fmt::Result;

	fn display<'a, 'c>(&self, context: &'c Context<'a, F>) -> DisplayWith<'a, 'c, '_, F, Self> {
		DisplayWith(context, self)
	}
}

pub struct DisplayWith<'a, 'c, 't, F, T: ?Sized>(&'c Context<'a, F>, &'t T);

impl<'a, 'c, 't, F, T: ?Sized + Display<F>> fmt::Display for DisplayWith<'a, 'c, 't, F, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.1.fmt(self.0, f)
	}
}
