use core::fmt;
use iref::IriBuf;
use rdf_types::{
	generator,
	interpretation::{IriInterpretationMut, LiteralInterpretationMut},
	InterpretationMut, VocabularyMut,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod intersection;
mod list;
mod literal;
mod product;
mod sum;
mod r#union;

pub use intersection::*;
pub use list::*;
pub use literal::*;
pub use product::*;
pub use r#union::*;
pub use sum::*;

use crate::{
	abs::{self, Builder},
	layout::LayoutType,
	Ref,
};

use super::{
	Build, BuildError, CompactIri, Context, Dataset, Error, OneOrMany, Pattern, Resource, Scope, ValueFormat, VariableName
};

/// Abstract syntax layout.
///
/// This is the most human friendly version of the `Layout` type.
/// It implements `serde::Serialize` and `serde::Deserialize` in a way that
/// produces and consumes human readable data. It is the basis to the JSON
/// syntax for layouts often used throughout the documentation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Layout {
	/// Matches literal values.
	///
	/// # Example
	///
	/// The following JSON value is a serialized abstract literal (number)
	/// layout:
	/// ```json
	/// { "type": "number" }
	/// ```
	/// It matches the following example value (a JSON number):
	/// ```json
	/// 12
	/// ```
	///
	/// It matches any RDF dataset for any input resource that is a number
	/// literal.
	Literal(LiteralLayout),

	/// Matches objects/records.
	///
	/// # Example
	///
	/// The following JSON value is a serialized abstract record layout:
	/// ```json
	/// {
	///   "type": "record",
	///   "fields": {
	///     "f1": ...,
	///     ...
	///     "fn": ...
	///   }
	/// }
	/// ```
	///
	/// It matches any record value (JSON object) containing the fields `f1`,
	/// ..., `fn`.
	Product(ProductLayout),

	/// Matches exactly one of the given layouts.
	Sum(SumLayout),

	/// Matches lists.
	List(ListLayout),

	/// Either never (`false`) or always (`true`)
	Boolean(bool),

	/// Layout union.
	Union(UnionLayout),

	/// Layout intersection.
	Intersection(IntersectionLayout),
}

impl Layout {
	pub fn id(&self) -> Option<&CompactIri> {
		match self {
			Self::Literal(l) => l.id(),
			Self::Product(l) => l.header.id.as_ref(),
			Self::Sum(l) => l.header.id.as_ref(),
			Self::List(l) => l.id(),
			Self::Boolean(false) => None,
			Self::Boolean(true) => None,
			Self::Union(l) => l.header.id.as_ref(),
			Self::Intersection(l) => l.header.id.as_ref(),
		}
	}

	pub fn build(&self, builder: &mut Builder) -> Result<Ref<LayoutType>, BuildError> {
		let mut context = builder.with_generator_mut(generator::Blank::new());
		self.build_with_context(&mut context)
	}

	pub fn build_with_interpretation<V, I>(
		&self,
		vocabulary: &mut V,
		interpretation: &mut I,
		builder: &mut Builder<I::Resource>,
	) -> Result<Ref<LayoutType, I::Resource>, BuildError>
	where
		V: VocabularyMut,
		I: IriInterpretationMut<V::Iri>
			+ LiteralInterpretationMut<V::Literal>
			+ InterpretationMut<V>,
		I::Resource: Clone + Eq + Ord + std::fmt::Debug,
	{
		let mut context = builder.with_interpretation_mut(vocabulary, interpretation);
		self.build_with_context(&mut context)
	}

	pub fn build_with_context<C: Context>(
		&self,
		context: &mut C,
	) -> Result<Ref<LayoutType, C::Resource>, BuildError>
	where
		C::Resource: Clone,
	{
		let scope = Scope::default();
		Build::build(self, context, &scope)
	}
}

impl<C: Context> Build<C> for Layout
where
	C::Resource: Clone,
{
	type Target = Ref<LayoutType, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let id = match self.id() {
			Some(id) => {
				let iri = id.resolve(scope)?;
				context.iri_resource(&iri)
			}
			None => context.anonymous_resource(),
		};

		let layout = match self {
			Self::Literal(l) => abs::Layout::Literal(l.build(context, scope)?),
			Self::Product(l) => abs::Layout::Product(l.build(context, scope)?),
			Self::Sum(l) => abs::Layout::Sum(l.build(context, scope)?),
			Self::List(l) => abs::Layout::List(l.build(context, scope)?),
			Self::Boolean(false) => abs::Layout::Never,
			Self::Boolean(true) => abs::Layout::Always,
			Self::Union(l) => abs::Layout::Union(l.build(context, scope)?),
			Self::Intersection(l) => abs::Layout::Intersection(l.build(context, scope)?),
		};

		let (layout_ref, old_layout) = context.insert_layout(id, layout);

		if old_layout.is_some() {
			Err(BuildError::LayoutRedefinition)
		} else {
			Ok(layout_ref)
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LayoutRef {
	Ref(CompactIri),
	Layout(Box<Layout>),
}

impl<C: Context> Build<C> for LayoutRef
where
	C::Resource: Clone,
{
	type Target = Ref<LayoutType, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let scope = scope.without_variables();
		match self {
			Self::Ref(r) => r.build(context, &scope).map(Ref::new),
			Self::Layout(l) => l.build(context, &scope),
		}
	}
}

impl From<IriBuf> for LayoutRef {
	fn from(value: IriBuf) -> Self {
		Self::Ref(value.into())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayoutInput(OneOrMany<String>);

impl LayoutInput {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn as_slice(&self) -> &[String] {
		self.0.as_slice()
	}

	pub fn is_default(&self) -> bool {
		let slice = self.0.as_slice();
		slice.len() == 1 && slice[0] == "self"
	}
}

impl Default for LayoutInput {
	fn default() -> Self {
		Self(OneOrMany::One("self".to_owned()))
	}
}

impl From<Vec<String>> for LayoutInput {
	fn from(value: Vec<String>) -> Self {
		Self(value.into())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutHeader {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub base: Option<CompactIri>,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub prefixes: BTreeMap<String, CompactIri>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub id: Option<CompactIri>,

	#[serde(default, skip_serializing_if = "LayoutInput::is_default")]
	pub input: LayoutInput,

	#[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
	pub intro: OneOrMany<String>,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,

	#[serde(default, skip_serializing_if = "ExtraProperties::is_empty")]
	pub extra: ExtraProperties,
}

impl LayoutHeader {
	fn try_from_json_syntax_at(object: &json_syntax::Object, code_map: &json_syntax::CodeMap, offset: usize) -> Result<Self, Error> {
		todo!()
	}
}

impl<C: Context> Build<C> for LayoutHeader
where
	C::Resource: Clone,
{
	type Target = (BuiltLayoutHeader<C::Resource>, Scope);

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let scope = scope.with_header(self)?;

		let header = BuiltLayoutHeader {
			input: self.input.len() as u32,
			intro: self.intro.len() as u32,
			dataset: self.dataset.build(context, &scope)?,
			properties: self.extra.build(context, &scope)?,
		};

		Ok((header, scope))
	}
}

pub struct BuiltLayoutHeader<R> {
	input: u32,

	intro: u32,

	dataset: crate::Dataset<R>,

	properties: BTreeMap<R, R>,
}

/// RDF Resource properties.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExtraProperties(BTreeMap<CompactIri, Resource>);

impl ExtraProperties {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl<C: Context> Build<C> for ExtraProperties {
	type Target = BTreeMap<C::Resource, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let mut result = BTreeMap::new();

		for (prop, value) in &self.0 {
			let prop = prop.build(context, scope)?;
			let value = value.build(context, scope)?;
			result.insert(prop, value);
		}

		Ok(result)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValueInput(OneOrMany<Pattern>);

impl ValueInput {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn as_slice(&self) -> &[Pattern] {
		self.0.as_slice()
	}

	pub fn is_default(&self) -> bool {
		let slice = self.0.as_slice();
		slice.len() == 1 && slice[0].is_variable("value")
	}

	pub fn first(&self) -> Option<&Pattern> {
		self.0.as_slice().first()
	}
}

impl Default for ValueInput {
	fn default() -> Self {
		Self(OneOrMany::One(Pattern::Var(VariableName::VALUE.to_owned())))
	}
}

impl From<Vec<Pattern>> for ValueInput {
	fn from(value: Vec<Pattern>) -> Self {
		Self(value.into())
	}
}

impl<C: Context> Build<C> for ValueFormat
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let mut inputs = Vec::with_capacity(self.input.len());
		for i in self.input.as_slice() {
			inputs.push(i.build(context, scope)?);
		}

		Ok(crate::ValueFormat {
			layout: self.layout.build(context, scope)?,
			input: inputs,
			graph: self
				.graph
				.as_ref()
				.map(|g| g.as_ref().map(|g| g.build(context, scope)).transpose())
				.transpose()?,
		})
	}
}

macro_rules! type_markers {
	($($id:ident: $value:literal),*) => {
		$(
			#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
			pub struct $id;

			impl $id {
				pub const NAME: &'static str = $value;

				pub fn as_str(&self) -> &'static str {
					$value
				}

				pub fn into_str(self) -> &'static str {
					$value
				}
			}

			impl AsRef<str> for $id {
				fn as_ref(&self) -> &str {
					self.as_str()
				}
			}

			impl std::borrow::Borrow<str> for $id {
				fn borrow(&self) -> &str {
					self.as_str()
				}
			}

			impl fmt::Display for $id {
				fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
					self.as_str().fmt(f)
				}
			}

			impl Serialize for $id {
				fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
				where
					S: serde::Serializer
				{
					serializer.serialize_str(self.as_str())
				}
			}

			impl<'de> Deserialize<'de> for $id {
				fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
				where
					D: serde::Deserializer<'de>
				{
					struct Visitor;

					impl<'de> serde::de::Visitor<'de> for Visitor {
						type Value = $id;

						fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
							write!(f, "the string `{}`", $value)
						}

						fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
						where
							E: serde::de::Error
						{
							if v == $value {
								Ok($id)
							} else {
								Err(E::invalid_value(
									serde::de::Unexpected::Str(v),
									&self
								))
							}
						}
					}

					deserializer.deserialize_str(Visitor)
				}
			}
		)*
	};
}

type_markers! {
	UnitLayoutType: "unit",
	BooleanLayoutType: "boolean",
	NumberLayoutType: "number",
	ByteStringLayoutType: "bytes",
	TextStringLayoutType: "string",
	IdLayoutType: "id",
	ProductLayoutType: "record",
	SumLayoutType: "sum",
	OrderedListLayoutType: "list",
	UnorderedListLayoutType: "set",
	SizedListLayoutType: "tuple",
	UnionLayoutType: "union",
	IntersectionLayoutType: "intersection"
}
