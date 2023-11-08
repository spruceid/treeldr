use iref::{Iri, IriBuf, IriRefBuf};
use rdf_types::{InterpretationMut, IriInterpretationMut, IriVocabularyMut};
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	fmt,
};
use treeldr::Ref;

use crate::{InsertResult, RegExp};

pub trait Context {
	type Resource: Ord;

	fn insert_layout(
		&mut self,
		id: Self::Resource,
		layout: crate::Layout<Self::Resource>,
	) -> InsertResult<Self::Resource>;

	fn iri_resource(&mut self, iri: &Iri) -> Self::Resource;

	fn anonymous_resource(&mut self) -> Self::Resource;
}

impl<'a, V, I> Context for super::BuilderWithInterpretationMut<'a, V, I>
where
	V: IriVocabularyMut,
	I: IriInterpretationMut<V::Iri> + InterpretationMut<V>,
	I::Resource: Clone + Eq + Ord,
{
	type Resource = I::Resource;

	fn insert_layout(
		&mut self,
		id: Self::Resource,
		layout: crate::Layout<Self::Resource>,
	) -> (
		Ref<crate::LayoutType, Self::Resource>,
		Option<crate::Layout<Self::Resource>>,
	) {
		self.builder.insert(id, layout)
	}

	fn iri_resource(&mut self, iri: &Iri) -> Self::Resource {
		let i = self.vocabulary.insert(iri);
		self.interpretation.interpret_iri(i)
	}

	fn anonymous_resource(&mut self) -> Self::Resource {
		self.interpretation.new_resource(self.vocabulary)
	}
}

pub trait Build<C> {
	type Target;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("no base IRI")]
	NoBaseIri,

	#[error("invalid IRI `{0}`")]
	InvalidIri(String),

	#[error("undeclared variable `{0}`")]
	UndeclaredVariable(String),

	#[error("layout redefinition")]
	LayoutRedefinition,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompactIri(IriRefBuf);

impl CompactIri {
	pub fn resolve(&self, scope: &Scope) -> Result<IriBuf, Error> {
		match self.0.as_iri() {
			Some(iri) => match scope.iri_prefixes.get(iri.scheme().as_str()) {
				Some(prefix) => {
					let suffix = iri.split_once(':').unwrap().1;
					IriBuf::new(format!("{prefix}{suffix}")).map_err(|e| Error::InvalidIri(e.0))
				}
				None => Ok(iri.to_owned()),
			},
			None => match &scope.base_iri {
				Some(base_iri) => Ok(self.0.resolved(base_iri)),
				None => Err(Error::NoBaseIri),
			},
		}
	}
}

impl<C: Context> Build<C> for CompactIri {
	type Target = C::Resource;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let iri = self.resolve(scope)?;
		Ok(context.iri_resource(&iri))
	}
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
	base_iri: Option<IriBuf>,
	iri_prefixes: HashMap<String, IriBuf>,
	variables: HashMap<String, u32>,
}

impl Scope {
	pub fn with_header(&self, header: &LayoutHeader) -> Result<Self, Error> {
		let mut result = self.clone();

		if let Some(base_iri) = &header.base_iri {
			result.base_iri = Some(base_iri.resolve(self)?)
		}

		for (name, prefix) in &header.prefixes {
			result
				.iri_prefixes
				.insert(name.clone(), prefix.resolve(self)?);
		}

		for name in header.input.iter().chain(&header.intro) {
			result.bind(name)
		}

		Ok(result)
	}

	pub fn with_intro<'s>(
		&self,
		intros: impl IntoIterator<Item = &'s String>,
	) -> Result<Self, Error> {
		let mut result = self.clone();

		for name in intros {
			result.bind(name)
		}

		Ok(result)
	}

	pub fn without_variables(&self) -> Self {
		Self {
			base_iri: self.base_iri.clone(),
			iri_prefixes: self.iri_prefixes.clone(),
			variables: HashMap::new(),
		}
	}

	pub fn bind(&mut self, name: &str) {
		let i = self.variables.len() as u32;
		self.variables.insert(name.to_owned(), i);
	}

	pub fn variable(&self, name: &str) -> Result<u32, Error> {
		self.variables
			.get(name)
			.copied()
			.ok_or_else(|| Error::UndeclaredVariable(name.to_owned()))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Layout {
	/// Matches literal values.
	Literal(LiteralLayout),

	/// Matches objects/records.
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

	fn build<C: Context>(
		&self,
		context: &mut C,
		scope: &Scope,
	) -> Result<Ref<crate::LayoutType, C::Resource>, Error> {
		Build::build(self, context, scope)
	}
}

impl<C: Context> Build<C> for Layout {
	type Target = Ref<crate::LayoutType, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let id = match self.id() {
			Some(id) => {
				let iri = id.resolve(scope)?;
				context.iri_resource(&iri)
			}
			None => context.anonymous_resource(),
		};

		let layout = match self {
			Self::Literal(l) => crate::Layout::Literal(l.build(context, scope)?),
			Self::Product(l) => crate::Layout::Product(l.build(context, scope)?),
			Self::Sum(l) => crate::Layout::Sum(l.build(context, scope)?),
			Self::List(l) => crate::Layout::List(l.build(context, scope)?),
			Self::Boolean(false) => crate::Layout::Never,
			Self::Boolean(true) => crate::Layout::Always,
			Self::Union(l) => crate::Layout::Union(l.build(context, scope)?),
			Self::Intersection(l) => crate::Layout::Intersection(l.build(context, scope)?),
		};

		let (layout_ref, old_layout) = context.insert_layout(id, layout);

		if old_layout.is_some() {
			Err(Error::LayoutRedefinition)
		} else {
			Ok(layout_ref)
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LayoutRef {
	Ref(Reference),
	Layout(Box<Layout>),
}

impl<C: Context> Build<C> for LayoutRef {
	type Target = Ref<crate::LayoutType, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let scope = scope.without_variables();
		match self {
			Self::Ref(r) => r.build(context, &scope),
			Self::Layout(l) => l.build(context, &scope),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
	#[serde(rename = "ref")]
	ref_: CompactIri,
}

impl<C: Context> Build<C> for Reference {
	type Target = Ref<crate::LayoutType, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let iri = self.ref_.resolve(scope)?;
		Ok(treeldr::Ref::new(context.iri_resource(&iri)))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Pattern {
	Var(String),
	Iri(CompactIri),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LayoutHeader {
	base_iri: Option<CompactIri>,

	prefixes: HashMap<String, CompactIri>,

	id: Option<CompactIri>,

	input: Vec<String>,

	intro: Vec<String>,

	dataset: Dataset,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dataset(Vec<Quad>);

impl<C: Context> Build<C> for Dataset {
	type Target = treeldr::Dataset<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let mut dataset = treeldr::Dataset::new();
		for quad in &self.0 {
			dataset.insert(quad.build(context, scope)?);
		}

		Ok(dataset)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quad(
	Pattern,
	Pattern,
	Pattern,
	#[serde(default, skip_serializing_if = "Option::is_none")] Option<Pattern>,
);

impl<C: Context> Build<C> for Pattern {
	type Target = treeldr::Pattern<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Var(name) => Ok(treeldr::Pattern::Var(scope.variable(name)?)),
			Self::Iri(compact_iri) => {
				let iri = compact_iri.resolve(scope)?;
				Ok(treeldr::Pattern::Resource(context.iri_resource(&iri)))
			}
		}
	}
}

impl<C: Context> Build<C> for Quad {
	type Target = rdf_types::Quad<
		treeldr::Pattern<C::Resource>,
		treeldr::Pattern<C::Resource>,
		treeldr::Pattern<C::Resource>,
		treeldr::Pattern<C::Resource>,
	>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		Ok(rdf_types::Quad(
			self.0.build(context, scope)?,
			self.1.build(context, scope)?,
			self.2.build(context, scope)?,
			self.3
				.as_ref()
				.map(|g| g.build(context, scope))
				.transpose()?,
		))
	}
}

impl<C: Context> Build<C> for LayoutHeader {
	type Target = BuiltLayoutHeader<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		Ok(BuiltLayoutHeader {
			input: self.input.len() as u32,
			intro: self.intro.len() as u32,
			dataset: self.dataset.build(context, scope)?,
		})
	}
}

pub struct BuiltLayoutHeader<R> {
	input: u32,

	intro: u32,

	dataset: treeldr::Dataset<R>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Format {
	layout: LayoutRef,

	inputs: Vec<Pattern>,

	graph: Option<Option<Pattern>>,
}

impl<C: Context> Build<C> for Format {
	type Target = treeldr::Format<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let mut inputs = Vec::with_capacity(self.inputs.len());
		for i in &self.inputs {
			inputs.push(i.build(context, scope)?);
		}

		Ok(treeldr::Format {
			layout: self.layout.build(context, scope)?,
			inputs,
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
			#[derive(Debug, Default, Clone, Copy)]
			pub struct $id;

			impl $id {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LiteralLayout {
	Data(DataLayout),
	Id(IdLayout),
}

impl LiteralLayout {
	pub fn id(&self) -> Option<&CompactIri> {
		match self {
			Self::Data(l) => l.id(),
			Self::Id(l) => l.header.id.as_ref(),
		}
	}
}

impl<C: Context> Build<C> for LiteralLayout {
	type Target = crate::layout::LiteralLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Data(l) => Ok(crate::layout::LiteralLayout::Data(l.build(context, scope)?)),
			Self::Id(l) => Ok(crate::layout::LiteralLayout::Id(l.build(context, scope)?)),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataLayout {
	Unit(UnitLayout),
	Boolean(BooleanLayout),
	Number(NumberLayout),
	ByteString(ByteStringLayout),
	TextString(TextStringLayout),
}

impl DataLayout {
	pub fn id(&self) -> Option<&CompactIri> {
		match self {
			Self::Unit(l) => l.header.id.as_ref(),
			Self::Boolean(l) => l.header.id.as_ref(),
			Self::Number(l) => l.header.id.as_ref(),
			Self::ByteString(l) => l.header.id.as_ref(),
			Self::TextString(l) => l.header.id.as_ref(),
		}
	}
}

impl<C: Context> Build<C> for DataLayout {
	type Target = crate::layout::DataLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Unit(l) => l.build(context, scope).map(crate::layout::DataLayout::Unit),
			Self::Boolean(l) => l
				.build(context, scope)
				.map(crate::layout::DataLayout::Boolean),
			Self::Number(l) => l
				.build(context, scope)
				.map(crate::layout::DataLayout::Number),
			Self::ByteString(l) => l
				.build(context, scope)
				.map(crate::layout::DataLayout::ByteString),
			Self::TextString(l) => l
				.build(context, scope)
				.map(crate::layout::DataLayout::TextString),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnitLayout {
	#[serde(rename = "type")]
	type_: UnitLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,
}

impl<C: Context> Build<C> for UnitLayout {
	type Target = crate::layout::UnitLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;
		Ok(crate::layout::UnitLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BooleanLayout {
	#[serde(rename = "type")]
	type_: BooleanLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	resource: Pattern,

	datatype: CompactIri,
}

impl<C: Context> Build<C> for BooleanLayout {
	type Target = crate::layout::BooleanLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		Ok(crate::layout::BooleanLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: self.resource.build(context, scope)?,
			datatype: self.datatype.build(context, scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberLayout {
	#[serde(rename = "type")]
	type_: NumberLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	resource: Pattern,

	datatype: CompactIri,
}

impl<C: Context> Build<C> for NumberLayout {
	type Target = crate::layout::NumberLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		Ok(crate::layout::NumberLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: self.resource.build(context, scope)?,
			datatype: self.datatype.build(context, scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ByteStringLayout {
	#[serde(rename = "type")]
	type_: ByteStringLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	resource: Pattern,

	datatype: CompactIri,
}

impl<C: Context> Build<C> for ByteStringLayout {
	type Target = crate::layout::ByteStringLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		Ok(crate::layout::ByteStringLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: self.resource.build(context, scope)?,
			datatype: self.datatype.build(context, scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextStringLayout {
	#[serde(rename = "type")]
	type_: TextStringLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	pattern: Option<RegExp>,

	resource: Pattern,

	datatype: CompactIri,
}

impl<C: Context> Build<C> for TextStringLayout {
	type Target = crate::layout::TextStringLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		Ok(crate::layout::TextStringLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			pattern: self.pattern.clone(),
			resource: self.resource.build(context, scope)?,
			datatype: self.datatype.build(context, scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdLayout {
	#[serde(rename = "type")]
	type_: IdLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	pattern: Option<RegExp>,

	resource: Pattern,
}

impl<C: Context> Build<C> for IdLayout {
	type Target = crate::layout::IdLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		Ok(crate::layout::IdLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			pattern: self.pattern.clone(),
			resource: self.resource.build(context, scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductLayout {
	#[serde(rename = "type")]
	type_: ProductLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	fields: BTreeMap<String, Field>,
}

impl<C: Context> Build<C> for ProductLayout {
	type Target = crate::layout::ProductLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let mut fields = Vec::with_capacity(self.fields.len());

		for (name, field) in &self.fields {
			let scope = scope.with_intro(&field.intros)?;
			fields.push(treeldr::layout::product::Field {
				name: name.to_owned(),
				intro: field.intros.len() as u32,
				format: field.format.build(context, &scope)?,
				dataset: field.dataset.build(context, &scope)?,
			})
		}

		let header = self.header.build(context, scope)?;

		Ok(crate::layout::ProductLayout {
			input: header.input,
			intro: header.intro,
			fields,
			dataset: header.dataset,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
	intros: Vec<String>,

	format: Format,

	dataset: Dataset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SumLayout {
	#[serde(rename = "type")]
	type_: SumLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	variants: BTreeMap<String, Variant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variant {
	intros: Vec<String>,

	format: Format,

	dataset: Dataset,
}

impl<C: Context> Build<C> for SumLayout {
	type Target = crate::layout::SumLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let mut variants = Vec::with_capacity(self.variants.len());

		for (name, variant) in &self.variants {
			let scope = scope.with_intro(&variant.intros)?;
			variants.push(treeldr::layout::sum::Variant {
				name: name.to_owned(),
				intro: variant.intros.len() as u32,
				format: variant.format.build(context, &scope)?,
				dataset: variant.dataset.build(context, &scope)?,
			})
		}

		let header = self.header.build(context, scope)?;

		Ok(crate::layout::SumLayout {
			input: header.input,
			intro: header.intro,
			variants,
			dataset: header.dataset,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListLayout {
	Ordered(OrderedListLayout),
	Unordered(UnorderedListLayout),
	Sized(SizedListLayout),
}

impl ListLayout {
	pub fn id(&self) -> Option<&CompactIri> {
		match self {
			Self::Ordered(l) => l.header.id.as_ref(),
			Self::Unordered(l) => l.header.id.as_ref(),
			Self::Sized(l) => l.header.id.as_ref(),
		}
	}
}

impl<C: Context> Build<C> for ListLayout {
	type Target = crate::layout::ListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Ordered(l) => l
				.build(context, scope)
				.map(crate::layout::ListLayout::Ordered),
			Self::Unordered(l) => l
				.build(context, scope)
				.map(crate::layout::ListLayout::Unordered),
			Self::Sized(l) => l
				.build(context, scope)
				.map(crate::layout::ListLayout::Sized),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderedListLayout {
	#[serde(rename = "type")]
	type_: OrderedListLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	node: ListNode,

	head: Pattern,

	tail: Pattern,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListNode {
	head: String,

	rest: String,

	intro: Vec<String>,

	format: Format,

	dataset: Dataset,
}

impl<C: Context> Build<C> for OrderedListLayout {
	type Target = crate::layout::OrderedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		Ok(crate::layout::OrderedListLayout {
			input: header.input,
			intro: header.intro,
			node: self.node.build(context, scope)?,
			head: self.head.build(context, scope)?,
			tail: self.tail.build(context, scope)?,
			dataset: header.dataset,
		})
	}
}

impl<C: Context> Build<C> for ListNode {
	type Target = crate::layout::list::ordered::NodeLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let scope = scope.with_intro([&self.head, &self.rest].into_iter().chain(&self.intro))?;
		Ok(crate::layout::list::ordered::NodeLayout {
			intro: self.intro.len() as u32,
			format: self.format.build(context, &scope)?,
			dataset: self.dataset.build(context, &scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnorderedListLayout {
	#[serde(rename = "type")]
	type_: UnorderedListLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	item: ListItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItem {
	intro: Vec<String>,

	format: Format,

	dataset: Dataset,
}

impl<C: Context> Build<C> for UnorderedListLayout {
	type Target = crate::layout::UnorderedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		Ok(crate::layout::UnorderedListLayout {
			input: header.input,
			intro: header.intro,
			item: self.item.build(context, scope)?,
			dataset: header.dataset,
		})
	}
}

impl<C: Context> Build<C> for ListItem {
	type Target = crate::layout::list::ItemLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let scope = scope.with_intro(&self.intro)?;
		Ok(crate::layout::list::ItemLayout {
			intro: self.intro.len() as u32,
			format: self.format.build(context, &scope)?,
			dataset: self.dataset.build(context, &scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SizedListLayout {
	#[serde(rename = "type")]
	type_: SizedListLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	items: Vec<ListItem>,
}

impl<C: Context> Build<C> for SizedListLayout {
	type Target = crate::layout::SizedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let header = self.header.build(context, scope)?;

		let mut items = Vec::with_capacity(self.items.len());
		for item in &self.items {
			items.push(item.build(context, scope)?)
		}

		Ok(crate::layout::SizedListLayout {
			input: header.input,
			intro: header.intro,
			items,
			dataset: header.dataset,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntersectionLayout {
	#[serde(rename = "type")]
	type_: IntersectionLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,
}

impl<C: Context> Build<C> for IntersectionLayout {
	type Target = Vec<Ref<crate::LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, Error> {
		unimplemented!()
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnionLayout {
	#[serde(rename = "type")]
	type_: UnionLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,
}

impl<C: Context> Build<C> for UnionLayout {
	type Target = Vec<Ref<crate::LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, Error> {
		unimplemented!()
	}
}
