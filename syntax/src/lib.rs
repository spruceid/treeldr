use iref::IriBuf;
use locspan::Meta;
use locspan_derive::{StrippedEq, StrippedPartialEq};

pub mod build;
pub mod lexing;
pub mod parsing;
mod peekable3;

pub use lexing::{Id, Label, Lexer};
pub use parsing::{Parse, Parser};
pub use treeldr::{layout::Primitive, vocab};

#[derive(Clone)]
pub struct Documentation<M> {
	pub items: Vec<Meta<String, M>>,
}

impl<M> Documentation<M> {
	pub fn new(items: Vec<Meta<String, M>>) -> Self {
		Self { items }
	}
}

pub struct Document<M> {
	pub bases: Vec<Meta<IriBuf, M>>,
	pub uses: Vec<Meta<Use<M>, M>>,
	pub types: Vec<Meta<TypeDefinition<M>, M>>,
	pub properties: Vec<Meta<PropertyDefinition<M>, M>>,
	pub layouts: Vec<Meta<LayoutDefinition<M>, M>>,
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

pub enum Item<M> {
	Base(Meta<IriBuf, M>),
	Use(Meta<Use<M>, M>),
	Type(Meta<TypeDefinition<M>, M>),
	Property(Meta<PropertyDefinition<M>, M>),
	Layout(Meta<LayoutDefinition<M>, M>),
}

pub struct Use<M> {
	pub iri: Meta<IriBuf, M>,
	pub prefix: Meta<Prefix, M>,
	pub doc: Option<Meta<Documentation<M>, M>>,
}

pub struct TypeDefinition<M> {
	pub id: Meta<Id, M>,
	pub description: Meta<TypeDescription<M>, M>,
	pub doc: Option<Meta<Documentation<M>, M>>,
	pub layout: Option<Meta<LayoutDescription<M>, M>>,
}

impl<M: Clone> TypeDefinition<M> {
	pub fn implicit_layout_definition(&self) -> LayoutDefinition<M> {
		let description = match &self.layout {
			Some(d) => d.clone(),
			None => Meta(
				self.description.implicit_layout_description(),
				self.description.metadata().clone(),
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

pub enum TypeDescription<M> {
	Normal(Vec<Meta<PropertyDefinition<M>, M>>),
	Alias(OuterTypeExpr<M>),
}

impl<M: Clone> TypeDescription<M> {
	pub fn implicit_layout_description(&self) -> LayoutDescription<M> {
		match self {
			Self::Normal(properties) => LayoutDescription::Normal(
				properties
					.iter()
					.map(|Meta(prop, prop_loc)| {
						Meta(prop.implicit_field_definition(), prop_loc.clone())
					})
					.collect(),
			),
			Self::Alias(expr) => LayoutDescription::Alias(expr.implicit_layout_expr()),
		}
	}
}

pub struct PropertyDefinition<M> {
	pub id: Meta<Id, M>,
	pub alias: Option<Meta<Alias, M>>,
	pub ty: Option<Meta<AnnotatedTypeExpr<M>, M>>,
	pub doc: Option<Meta<Documentation<M>, M>>,
}

impl<M: Clone> PropertyDefinition<M> {
	pub fn implicit_field_definition(&self) -> FieldDefinition<M> {
		FieldDefinition {
			id: self.id.clone(),
			layout: self
				.ty
				.as_ref()
				.map(|Meta(ty, ty_loc)| Meta(ty.implicit_layout_expr(), ty_loc.clone())),
			alias: self.alias.clone(),
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

	/// Field with a single value.
	///
	/// In can be combined with `multiple` to use the `one or many` container
	/// layout.
	Single,
}

impl Annotation {
	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"required" => Some(Self::Required),
			"multiple" => Some(Self::Multiple),
			"single" => Some(Self::Single),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Required => "required",
			Self::Multiple => "multiple",
			Self::Single => "single",
		}
	}
}

/// Annotated type expression.
pub struct AnnotatedTypeExpr<M> {
	pub expr: Meta<OuterTypeExpr<M>, M>,
	pub annotations: Vec<Meta<Annotation, M>>,
}

impl<M: Clone> AnnotatedTypeExpr<M> {
	pub fn implicit_layout_expr(&self) -> AnnotatedLayoutExpr<M> {
		AnnotatedLayoutExpr {
			expr: Meta(
				self.expr.implicit_layout_expr(),
				self.expr.metadata().clone(),
			),
			annotations: self.annotations.clone(),
		}
	}
}

#[derive(Clone)]
pub enum OuterTypeExpr<M> {
	Inner(NamedInnerTypeExpr<M>),
	Union(Label, Vec<Meta<NamedInnerTypeExpr<M>, M>>),
	Intersection(Label, Vec<Meta<NamedInnerTypeExpr<M>, M>>),
}

impl<M: Clone> OuterTypeExpr<M> {
	pub fn implicit_layout_expr(&self) -> OuterLayoutExpr<M> {
		match self {
			Self::Inner(i) => OuterLayoutExpr::Inner(i.implicit_layout_expr()),
			Self::Union(label, options) => OuterLayoutExpr::Union(
				*label,
				options
					.iter()
					.map(|Meta(ty_expr, loc)| Meta(ty_expr.implicit_layout_expr(), loc.clone()))
					.collect(),
			),
			Self::Intersection(label, types) => OuterLayoutExpr::Intersection(
				*label,
				types
					.iter()
					.map(|Meta(ty_expr, loc)| Meta(ty_expr.implicit_layout_expr(), loc.clone()))
					.collect(),
			),
		}
	}
}

#[derive(Clone)]
pub struct NamedInnerTypeExpr<M> {
	pub expr: Meta<InnerTypeExpr<M>, M>,
	pub layout: NamedInnerTypeExprLayout<M>,
}

#[derive(Clone)]
pub enum NamedInnerTypeExprLayout<M> {
	Implicit(Option<Meta<Alias, M>>),
	Explicit(Meta<NamedInnerLayoutExpr<M>, M>),
}

impl<M: Clone> NamedInnerTypeExpr<M> {
	pub fn as_id(&self) -> Option<Meta<Id, M>> {
		self.expr.as_id()
	}

	pub fn implicit_layout_expr(&self) -> NamedInnerLayoutExpr<M> {
		match &self.layout {
			NamedInnerTypeExprLayout::Implicit(name) => NamedInnerLayoutExpr {
				expr: Meta(
					self.expr.implicit_layout_expr(),
					self.expr.metadata().clone(),
				),
				name: name.clone(),
			},
			NamedInnerTypeExprLayout::Explicit(expr) => expr.value().clone(),
		}
	}
}

#[derive(Clone)]
pub enum InnerTypeExpr<M> {
	Id(Meta<Id, M>),
	Reference(Box<Meta<Self, M>>),
	Literal(Literal),
	PropertyRestriction(TypeRestrictedProperty<M>),
	List(Label, Box<Meta<OuterTypeExpr<M>, M>>),
	Outer(Box<Meta<OuterTypeExpr<M>, M>>),
}

impl<M: Clone> InnerTypeExpr<M> {
	pub fn as_id(&self) -> Option<Meta<Id, M>> {
		match self {
			Self::Id(id) => Some(id.clone()),
			_ => None,
		}
	}

	pub fn implicit_layout_expr(&self) -> InnerLayoutExpr<M> {
		match self {
			Self::Id(id) => InnerLayoutExpr::Id(id.clone()),
			Self::Reference(r) => InnerLayoutExpr::Reference(r.clone()),
			Self::Literal(lit) => InnerLayoutExpr::Literal(lit.clone()),
			Self::PropertyRestriction(r) => {
				InnerLayoutExpr::FieldRestriction(r.implicit_layout_restricted_field())
			}
			Self::List(label, item) => InnerLayoutExpr::Array(
				*label,
				Box::new(Meta(item.implicit_layout_expr(), item.metadata().clone())),
			),
			Self::Outer(outer) => InnerLayoutExpr::Outer(Box::new(Meta(
				outer.implicit_layout_expr(),
				outer.metadata().clone(),
			))),
		}
	}
}

#[derive(Clone)]
pub struct TypeRestrictedProperty<M> {
	prop: Meta<Id, M>,
	alias: Option<Meta<Alias, M>>,
	restriction: Meta<TypePropertyRestriction<M>, M>,
}

impl<M: Clone> TypeRestrictedProperty<M> {
	pub fn implicit_layout_restricted_field(&self) -> LayoutRestrictedField<M> {
		LayoutRestrictedField {
			prop: self.prop.clone(),
			alias: self.alias.clone(),
			restriction: Meta(
				self.restriction.implicit_field_restriction(),
				self.restriction.metadata().clone(),
			),
		}
	}
}

#[derive(Clone)]
pub enum TypePropertyRangeRestriction<M> {
	Any(Box<Meta<InnerTypeExpr<M>, M>>),
	All(Box<Meta<InnerTypeExpr<M>, M>>),
}

impl<M: Clone> TypePropertyRangeRestriction<M> {
	pub fn implicit_field_range_restriction(&self) -> LayoutFieldRangeRestriction<M> {
		match self {
			Self::Any(ty) => LayoutFieldRangeRestriction::Any(Box::new(Meta(
				ty.implicit_layout_expr(),
				ty.metadata().clone(),
			))),
			Self::All(ty) => LayoutFieldRangeRestriction::All(Box::new(Meta(
				ty.implicit_layout_expr(),
				ty.metadata().clone(),
			))),
		}
	}
}

#[derive(Clone)]
pub enum TypePropertyCardinalityRestriction {
	AtLeast(u64),
	AtMost(u64),
	Exactly(u64),
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
pub enum TypePropertyRestriction<M> {
	Range(TypePropertyRangeRestriction<M>),
	Cardinality(TypePropertyCardinalityRestriction),
}

impl<M: Clone> TypePropertyRestriction<M> {
	pub fn implicit_field_restriction(&self) -> LayoutFieldRestriction<M> {
		match self {
			Self::Range(r) => LayoutFieldRestriction::Range(r.implicit_field_range_restriction()),
			Self::Cardinality(c) => {
				LayoutFieldRestriction::Cardinality(c.implicit_field_cardinality_restriction())
			}
		}
	}
}

pub struct LayoutDefinition<M> {
	pub id: Meta<Id, M>,
	pub ty_id: Option<Meta<Id, M>>,
	pub description: Meta<LayoutDescription<M>, M>,
	pub doc: Option<Meta<Documentation<M>, M>>,
}

#[derive(Clone)]
pub enum LayoutDescription<M> {
	Normal(Vec<Meta<FieldDefinition<M>, M>>),
	Alias(OuterLayoutExpr<M>),
}

#[derive(Clone)]
pub struct FieldDefinition<M> {
	pub id: Meta<Id, M>,
	pub layout: Option<Meta<AnnotatedLayoutExpr<M>, M>>,
	pub alias: Option<Meta<Alias, M>>,
	pub doc: Option<Meta<Documentation<M>, M>>,
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
pub struct AnnotatedLayoutExpr<M> {
	pub expr: Meta<OuterLayoutExpr<M>, M>,
	pub annotations: Vec<Meta<Annotation, M>>,
}

#[derive(Clone)]
pub enum OuterLayoutExpr<M> {
	Inner(NamedInnerLayoutExpr<M>),
	Union(Label, Vec<Meta<NamedInnerLayoutExpr<M>, M>>),
	Intersection(Label, Vec<Meta<NamedInnerLayoutExpr<M>, M>>),
}

impl<M> OuterLayoutExpr<M> {
	pub fn into_restriction(self) -> Result<LayoutRestrictedField<M>, Self> {
		match self {
			Self::Inner(i) => i.into_restriction().map_err(Self::Inner),
			other => Err(other),
		}
	}
}

#[derive(Clone)]
pub struct NamedInnerLayoutExpr<M> {
	pub expr: Meta<InnerLayoutExpr<M>, M>,
	pub name: Option<Meta<Alias, M>>,
}

impl<M> NamedInnerLayoutExpr<M> {
	#[allow(clippy::type_complexity)]
	pub fn into_parts(self) -> (Meta<InnerLayoutExpr<M>, M>, Option<Meta<Alias, M>>) {
		(self.expr, self.name)
	}

	pub fn as_id(&self) -> Option<Meta<Id, M>>
	where
		M: Clone,
	{
		self.expr.as_id()
	}

	pub fn into_restriction(self) -> Result<LayoutRestrictedField<M>, Self> {
		if self.name.is_some() {
			Err(self)
		} else {
			let Meta(e, loc) = self.expr;
			e.into_restriction().map_err(|other| Self {
				expr: Meta(other, loc),
				name: None,
			})
		}
	}
}

#[derive(Clone)]
pub enum InnerLayoutExpr<M> {
	Id(Meta<Id, M>),
	Primitive(Primitive),
	Reference(Box<Meta<InnerTypeExpr<M>, M>>),
	Literal(Literal),
	FieldRestriction(LayoutRestrictedField<M>),
	Array(Label, Box<Meta<OuterLayoutExpr<M>, M>>),
	Outer(Box<Meta<OuterLayoutExpr<M>, M>>),
}

impl<M> InnerLayoutExpr<M> {
	pub fn is_namable(&self) -> bool {
		!matches!(self, Self::Id(_))
	}

	pub fn as_id(&self) -> Option<Meta<Id, M>>
	where
		M: Clone,
	{
		match self {
			Self::Id(id) => Some(id.clone()),
			_ => None,
		}
	}

	pub fn into_restriction(self) -> Result<LayoutRestrictedField<M>, Self> {
		match self {
			Self::FieldRestriction(r) => Ok(r),
			other => Err(other),
		}
	}
}

#[derive(Clone)]
pub struct LayoutRestrictedField<M> {
	prop: Meta<Id, M>,
	alias: Option<Meta<Alias, M>>,
	restriction: Meta<LayoutFieldRestriction<M>, M>,
}

#[derive(Clone)]
pub enum LayoutFieldRangeRestriction<M> {
	Any(Box<Meta<InnerLayoutExpr<M>, M>>),
	All(Box<Meta<InnerLayoutExpr<M>, M>>),
}

#[derive(Clone, PartialEq, Eq, StrippedPartialEq, StrippedEq)]
pub enum LayoutFieldCardinalityRestriction {
	AtLeast(u64),
	AtMost(u64),
	Exactly(u64),
}

#[derive(Clone)]
pub enum LayoutFieldRestriction<M> {
	/// Affects the item of a collection layout?
	Range(LayoutFieldRangeRestriction<M>),

	/// Affects the size of a collection layout.
	Cardinality(LayoutFieldCardinalityRestriction),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Literal {
	String(String),
	RegExp(String),
}
