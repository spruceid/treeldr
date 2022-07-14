use iref::IriBuf;
use locspan::{Meta, Loc};

pub mod build;
pub mod lexing;
pub mod parsing;
mod peekable3;

pub use lexing::{Id, Label, Lexer};
pub use parsing::Parse;
pub use treeldr::{layout::Primitive, vocab};

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
	pub properties: Vec<Loc<PropertyDefinition<F>, F>>,
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
	Property(Loc<PropertyDefinition<F>, F>),
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
	pub layout: Option<Loc<LayoutDescription<F>, F>>,
}

impl<F: Clone> TypeDefinition<F> {
	pub fn implicit_layout_definition(&self) -> LayoutDefinition<F> {
		let description = match &self.layout {
			Some(d) => d.clone(),
			None => Loc(
				self.description.implicit_layout_description(),
				self.description.location().clone(),
			),
		};

		LayoutDefinition {
			id: self.id.clone(),
			ty_id: Some(self.id.clone()),
			description,
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
					.map(|Meta(prop, prop_loc)| {
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
	pub alias: Option<Loc<Alias, F>>,
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
				.map(|Meta(ty, ty_loc)| Loc(ty.implicit_layout_expr(), ty_loc.clone())),
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

#[derive(Clone)]
pub enum OuterTypeExpr<F> {
	Inner(NamedInnerTypeExpr<F>),
	Union(Label, Vec<Loc<NamedInnerTypeExpr<F>, F>>),
	Intersection(Label, Vec<Loc<NamedInnerTypeExpr<F>, F>>),
}

impl<F: Clone> OuterTypeExpr<F> {
	pub fn implicit_layout_expr(&self) -> OuterLayoutExpr<F> {
		match self {
			Self::Inner(i) => OuterLayoutExpr::Inner(i.implicit_layout_expr()),
			Self::Union(label, options) => OuterLayoutExpr::Union(
				*label,
				options
					.iter()
					.map(|Meta(ty_expr, loc)| Loc(ty_expr.implicit_layout_expr(), loc.clone()))
					.collect(),
			),
			Self::Intersection(label, types) => OuterLayoutExpr::Intersection(
				*label,
				types
					.iter()
					.map(|Meta(ty_expr, loc)| Loc(ty_expr.implicit_layout_expr(), loc.clone()))
					.collect(),
			),
		}
	}
}

#[derive(Clone)]
pub struct NamedInnerTypeExpr<F> {
	pub expr: Loc<InnerTypeExpr<F>, F>,
	pub layout: NamedInnerTypeExprLayout<F>,
}

#[derive(Clone)]
pub enum NamedInnerTypeExprLayout<F> {
	Implicit(Option<Loc<Alias, F>>),
	Explicit(Loc<NamedInnerLayoutExpr<F>, F>),
}

impl<F: Clone> NamedInnerTypeExpr<F> {
	pub fn as_id(&self) -> Option<Loc<Id, F>> {
		self.expr.as_id()
	}

	pub fn implicit_layout_expr(&self) -> NamedInnerLayoutExpr<F> {
		match &self.layout {
			NamedInnerTypeExprLayout::Implicit(name) => NamedInnerLayoutExpr {
				expr: Loc(
					self.expr.implicit_layout_expr(),
					self.expr.location().clone(),
				),
				name: name.clone(),
			},
			NamedInnerTypeExprLayout::Explicit(expr) => expr.value().clone(),
		}
	}
}

#[derive(Clone)]
pub enum InnerTypeExpr<F> {
	Id(Loc<Id, F>),
	Reference(Box<Loc<Self, F>>),
	Literal(Literal),
	PropertyRestriction(TypeRestrictedProperty<F>),
	List(Label, Box<Loc<OuterTypeExpr<F>, F>>),
	Outer(Box<Loc<OuterTypeExpr<F>, F>>),
}

impl<F: Clone> InnerTypeExpr<F> {
	pub fn as_id(&self) -> Option<Loc<Id, F>> {
		match self {
			Self::Id(id) => Some(id.clone()),
			_ => None,
		}
	}

	pub fn implicit_layout_expr(&self) -> InnerLayoutExpr<F> {
		match self {
			Self::Id(id) => InnerLayoutExpr::Id(id.clone()),
			Self::Reference(r) => InnerLayoutExpr::Reference(r.clone()),
			Self::Literal(lit) => InnerLayoutExpr::Literal(lit.clone()),
			Self::PropertyRestriction(r) => {
				InnerLayoutExpr::FieldRestriction(r.implicit_layout_restricted_field())
			}
			Self::List(label, item) => InnerLayoutExpr::Array(
				*label,
				Box::new(Loc(item.implicit_layout_expr(), item.location().clone())),
			),
			Self::Outer(outer) => InnerLayoutExpr::Outer(Box::new(Loc(
				outer.implicit_layout_expr(),
				outer.location().clone(),
			))),
		}
	}
}

#[derive(Clone)]
pub struct TypeRestrictedProperty<F> {
	prop: Loc<Id, F>,
	alias: Option<Loc<Alias, F>>,
	restriction: Loc<TypePropertyRestriction<F>, F>,
}

impl<F: Clone> TypeRestrictedProperty<F> {
	pub fn implicit_layout_restricted_field(&self) -> LayoutRestrictedField<F> {
		LayoutRestrictedField {
			prop: self.prop.clone(),
			alias: self.alias.clone(),
			restriction: Loc(
				self.restriction.implicit_field_restriction(),
				self.restriction.location().clone(),
			),
		}
	}
}

#[derive(Clone)]
pub enum TypePropertyRangeRestriction<F> {
	Any(Box<Loc<InnerTypeExpr<F>, F>>),
	All(Box<Loc<InnerTypeExpr<F>, F>>),
}

impl<F: Clone> TypePropertyRangeRestriction<F> {
	pub fn implicit_field_range_restriction(&self) -> LayoutFieldRangeRestriction<F> {
		match self {
			Self::Any(ty) => LayoutFieldRangeRestriction::Any(Box::new(Loc(
				ty.implicit_layout_expr(),
				ty.location().clone(),
			))),
			Self::All(ty) => LayoutFieldRangeRestriction::All(Box::new(Loc(
				ty.implicit_layout_expr(),
				ty.location().clone(),
			))),
		}
	}
}

#[derive(Clone)]
pub enum TypePropertyCardinalityRestriction {
	AtLeast(u32),
	AtMost(u32),
	Exactly(u32),
}

impl TypePropertyCardinalityRestriction {
	pub fn implicit_field_cardinality_restriction(&self) -> LayoutFieldCardinalityRestriction {
		match self {
			Self::AtLeast(n) => LayoutFieldCardinalityRestriction::AtLeast(*n),
			Self::AtMost(n) => LayoutFieldCardinalityRestriction::AtMost(*n),
			Self::Exactly(n) => LayoutFieldCardinalityRestriction::Exactly(*n),
		}
	}
}

#[derive(Clone)]
pub enum TypePropertyRestriction<F> {
	Range(TypePropertyRangeRestriction<F>),
	Cardinality(TypePropertyCardinalityRestriction),
}

impl<F: Clone> TypePropertyRestriction<F> {
	pub fn implicit_field_restriction(&self) -> LayoutFieldRestriction<F> {
		match self {
			Self::Range(r) => LayoutFieldRestriction::Range(r.implicit_field_range_restriction()),
			Self::Cardinality(c) => {
				LayoutFieldRestriction::Cardinality(c.implicit_field_cardinality_restriction())
			}
		}
	}
}

pub struct LayoutDefinition<F> {
	pub id: Loc<Id, F>,
	pub ty_id: Option<Loc<Id, F>>,
	pub description: Loc<LayoutDescription<F>, F>,
	pub doc: Option<Loc<Documentation<F>, F>>,
}

#[derive(Clone)]
pub enum LayoutDescription<F> {
	Normal(Vec<Loc<FieldDefinition<F>, F>>),
	Alias(OuterLayoutExpr<F>),
}

#[derive(Clone)]
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
#[derive(Clone)]
pub struct AnnotatedLayoutExpr<F> {
	pub expr: Loc<OuterLayoutExpr<F>, F>,
	pub annotations: Vec<Loc<Annotation, F>>,
}

#[derive(Clone)]
pub enum OuterLayoutExpr<F> {
	Inner(NamedInnerLayoutExpr<F>),
	Union(Label, Vec<Loc<NamedInnerLayoutExpr<F>, F>>),
	Intersection(Label, Vec<Loc<NamedInnerLayoutExpr<F>, F>>),
}

impl<F> OuterLayoutExpr<F> {
	pub fn into_restriction(self) -> Result<LayoutRestrictedField<F>, Self> {
		match self {
			Self::Inner(i) => i.into_restriction().map_err(Self::Inner),
			other => Err(other),
		}
	}
}

#[derive(Clone)]
pub struct NamedInnerLayoutExpr<F> {
	pub expr: Loc<InnerLayoutExpr<F>, F>,
	pub name: Option<Loc<Alias, F>>,
}

impl<F> NamedInnerLayoutExpr<F> {
	#[allow(clippy::type_complexity)]
	pub fn into_parts(self) -> (Loc<InnerLayoutExpr<F>, F>, Option<Loc<Alias, F>>) {
		(self.expr, self.name)
	}

	pub fn as_id(&self) -> Option<Loc<Id, F>>
	where
		F: Clone,
	{
		self.expr.as_id()
	}

	pub fn into_restriction(self) -> Result<LayoutRestrictedField<F>, Self> {
		if self.name.is_some() {
			Err(self)
		} else {
			let Meta(e, loc) = self.expr;
			e.into_restriction().map_err(|other| Self {
				expr: Loc(other, loc),
				name: None,
			})
		}
	}
}

#[derive(Clone)]
pub enum InnerLayoutExpr<F> {
	Id(Loc<Id, F>),
	Primitive(Primitive),
	Reference(Box<Loc<InnerTypeExpr<F>, F>>),
	Literal(Literal),
	FieldRestriction(LayoutRestrictedField<F>),
	Array(Label, Box<Loc<OuterLayoutExpr<F>, F>>),
	Outer(Box<Loc<OuterLayoutExpr<F>, F>>),
}

impl<F> InnerLayoutExpr<F> {
	pub fn is_namable(&self) -> bool {
		!matches!(self, Self::Id(_))
	}

	pub fn as_id(&self) -> Option<Loc<Id, F>>
	where
		F: Clone,
	{
		match self {
			Self::Id(id) => Some(id.clone()),
			_ => None,
		}
	}

	pub fn into_restriction(self) -> Result<LayoutRestrictedField<F>, Self> {
		match self {
			Self::FieldRestriction(r) => Ok(r),
			other => Err(other),
		}
	}
}

#[derive(Clone)]
pub struct LayoutRestrictedField<F> {
	prop: Loc<Id, F>,
	alias: Option<Loc<Alias, F>>,
	restriction: Loc<LayoutFieldRestriction<F>, F>,
}

#[derive(Clone)]
pub enum LayoutFieldRangeRestriction<F> {
	Any(Box<Loc<InnerLayoutExpr<F>, F>>),
	All(Box<Loc<InnerLayoutExpr<F>, F>>),
}

#[derive(Clone, PartialEq, Eq)]
pub enum LayoutFieldCardinalityRestriction {
	AtLeast(u32),
	AtMost(u32),
	Exactly(u32),
}

#[derive(Clone)]
pub enum LayoutFieldRestriction<F> {
	/// Affects the item of a collection layout?
	Range(LayoutFieldRangeRestriction<F>),

	/// Affects the size of a collection layout.
	Cardinality(LayoutFieldCardinalityRestriction),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Literal {
	String(String),
	RegExp(String),
}
