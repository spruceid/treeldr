pub mod lexing;
mod loc;
pub mod parsing;
mod span;

pub use lexing::{Id, Lexer};
pub use loc::*;
pub use parsing::Parse;
pub use span::*;

pub struct Documentation {
	pub items: Vec<Loc<String>>
}

impl Documentation {
	pub fn new(items: Vec<Loc<String>>) -> Self {
		Self {
			items
		}
	}
}

pub struct Document {
	pub items: Vec<Loc<Item>>,
}

pub enum Item {
	Type(Loc<TypeDefinition>),
	Layout(Loc<LayoutDefinition>),
}

pub struct TypeDefinition {
	pub id: Loc<Id>,
	pub properties: Vec<Loc<PropertyDefinition>>,
	pub doc: Documentation
}

pub struct PropertyDefinition {
	pub id: Loc<Id>,
	pub ty: Option<Loc<TypeExpr>>,
	pub doc: Documentation
}

pub struct TypeExpr {
	pub ty: Loc<Id>,
	pub args: Vec<Loc<TypeExpr>>,
}

pub struct LayoutDefinition {
	pub id: Loc<Id>,
	pub ty_id: Loc<Id>,
	pub fields: Vec<Loc<FieldDefinition>>,
	pub doc: Documentation
}

pub struct FieldDefinition {
	pub id: Loc<Id>,
	pub layout: Loc<LayoutExpr>,
	pub alias: Option<Loc<Alias>>,
	pub doc: Documentation
}

pub struct Alias(String);

impl Alias {
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

pub struct LayoutExpr {
	pub layout: Loc<Id>,
	pub args: Vec<Loc<LayoutExpr>>,
}
