mod span;
mod loc;
pub mod lexing;
pub mod parsing;

pub use span::*;
pub use loc::*;
pub use lexing::{Id, Lexer};
pub use parsing::Parse;

pub struct Document {
	pub items: Vec<Loc<Item>>
}

pub enum Item {
	Type(TypeDefinition),
	Layout(LayoutDefinition)
}

pub struct TypeDefinition {
	pub id: Loc<Id>,
	pub properties: Vec<Loc<PropertyDefinition>>
}

pub struct PropertyDefinition {
	pub id: Loc<Id>,
	pub ty: Option<Loc<TypeExpr>>
}

pub struct TypeExpr {
	pub ty: Loc<Id>,
	pub args: Vec<Loc<TypeExpr>>
}

pub struct LayoutDefinition {
	pub id: Loc<Id>,
	pub fields: Vec<Loc<FieldDefinition>>
}

pub struct FieldDefinition {
	pub id: Loc<Id>,
	pub layout: Option<Loc<LayoutExpr>>,
	pub alias: Option<Loc<Id>>
}

pub struct LayoutExpr {
	pub layout: Loc<Id>,
	pub args: Vec<Loc<LayoutExpr>>
}