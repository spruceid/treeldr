use iref::IriBuf;
use locspan::Loc;

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
	pub expr: Loc<OuterTypeExpr<F>, F>,
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

pub enum OuterTypeExpr<F> {
	Inner(InnerTypeExpr<F>),
	Union(Vec<Loc<InnerTypeExpr<F>, F>>)
}

impl<F: Clone> OuterTypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> OuterLayoutExpr<F> {
		match self {
			Self::Inner(i) => OuterLayoutExpr::Inner(i.implicit_layout_expr()),
			Self::Union(options) => OuterLayoutExpr::Union(options.iter().map(|Loc(ty_expr, loc)| {
				Loc(ty_expr.implicit_layout_expr(), loc.clone())
			}).collect()),
		}
	}
}

pub enum InnerTypeExpr<F> {
	Id(Loc<Id, F>),
	Reference(Box<Loc<Self, F>>),
	Literal(Literal)
}

impl<F: Clone> InnerTypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> InnerLayoutExpr<F> {
		match self {
			Self::Id(id) => InnerLayoutExpr::Id(id.clone()),
			Self::Reference(r) => InnerLayoutExpr::Reference(Box::new(Loc(
				r.implicit_layout_expr(),
				r.location().clone(),
			))),
			Self::Literal(lit) => InnerLayoutExpr::Literal(lit.clone())
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
	pub expr: Loc<OuterLayoutExpr<F>, F>,
	pub annotations: Vec<Loc<Annotation, F>>,
}

pub enum OuterLayoutExpr<F> {
	Inner(InnerLayoutExpr<F>),
	Union(Vec<Loc<InnerLayoutExpr<F>, F>>)
}

pub enum InnerLayoutExpr<F> {
	Id(Loc<Id, F>),
	Reference(Box<Loc<Self, F>>),
	Literal(Literal)
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Literal {
	String(String),
	RegExp(String),
}
