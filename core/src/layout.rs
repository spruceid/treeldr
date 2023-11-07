mod intersection;
mod list;
mod literal;
mod product;
pub mod sum;
mod r#union;

pub use list::{ListLayout, ListLayoutType};
pub use literal::{DataLayout, IdLayout, LiteralLayout, UnitLayout, BooleanLayout, NumberLayout, ByteStringLayout, TextStringLayout, LiteralLayoutType, DataLayoutType, UnitLayoutType, BooleanLayoutType, NumberLayoutType, ByteStringLayoutType, TextStringLayoutType};
pub use product::{ProductLayout, ProductLayoutType};
pub use sum::{SumLayout, SumLayoutType};

use crate::{Context, GetFromContext};

/// Layout type.
pub struct LayoutType;

impl<R> GetFromContext<Context<R>, R> for LayoutType {
	type Target<'c> = &'c Layout<R> where R: 'c;

	fn get_from_context<'c>(
		context: &'c crate::Context<R>,
		r: &crate::Ref<R, Self>,
	) -> Option<Self::Target<'c>> {
		context.layout(&r.0)
	}
}

/// Layout value.
pub enum Layout<R> {
	Never,
	Literal(LiteralLayout<R>),
	Product(ProductLayout<R>),
	List(ListLayout<R>),
	Sum(SumLayout<R>),
	Always,
}

impl<R> Layout<R> {
	pub fn input_count(&self) -> Option<u32> {
		match self {
			Self::Never => None,
			Self::Literal(_) => Some(1),
			Self::Product(p) => Some(p.input),
			Self::List(l) => Some(l.input_count()),
			Self::Sum(s) => Some(s.input),
			Self::Always => None
		}
	}
}