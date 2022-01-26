pub use locspan::Span;
use crate::source;

mod peekable3;
pub mod lexing;
pub mod parsing;

pub use lexing::{Id, Lexer};
pub use parsing::Parse;

pub type Loc<T> = locspan::Loc<T, source::Id>;
pub type Location = locspan::Location<source::Id>;

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
	pub doc: Loc<Documentation>
}

pub struct PropertyDefinition {
	pub id: Loc<Id>,
	pub ty: Option<Loc<TypeExpr>>,
	pub doc: Loc<Documentation>
}

/// Type annotation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Annotation {
	/// Required field.
	Required
}

/// Annotated type expression.
pub struct AnnotatedTypeExpr {
	expr: Loc<TypeExpr>,
	annotations: Vec<Loc<Annotation>>
}

pub struct TypeExpr {
	pub ty: Loc<Id>,
	pub args: Vec<Loc<TypeExpr>>,
}

pub struct LayoutDefinition {
	pub id: Loc<Id>,
	pub ty_id: Loc<Id>,
	pub fields: Vec<Loc<FieldDefinition>>,
	pub doc: Loc<Documentation>
}

pub struct FieldDefinition {
	pub id: Loc<Id>,
	pub layout: Loc<LayoutExpr>,
	pub alias: Option<Loc<Alias>>,
	pub doc: Loc<Documentation>
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
