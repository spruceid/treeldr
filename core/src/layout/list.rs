pub mod r#unsized;
pub mod sized;

pub use r#unsized::UnsizedListLayout;
pub use sized::SizedListLayout;

pub enum ListLayout {
	Unsized(UnsizedListLayout),
	Sized(SizedListLayout)
}
