use crate::Context;
use std::fmt;

pub trait Display<V, F> {
	fn fmt(&self, context: &Context<V, F>, f: &mut fmt::Formatter) -> fmt::Result;

	fn display<'a, 'c>(
		&self,
		context: &'c Context<'a, V, F>,
	) -> DisplayWith<'a, 'c, '_, V, F, Self> {
		DisplayWith(context, self)
	}
}

pub struct DisplayWith<'a, 'c, 't, V, F, T: ?Sized>(&'c Context<'a, V, F>, &'t T);

impl<'a, 'c, 't, V, F, T: ?Sized + Display<V, F>> fmt::Display
	for DisplayWith<'a, 'c, 't, V, F, T>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.1.fmt(self.0, f)
	}
}
