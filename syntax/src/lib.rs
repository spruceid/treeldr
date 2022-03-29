use iref::IriBuf;
use locspan::{
	Loc,
	Location
};

pub use treeldr_vocab as vocab;

pub mod build;
pub mod lexing;
pub mod parsing;
mod peekable3;

pub mod reporting;

pub use build::Build;
pub use lexing::{Id, Lexer};
pub use parsing::Parse;

#[derive(Clone)]
pub struct Documentation<F> {
	pub items: Vec<Loc<String, F>>,
}

impl<F> Documentation<F> {
	pub fn new(items: Vec<Loc<String, F>>) -> Self {
		Self { items }
	}
}

pub struct Document<F> {
	pub bases: Vec<Loc<IriBuf, F>>,
	pub uses: Vec<Loc<Use<F>, F>>,
	pub types: Vec<Loc<TypeDefinition<F>, F>>,
	pub layouts: Vec<Loc<LayoutDefinition<F>, F>>,
}

pub struct Prefix(String);

impl Prefix {
	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

pub enum Item<F> {
	Base(Loc<IriBuf, F>),
	Use(Loc<Use<F>, F>),
	Type(Loc<TypeDefinition<F>, F>),
	Layout(Loc<LayoutDefinition<F>, F>),
}

pub struct Use<F> {
	pub iri: Loc<IriBuf, F>,
	pub prefix: Loc<Prefix, F>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

pub struct TypeDefinition<F> {
	pub id: Loc<Id, F>,
	pub properties: Loc<Vec<Loc<PropertyDefinition<F>, F>>, F>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

impl<F: Clone> TypeDefinition<F> {
	pub fn implicit_layout_definition(&self) -> LayoutDefinition<F> {
		LayoutDefinition {
			id: self.id.clone(),
			ty_id: self.id.clone(),
			fields: Loc(
				self.properties
					.iter()
					.map(|Loc(prop, prop_loc)| {
						Loc(prop.implicit_field_definition(), prop_loc.clone())
					})
					.collect(),
				self.properties.location().clone(),
			),
			doc: self.doc.clone(),
		}
	}
}

pub struct PropertyDefinition<F> {
	pub id: Loc<Id, F>,
	pub ty: Option<Loc<AnnotatedTypeExpr<F>, F>>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

impl<F: Clone> PropertyDefinition<F> {
	pub fn implicit_field_definition(&self) -> FieldDefinition<F> {
		FieldDefinition {
			id: self.id.clone(),
			layout: self
				.ty
				.as_ref()
				.map(|Loc(ty, ty_loc)| Loc(ty.implicit_layout_expr(), ty_loc.clone())),
			alias: None,
			doc: self.doc.clone(),
		}
	}
}

/// Type annotation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Annotation {
	/// Required field.
	Required,

	/// Field with multiple values.
	Multiple,
}

impl Annotation {
	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"required" => Some(Self::Required),
			"multiple" => Some(Self::Multiple),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Required => "required",
			Self::Multiple => "multiple",
		}
	}
}

/// Annotated type expression.
pub struct AnnotatedTypeExpr<F> {
	pub expr: Loc<TypeExpr<F>, F>,
	pub annotations: Vec<Loc<Annotation, F>>,
}

impl<F: Clone> AnnotatedTypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> AnnotatedLayoutExpr<F> {
		AnnotatedLayoutExpr {
			expr: Loc(
				self.expr.implicit_layout_expr(),
				self.expr.location().clone(),
			),
			annotations: self.annotations.clone(),
		}
	}
}

pub enum TypeExpr<F> {
	Id(Loc<Id, F>),
	Reference(Box<Loc<TypeExpr<F>, F>>),
	Literal(Literal<F>)
}

impl<F: Clone> TypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> LayoutExpr<F> {
		match self {
			Self::Id(id) => LayoutExpr::Id(id.clone()),
			Self::Reference(r) => LayoutExpr::Reference(Box::new(Loc(
				r.implicit_layout_expr(),
				r.location().clone(),
			))),
			Self::Literal(lit) => LayoutExpr::Literal(lit.clone())
		}
	}
}

pub struct LayoutDefinition<F> {
	pub id: Loc<Id, F>,
	pub ty_id: Loc<Id, F>,
	pub fields: Loc<Vec<Loc<FieldDefinition<F>, F>>, F>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

pub struct FieldDefinition<F> {
	pub id: Loc<Id, F>,
	pub layout: Option<Loc<AnnotatedLayoutExpr<F>, F>>,
	pub alias: Option<Loc<Alias, F>>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

pub struct Alias(String);

impl Alias {
	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

/// Annotated layout expression.
pub struct AnnotatedLayoutExpr<F> {
	pub expr: Loc<LayoutExpr<F>, F>,
	pub annotations: Vec<Loc<Annotation, F>>,
}

pub enum LayoutExpr<F> {
	Id(Loc<Id, F>),
	Reference(Box<Loc<LayoutExpr<F>, F>>),
	Literal(Literal<F>)
}

#[derive(Clone)]
pub enum Literal<F> {
	String(String),
	RegExp(RegExp<F>)
}

impl<F> locspan::Strip for Literal<F> {
	type Stripped = stripped::Literal;

	fn strip(self) -> Self::Stripped {
		match self {
			Self::String(s) => stripped::Literal::String(s),
			Self::RegExp(e) => stripped::Literal::RegExp(e.strip())
		}
	}
}

#[derive(Clone)]
pub enum RegExp<F> {
	Sub(SubRegExp<F>),
	Full(FullRegExp<F>)
}

impl<F> locspan::Strip for RegExp<F> {
	type Stripped = stripped::RegExp;

	fn strip(self) -> Self::Stripped {
		match self {
			Self::Sub(e) => stripped::RegExp::Sub(e.strip()),
			Self::Full(e) => stripped::RegExp::Full(e.strip())
		}
	}
}

#[derive(Clone)]
pub enum SubRegExp<F> {
	Any,
	Set(CharSet<F>),
	Sequence(Vec<Loc<RegExp<F>, F>>)
}

impl<F> locspan::Strip for SubRegExp<F> {
	type Stripped = stripped::SubRegExp;

	fn strip(self) -> Self::Stripped {
		match self {
			Self::Any => stripped::SubRegExp::Any,
			Self::Set(charset) => stripped::SubRegExp::Set(charset.strip()),
			Self::Sequence(seq) => stripped::SubRegExp::Sequence(seq.strip())
		}
	}
}

#[derive(Clone)]
pub enum FullRegExp<F> {
	Sub(SubRegExp<F>),
	Optional(Box<Loc<RegExp<F>, F>>, Location<F>),
	Star(Box<Loc<RegExp<F>, F>>, Location<F>),
	Plus(Box<Loc<RegExp<F>, F>>, Location<F>),
	AtLeast(Box<Loc<RegExp<F>, F>>, Loc<u32, F>),
	AtMost(Box<Loc<RegExp<F>, F>>, Loc<u32, F>),
	Bounded(Box<Loc<RegExp<F>, F>>, Loc<u32, F>, Loc<u32, F>),
	Union(Vec<Loc<RegExp<F>, F>>)
}

impl<F> locspan::Strip for FullRegExp<F> {
	type Stripped = stripped::FullRegExp;

	fn strip(self) -> Self::Stripped {
		match self {
			Self::Sub(e) => stripped::FullRegExp::Sub(e.strip()),
			Self::Optional(e, _) => stripped::FullRegExp::Optional(Box::new(e.strip())),
			Self::Star(e, _) => stripped::FullRegExp::Star(Box::new(e.strip())),
			Self::Plus(e, _) => stripped::FullRegExp::Plus(Box::new(e.strip())),
			Self::AtLeast(e, min) => stripped::FullRegExp::AtLeast(Box::new(e.strip()), min.into_value()),
			Self::AtMost(e, max) => stripped::FullRegExp::AtMost(Box::new(e.strip()), max.into_value()),
			Self::Bounded(e, min, max) => stripped::FullRegExp::Bounded(Box::new(e.strip()), min.into_value(), max.into_value()),
			Self::Union(items) => stripped::FullRegExp::Union(items.strip())
		}
	}
}

#[derive(Clone)]
pub struct CharSet<F> {
	pub negate: Loc<bool, F>,
	pub chars: Loc<String, F>
}

impl<F> locspan::Strip for CharSet<F> {
	type Stripped = stripped::CharSet;

	fn strip(self) -> Self::Stripped {
		stripped::CharSet {
			negate: self.negate.into_value(),
			chars: self.chars.into_value()
		}
	}
}

pub mod stripped {
	#[derive(Clone, PartialEq, Eq, Hash)]
	pub enum Literal {
		String(String),
		RegExp(RegExp)
	}

	#[derive(Clone, PartialEq, Eq, Hash)]
	pub enum RegExp {
		Sub(SubRegExp),
		Full(FullRegExp)
	}

	#[derive(Clone, PartialEq, Eq, Hash)]
	pub enum SubRegExp {
		Any,
		Set(CharSet),
		Sequence(Vec<RegExp>)
	}

	#[derive(Clone, PartialEq, Eq, Hash)]
	pub enum FullRegExp {
		Sub(SubRegExp),
		Optional(Box<RegExp>),
		Star(Box<RegExp>),
		Plus(Box<RegExp>),
		AtLeast(Box<RegExp>, u32),
		AtMost(Box<RegExp>, u32),
		Bounded(Box<RegExp>, u32, u32),
		Union(Vec<RegExp>)
	}

	#[derive(Clone, PartialEq, Eq, Hash)]
	pub struct CharSet {
		pub negate: bool,
		pub chars: String
	}
}