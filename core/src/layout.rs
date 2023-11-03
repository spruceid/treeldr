mod literal;
mod record;
mod union;

pub use literal::LiteralLayout;

pub enum Layout {
	Never,
	Literal(LiteralLayout),
	Id,
	Record,
	Union,
	Always
}