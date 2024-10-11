mod rdf;
pub use rdf::*;

// use crate::Literal;

// pub mod typed;
// pub use typed::TypedPattern;

pub mod substitution;
pub use substitution::Substitution;

// /// Untyped tree value.
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub enum Pattern<R> {
// 	Bind,
// 	Resource(R),
// 	Literal(Literal),
// 	Map(MapPattern<R>),
// 	List(ListPattern<R>)
// }

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct ListPattern<R> {
// 	pub prefix: Vec<Pattern<R>>,
// 	pub suffix: Option<Vec<Pattern<R>>>
// }

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct MapPattern<R> {
// 	pub entries: BTreeMap<Pattern<R>, Pattern<R>>,
// 	pub ellipsis: bool
// }
