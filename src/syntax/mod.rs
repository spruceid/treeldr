pub use ll1::{Parse, Span, Spanned};
pub mod ast;

pub type ParseError<E = std::convert::Infallible> = ll1::ParseError<ast::Token, E>;
