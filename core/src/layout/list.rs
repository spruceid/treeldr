pub mod r#unsized;
pub mod sized;

pub use r#unsized::UnsizedListLayout;
pub use sized::SizedListLayout;

use crate::{Format, Graph};

pub enum ListLayout<R> {
	Unsized(UnsizedListLayout<R>),
	Sized(SizedListLayout<R>)
}

pub struct ItemLayout<R> {
	/// Intros.
	pub intro: u32,

	/// Format.
	pub format: Format<R>,

	/// Graph.
	pub graph: Graph<R>,
}