pub mod data;
pub mod id;

pub use data::DataLayout;
pub use id::IdLayout;

pub enum LiteralLayout<R> {
	Data(DataLayout<R>),
	Id(IdLayout<R>),
}
