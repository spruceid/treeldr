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
	pub description: Loc<TypeDescription<F>, F>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

impl<F: Clone> TypeDefinition<F> {
	pub fn implicit_layout_definition(&self) -> LayoutDefinition<F> {
		LayoutDefinition {
			id: self.id.clone(),
			ty_id: self.id.clone(),
			description: Loc(
				self.description.implicit_layout_description(),
				self.description.location().clone(),
			),
			doc: self.doc.clone(),
		}
	}
}

pub enum TypeDescription<F> {
	Normal(Vec<Loc<PropertyDefinition<F>, F>>),
	Alias(OuterTypeExpr<F>),
}

impl<F: Clone> TypeDescription<F> {
	pub fn implicit_layout_description(&self) -> LayoutDescription<F> {
		match self {
			Self::Normal(properties) => LayoutDescription::Normal(
				properties
					.iter()
					.map(|Loc(prop, prop_loc)| {
						Loc(prop.implicit_field_definition(), prop_loc.clone())
					})
					.collect(),
			),
			Self::Alias(expr) => LayoutDescription::Alias(expr.implicit_layout_expr()),
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
	Inner(NamedInnerTypeExpr<F>),
	Union(Vec<Loc<NamedInnerTypeExpr<F>, F>>),
	Intersection(Vec<Loc<NamedInnerTypeExpr<F>, F>>),
}

impl<F: Clone> OuterTypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> OuterLayoutExpr<F> {
		match self {
			Self::Inner(i) => OuterLayoutExpr::Inner(i.implicit_layout_expr()),
			Self::Union(options) => OuterLayoutExpr::Union(
				options
					.iter()
					.map(|Loc(ty_expr, loc)| Loc(ty_expr.implicit_layout_expr(), loc.clone()))
					.collect(),
			),
			Self::Intersection(types) => OuterLayoutExpr::Intersection(
				types
					.iter()
					.map(|Loc(ty_expr, loc)| Loc(ty_expr.implicit_layout_expr(), loc.clone()))
					.collect(),
			),
		}
	}
}

pub struct NamedInnerTypeExpr<F> {
	pub expr: Loc<InnerTypeExpr<F>, F>,
	pub name: Option<Loc<Alias, F>>,
}

impl<F: Clone> NamedInnerTypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> NamedInnerLayoutExpr<F> {
		NamedInnerLayoutExpr {
			expr: Loc(
				self.expr.implicit_layout_expr(),
				self.expr.location().clone(),
			),
			name: self.name.clone(),
		}
	}
}

pub enum InnerTypeExpr<F> {
	Id(Loc<Id, F>),
	Reference(Box<Loc<Self, F>>),
	Literal(Literal),
	PropertyRestriction(TypePropertyRestriction<F>)
}

impl<F: Clone> InnerTypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> InnerLayoutExpr<F> {
		match self {
			Self::Id(id) => InnerLayoutExpr::Id(id.clone()),
			Self::Reference(r) => InnerLayoutExpr::Reference(Box::new(Loc(
				r.implicit_layout_expr(),
				r.location().clone(),
			))),
			Self::Literal(lit) => InnerLayoutExpr::Literal(lit.clone()),
			Self::PropertyRestriction(r) => InnerLayoutExpr::FieldRestriction(r.implicit_field_restriction())
		}
	}
}

pub enum TypePropertyRangeRestriction<F> {
	Any(Box<Loc<InnerTypeExpr<F>, F>>),
	All(Box<Loc<InnerTypeExpr<F>, F>>),
}

impl<F: Clone> TypePropertyRangeRestriction<F> {
	pub fn implicit_field_range_restriction(&self) -> LayoutFieldRangeRestriction<F> {
		match self {
			Self::Any(ty) => LayoutFieldRangeRestriction::Any(Box::new(Loc(
				ty.implicit_layout_expr(),
				ty.location().clone()
			))),
			Self::All(ty) => LayoutFieldRangeRestriction::All(Box::new(Loc(
				ty.implicit_layout_expr(),
				ty.location().clone()
			)))
		}
	}
}

pub enum TypePropertyCardinalityRestriction {
	AtLeast(u32),
	AtMost(u32),
	Exactly(u32)
}

impl TypePropertyCardinalityRestriction {
	pub fn implicit_field_cardinality_restriction(&self) -> LayoutFieldCardinalityRestriction {
		match self {
			Self::AtLeast(n) => LayoutFieldCardinalityRestriction::AtLeast(*n),
			Self::AtMost(n) => LayoutFieldCardinalityRestriction::AtMost(*n),
			Self::Exactly(n) => LayoutFieldCardinalityRestriction::Exactly(*n)
		}
	}
}

pub enum TypePropertyRestriction<F> {
	Range(TypePropertyRangeRestriction<F>),
	Cardinality(TypePropertyCardinalityRestriction)
}

impl<F: Clone> TypePropertyRestriction<F> {
	pub fn implicit_field_restriction(&self) -> LayoutFieldRestriction<F> {
		match self {
			Self::Range(r) => LayoutFieldRestriction::Range(r.implicit_field_range_restriction()),
			Self::Cardinality(c) => LayoutFieldRestriction::Cardinality(c.implicit_field_cardinality_restriction())
		}
	}
}

pub struct LayoutDefinition<F> {
	pub id: Loc<Id, F>,
	pub ty_id: Loc<Id, F>,
	pub description: Loc<LayoutDescription<F>, F>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

pub enum LayoutDescription<F> {
	Normal(Vec<Loc<FieldDefinition<F>, F>>),
	Alias(OuterLayoutExpr<F>),
}

pub struct FieldDefinition<F> {
	pub id: Loc<Id, F>,
	pub layout: Option<Loc<AnnotatedLayoutExpr<F>, F>>,
	pub alias: Option<Loc<Alias, F>>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

#[derive(Clone, Debug)]
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
	Inner(NamedInnerLayoutExpr<F>),
	Union(Vec<Loc<NamedInnerLayoutExpr<F>, F>>),
	Intersection(Vec<Loc<NamedInnerLayoutExpr<F>, F>>),
}

pub struct NamedInnerLayoutExpr<F> {
	pub expr: Loc<InnerLayoutExpr<F>, F>,
	pub name: Option<Loc<Alias, F>>,
}

pub enum InnerLayoutExpr<F> {
	Id(Loc<Id, F>),
	Reference(Box<Loc<Self, F>>),
	Literal(Literal),
	FieldRestriction(LayoutFieldRestriction<F>)
}

impl<F> InnerLayoutExpr<F> {
	fn is_namable(&self) -> bool {
		!matches!(self, Self::Id(_))
	}
}

pub enum LayoutFieldRangeRestriction<F> {
	Any(Box<Loc<InnerLayoutExpr<F>, F>>),
	All(Box<Loc<InnerLayoutExpr<F>, F>>),
}

pub enum LayoutFieldCardinalityRestriction {
	AtLeast(u32),
	AtMost(u32),
	Exactly(u32)
}

pub enum LayoutFieldRestriction<F> {
	/// Affects the item of a collection layout?
	Range(LayoutFieldRangeRestriction<F>),

	/// Affects the size of a collection layout.
	Cardinality(LayoutFieldCardinalityRestriction)
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Literal {
	String(String),
	RegExp(String),
}