use iref::{Iri, IriBuf, IriRefBuf};
use rdf_types::{
	generator, BlankIdBuf, Id, InterpretationMut, IriInterpretationMut, IriVocabularyMut, Term,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	fmt,
};

use crate::{
	abs::{self, InsertResult, LayoutType, RegExp},
	Ref,
};

use super::Builder;

pub trait Context {
	type Resource: Ord;

	fn insert_layout(
		&mut self,
		id: Self::Resource,
		layout: abs::Layout<Self::Resource>,
	) -> InsertResult<Self::Resource>;

	fn iri_resource(&mut self, iri: &Iri) -> Self::Resource;

	fn anonymous_resource(&mut self) -> Self::Resource;
}

impl<'a, G> Context for super::BuilderWithGeneratorMut<'a, G>
where
	G: rdf_types::Generator,
{
	type Resource = Term;

	fn insert_layout(
		&mut self,
		id: Self::Resource,
		layout: abs::Layout<Self::Resource>,
	) -> (
		Ref<LayoutType, Self::Resource>,
		Option<abs::Layout<Self::Resource>>,
	) {
		self.builder.insert(id, layout)
	}

	fn iri_resource(&mut self, iri: &Iri) -> Self::Resource {
		Term::Id(Id::Iri(iri.to_owned()))
	}

	fn anonymous_resource(&mut self) -> Self::Resource {
		Term::Id(self.generator.next(&mut ()))
	}
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
		layout: abs::Layout<Self::Resource>,
	) -> (
		Ref<LayoutType, Self::Resource>,
		Option<abs::Layout<Self::Resource>>,
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

impl<C, T: Build<C>> Build<C> for Box<T> {
	type Target = T::Target;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		T::build(&**self, context, scope)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("no base IRI to resolve `{0}`")]
	NoBaseIri(IriRefBuf),

	#[error("invalid IRI `{0}`")]
	InvalidIri(String),

	#[error("undeclared variable `{0}`")]
	UndeclaredVariable(String),

	#[error("layout redefinition")]
	LayoutRedefinition,

	#[error("missing literal layout target resource")]
	MissingLiteralTargetResource,

	#[error("no property subject")]
	NoPropertySubject,

	#[error("no property object")]
	NoPropertyObject,
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
				None => Err(Error::NoBaseIri(self.0.clone())),
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

		if let Some(base_iri) = &header.base {
			result.base_iri = Some(base_iri.resolve(self)?)
		}

		for (name, prefix) in &header.prefixes {
			result
				.iri_prefixes
				.insert(name.clone(), prefix.resolve(self)?);
		}

		for name in header
			.input
			.as_slice()
			.iter()
			.chain(header.intro.as_slice())
		{
			result.bind(name)
		}

		Ok(result)
	}

	pub fn with_intro<'s>(
		&self,
		intro: impl IntoIterator<Item = &'s String>,
	) -> Result<Self, Error> {
		let mut result = self.clone();

		for name in intro {
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
#[serde(untagged)]
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

	pub fn build(&self, builder: &mut Builder) -> Result<Ref<LayoutType>, Error> {
		let mut context = builder.with_generator_mut(generator::Blank::new());
		self.build_with_context(&mut context)
	}

	pub fn build_with_interpretation<V, I>(
		&self,
		vocabulary: &mut V,
		interpretation: &mut I,
		builder: &mut Builder<I::Resource>,
	) -> Result<Ref<LayoutType, I::Resource>, Error>
	where
		V: IriVocabularyMut,
		I: IriInterpretationMut<V::Iri> + InterpretationMut<V>,
		I::Resource: Clone + Eq + Ord,
	{
		let mut context = builder.with_interpretation_mut(vocabulary, interpretation);
		self.build_with_context(&mut context)
	}

	pub fn build_with_context<C: Context>(
		&self,
		context: &mut C,
	) -> Result<Ref<LayoutType, C::Resource>, Error> {
		let scope = Scope::default();
		Build::build(self, context, &scope)
	}
}

impl<C: Context> Build<C> for Layout {
	type Target = Ref<LayoutType, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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
			Err(Error::LayoutRedefinition)
		} else {
			Ok(layout_ref)
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LayoutRef {
	Ref(CompactIri),
	Layout(Box<Layout>),
}

impl<C: Context> Build<C> for LayoutRef {
	type Target = Ref<LayoutType, C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let scope = scope.without_variables();
		match self {
			Self::Ref(r) => r.build(context, &scope).map(Ref::new),
			Self::Layout(l) => l.build(context, &scope),
		}
	}
}

#[derive(Debug)]
pub enum Pattern {
	Var(String),
	Iri(CompactIri),
}

impl Pattern {
	pub fn is_variable(&self, name: &str) -> bool {
		match self {
			Self::Var(x) => x == name,
			Self::Iri(_) => false,
		}
	}
}

impl Serialize for Pattern {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Var(name) => format!("_:{name}").serialize(serializer),
			Self::Iri(compact_iri) => compact_iri.serialize(serializer),
		}
	}
}

impl<'de> Deserialize<'de> for Pattern {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Pattern;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a compact IRI or blank node identifier")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				self.visit_string(v.to_owned())
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				match BlankIdBuf::new(v) {
					Ok(blank_id) => Ok(Pattern::Var(blank_id.suffix().to_owned())),
					Err(e) => match IriRefBuf::new(e.0) {
						Ok(iri_ref) => Ok(Pattern::Iri(CompactIri(iri_ref))),
						Err(e) => Err(E::invalid_value(serde::de::Unexpected::Str(&e.0), &self)),
					},
				}
			}
		}

		deserializer.deserialize_string(Visitor)
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
	One(T),
	Many(Vec<T>),
}

impl<T> OneOrMany<T> {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::One(_) => false,
			Self::Many(v) => v.is_empty(),
		}
	}

	pub fn len(&self) -> usize {
		match self {
			Self::One(_) => 1,
			Self::Many(v) => v.len(),
		}
	}

	pub fn as_slice(&self) -> &[T] {
		match self {
			Self::One(t) => std::slice::from_ref(t),
			Self::Many(v) => v.as_slice(),
		}
	}
}

impl<T> Default for OneOrMany<T> {
	fn default() -> Self {
		Self::Many(Vec::new())
	}
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutHeader {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	base: Option<CompactIri>,

	#[serde(default, skip_serializing_if = "HashMap::is_empty")]
	prefixes: HashMap<String, CompactIri>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	id: Option<CompactIri>,

	#[serde(default, skip_serializing_if = "LayoutInput::is_default")]
	input: LayoutInput,

	#[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
	intro: OneOrMany<String>,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	dataset: Dataset,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dataset(Vec<Quad>);

impl Dataset {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl<C: Context> Build<C> for Dataset {
	type Target = crate::Dataset<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let mut dataset = crate::Dataset::new();
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
	type Target = crate::Pattern<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Var(name) => Ok(crate::Pattern::Var(scope.variable(name)?)),
			Self::Iri(compact_iri) => {
				let iri = compact_iri.resolve(scope)?;
				Ok(crate::Pattern::Resource(context.iri_resource(&iri)))
			}
		}
	}
}

impl<C: Context> Build<C> for Quad {
	type Target = rdf_types::Quad<
		crate::Pattern<C::Resource>,
		crate::Pattern<C::Resource>,
		crate::Pattern<C::Resource>,
		crate::Pattern<C::Resource>,
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
	type Target = (BuiltLayoutHeader<C::Resource>, Scope);

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let scope = scope.with_header(self)?;

		let header = BuiltLayoutHeader {
			input: self.input.len() as u32,
			intro: self.intro.len() as u32,
			dataset: self.dataset.build(context, &scope)?,
		};

		Ok((header, scope))
	}
}

pub struct BuiltLayoutHeader<R> {
	input: u32,

	intro: u32,

	dataset: crate::Dataset<R>,
}

#[derive(Debug, Serialize, Deserialize)]
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
}

impl Default for ValueInput {
	fn default() -> Self {
		Self(OneOrMany::One(Pattern::Var("value".to_owned())))
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueFormatOrLayout {
	Format(ValueFormat),
	Layout(LayoutRef),
}

impl<C: Context> Build<C> for ValueFormatOrLayout {
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Format(f) => f.build(context, scope),
			Self::Layout(layout) => Ok(crate::ValueFormat {
				layout: layout.build(context, scope)?,
				input: vec![Pattern::Var("value".to_string()).build(context, scope)?],
				graph: None,
			}),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueFormat {
	layout: LayoutRef,

	#[serde(default, skip_serializing_if = "ValueInput::is_default")]
	input: ValueInput,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	graph: Option<Option<Pattern>>,
}

impl<C: Context> Build<C> for ValueFormat {
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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
	type Target = abs::layout::LiteralLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Data(l) => Ok(abs::layout::LiteralLayout::Data(l.build(context, scope)?)),
			Self::Id(l) => Ok(abs::layout::LiteralLayout::Id(l.build(context, scope)?)),
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
	type Target = abs::layout::DataLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Unit(l) => l.build(context, scope).map(abs::layout::DataLayout::Unit),
			Self::Boolean(l) => l
				.build(context, scope)
				.map(abs::layout::DataLayout::Boolean),
			Self::Number(l) => l.build(context, scope).map(abs::layout::DataLayout::Number),
			Self::ByteString(l) => l
				.build(context, scope)
				.map(abs::layout::DataLayout::ByteString),
			Self::TextString(l) => l
				.build(context, scope)
				.map(abs::layout::DataLayout::TextString),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitLayout {
	#[serde(rename = "type")]
	type_: UnitLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,
}

impl<C: Context> Build<C> for UnitLayout {
	type Target = abs::layout::UnitLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, _) = self.header.build(context, scope)?;
		Ok(abs::layout::UnitLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanLayout {
	#[serde(rename = "type")]
	type_: BooleanLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	resource: Option<Pattern>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	datatype: Option<CompactIri>,
}

fn literal_resource<C: Context>(
	context: &mut C,
	scope: &Scope,
	input: &LayoutInput,
	resource: Option<&Pattern>,
) -> Result<crate::Pattern<C::Resource>, Error> {
	match resource {
		Some(r) => r.build(context, scope),
		None => {
			if input.is_empty() {
				Err(Error::MissingLiteralTargetResource)
			} else {
				Ok(crate::Pattern::Var(0))
			}
		}
	}
}

impl<C: Context> Build<C> for BooleanLayout {
	type Target = abs::layout::BooleanLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::BooleanLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self
				.datatype
				.as_ref()
				.map(|i| i.build(context, &scope))
				.transpose()?
				.unwrap_or_else(|| context.iri_resource(xsd_types::XSD_BOOLEAN)),
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberLayout {
	#[serde(rename = "type")]
	type_: NumberLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	resource: Option<Pattern>,

	datatype: CompactIri,
}

impl<C: Context> Build<C> for NumberLayout {
	type Target = abs::layout::NumberLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::NumberLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self.datatype.build(context, &scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByteStringLayout {
	#[serde(rename = "type")]
	type_: ByteStringLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	resource: Option<Pattern>,

	datatype: CompactIri,
}

impl<C: Context> Build<C> for ByteStringLayout {
	type Target = abs::layout::ByteStringLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::ByteStringLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self.datatype.build(context, &scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextStringLayout {
	#[serde(rename = "type")]
	type_: TextStringLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pattern: Option<RegExp>,

	resource: Option<Pattern>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	datatype: Option<CompactIri>,
}

impl<C: Context> Build<C> for TextStringLayout {
	type Target = abs::layout::TextStringLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::TextStringLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			pattern: self.pattern.clone(),
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
			datatype: self
				.datatype
				.as_ref()
				.map(|i| i.build(context, &scope))
				.transpose()?
				.unwrap_or_else(|| context.iri_resource(xsd_types::XSD_STRING)),
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdLayout {
	#[serde(rename = "type")]
	type_: IdLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pattern: Option<RegExp>,

	resource: Option<Pattern>,
}

impl<C: Context> Build<C> for IdLayout {
	type Target = abs::layout::IdLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::IdLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			pattern: self.pattern.clone(),
			resource: literal_resource(
				context,
				&scope,
				&self.header.input,
				self.resource.as_ref(),
			)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductLayout {
	#[serde(rename = "type")]
	type_: ProductLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	fields: BTreeMap<String, Field>,
}

impl<C: Context> Build<C> for ProductLayout {
	type Target = abs::layout::ProductLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		let mut fields = BTreeMap::new();

		for (name, field) in &self.fields {
			let scope = scope.with_intro(field.intro.as_slice())?;

			let mut dataset = field.dataset.build(context, &scope)?;

			if let Some(property) = &field.property {
				if self.header.input.is_empty() {
					return Err(Error::NoPropertySubject);
				} else {
					let subject = crate::Pattern::Var(0);
					if field.intro.is_empty() {
						return Err(Error::NoPropertyObject);
					} else {
						let object = crate::Pattern::Var(
							(self.header.input.len() + self.header.intro.len()) as u32,
						);
						let predicate = property.build(context, &scope)?;
						dataset.insert(rdf_types::Quad(subject, predicate, object, None));
					}
				}
			}

			fields.insert(
				name.to_owned(),
				crate::layout::product::Field {
					intro: field.intro.len() as u32,
					value: field.value.build(context, &scope)?,
					dataset,
				},
			);
		}

		Ok(abs::layout::ProductLayout {
			input: header.input,
			intro: header.intro,
			fields,
			dataset: header.dataset,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValueIntro(OneOrMany<String>);

impl ValueIntro {
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
		slice.len() == 1 && slice[0] == "value"
	}
}

impl Default for ValueIntro {
	fn default() -> Self {
		Self(OneOrMany::One("value".to_owned()))
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	intro: ValueIntro,

	value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	dataset: Dataset,

	property: Option<Pattern>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SumLayout {
	#[serde(rename = "type")]
	type_: SumLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	variants: BTreeMap<String, Variant>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	intro: ValueIntro,

	value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	dataset: Dataset,
}

impl<C: Context> Build<C> for SumLayout {
	type Target = abs::layout::SumLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		let mut variants = Vec::with_capacity(self.variants.len());

		for (name, variant) in &self.variants {
			let scope = scope.with_intro(variant.intro.as_slice())?;
			variants.push(crate::layout::sum::Variant {
				name: name.to_owned(),
				intro: variant.intro.len() as u32,
				value: variant.value.build(context, &scope)?,
				dataset: variant.dataset.build(context, &scope)?,
			})
		}

		Ok(abs::layout::SumLayout {
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
	type Target = abs::layout::ListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Ordered(l) => l
				.build(context, scope)
				.map(abs::layout::ListLayout::Ordered),
			Self::Unordered(l) => l
				.build(context, scope)
				.map(abs::layout::ListLayout::Unordered),
			Self::Sized(l) => l.build(context, scope).map(abs::layout::ListLayout::Sized),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ListNode {
	head: String,

	rest: String,

	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	intro: ValueIntro,

	value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	dataset: Dataset,
}

impl<C: Context> Build<C> for OrderedListLayout {
	type Target = abs::layout::OrderedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::OrderedListLayout {
			input: header.input,
			intro: header.intro,
			node: self.node.build(context, &scope)?,
			head: self.head.build(context, &scope)?,
			tail: self.tail.build(context, &scope)?,
			dataset: header.dataset,
		})
	}
}

impl<C: Context> Build<C> for ListNode {
	type Target = abs::layout::list::ordered::NodeLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let scope = scope.with_intro(
			[&self.head, &self.rest]
				.into_iter()
				.chain(self.intro.as_slice()),
		)?;
		Ok(abs::layout::list::ordered::NodeLayout {
			intro: self.intro.len() as u32,
			value: self.value.build(context, &scope)?,
			dataset: self.dataset.build(context, &scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnorderedListLayout {
	#[serde(rename = "type")]
	type_: UnorderedListLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	item: ListItem,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListItem {
	#[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
	intro: OneOrMany<String>,

	value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	dataset: Dataset,
}

impl<C: Context> Build<C> for UnorderedListLayout {
	type Target = abs::layout::UnorderedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::UnorderedListLayout {
			input: header.input,
			intro: header.intro,
			item: self.item.build(context, &scope)?,
			dataset: header.dataset,
		})
	}
}

impl<C: Context> Build<C> for ListItem {
	type Target = abs::layout::list::ItemLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let scope = scope.with_intro(self.intro.as_slice())?;
		Ok(abs::layout::list::ItemLayout {
			intro: self.intro.len() as u32,
			value: self.value.build(context, &scope)?,
			dataset: self.dataset.build(context, &scope)?,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizedListLayout {
	#[serde(rename = "type")]
	type_: SizedListLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	items: Vec<ListItem>,
}

impl<C: Context> Build<C> for SizedListLayout {
	type Target = abs::layout::SizedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;

		let mut items = Vec::with_capacity(self.items.len());
		for item in &self.items {
			items.push(item.build(context, &scope)?)
		}

		Ok(abs::layout::SizedListLayout {
			input: header.input,
			intro: header.intro,
			items,
			dataset: header.dataset,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntersectionLayout {
	#[serde(rename = "type")]
	type_: IntersectionLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,
}

impl<C: Context> Build<C> for IntersectionLayout {
	type Target = Vec<Ref<LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, Error> {
		unimplemented!()
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnionLayout {
	#[serde(rename = "type")]
	type_: UnionLayoutType,

	#[serde(flatten)]
	header: LayoutHeader,
}

impl<C: Context> Build<C> for UnionLayout {
	type Target = Vec<Ref<LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, Error> {
		unimplemented!()
	}
}
