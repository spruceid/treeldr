mod list;
mod literal;
pub mod sum;
mod product;
mod r#union;
mod intersection;

pub use list::ListLayout;
pub use literal::{DataLayout, IdLayout, LiteralLayout};
pub use sum::SumLayout;
pub use product::ProductLayout;

use crate::{GetFromContext, Context};

/// Layout type.
pub struct LayoutType;

impl<R> GetFromContext<Context<R>, R> for LayoutType {
	type Target<'c> = &'c Layout<R> where R: 'c;
	
	fn get_from_context<'c>(context: &'c crate::Context<R>, r: &crate::Ref<R, Self>) -> Option<Self::Target<'c>> {
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
	Always
}