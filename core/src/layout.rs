mod intersection;
mod list;
mod literal;
mod product;
pub mod sum;
mod r#union;

pub use list::{ListLayout, ListLayoutType};
pub use literal::{DataLayout, IdLayout, LiteralLayout};
pub use product::ProductLayout;
pub use sum::SumLayout;

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
