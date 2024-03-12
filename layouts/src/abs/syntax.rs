//! Abstract syntax for layouts.
//!
//! This module defines the most human friendly version of the `Layout` type.
//! It implements `serde::Serialize` and `serde::Deserialize` in a way that
//! produces and consumes human readable data. It is the basis to the JSON
//! syntax for layouts often used throughout the documentation.
//!
//! Abstract layouts must be compiled into `crate::Layout` before being used
//! to serialize/deserialize RDF datasets.
use iref::{Iri, IriBuf, IriRefBuf};
use rdf_types::{
	dataset::BTreeDataset,
	generator,
	interpretation::{IriInterpretationMut, LiteralInterpretationMut},
	BlankIdBuf, Id, InterpretationMut, Term, VocabularyMut, RDF_FIRST, RDF_NIL, RDF_REST,
};
use serde::{Deserialize, Serialize};
use std::{
	borrow::Borrow,
	collections::{BTreeMap, HashMap},
	fmt,
};
use xsd_types::{XSD_BOOLEAN, XSD_STRING};

use crate::{
	abs::{self, InsertResult, LayoutType, RegExp},
	Ref, Value,
};

use super::Builder;

pub trait Context {
	type Resource: Ord + std::fmt::Debug;

	fn insert_layout(
		&mut self,
		id: Self::Resource,
		layout: abs::Layout<Self::Resource>,
	) -> InsertResult<Self::Resource>;

	fn iri_resource(&mut self, iri: &Iri) -> Self::Resource;

	fn literal_resource(&mut self, value: &str, type_: &Iri) -> Self::Resource;

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

	fn literal_resource(&mut self, value: &str, type_: &Iri) -> Self::Resource {
		use rdf_types::{Literal, LiteralType};
		Term::Literal(Literal::new(
			value.to_owned(),
			LiteralType::Any(type_.to_owned()),
		))
	}

	fn anonymous_resource(&mut self) -> Self::Resource {
		Term::Id(self.generator.next(&mut ()))
	}
}

impl<'a, V, I> Context for super::BuilderWithInterpretationMut<'a, V, I>
where
	V: VocabularyMut,
	I: IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal> + InterpretationMut<V>,
	I::Resource: Clone + Eq + Ord + std::fmt::Debug,
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

	fn literal_resource(&mut self, value: &str, type_: &Iri) -> Self::Resource {
		use rdf_types::{Literal, LiteralType};
		let type_ = self.vocabulary.insert(type_);
		let l = self
			.vocabulary
			.insert_owned_literal(Literal::new(value.to_owned(), LiteralType::Any(type_)));
		self.interpretation.interpret_literal(l)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompactIri(pub IriRefBuf);

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

impl From<IriBuf> for CompactIri {
	fn from(value: IriBuf) -> Self {
		Self(value.into())
	}
}

impl<C: Context> Build<C> for CompactIri {
	type Target = C::Resource;

	/// Build this layout fragment using the given `context` in the given
	/// `scope`.
	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let iri = self.resolve(scope)?;
		Ok(context.iri_resource(&iri))
	}
}

/// Scope providing the active base IRI, the currently defined IRI prefixes and
/// variables.
#[derive(Debug, Default, Clone)]
pub struct Scope {
	/// Active base IRI.
	base_iri: Option<IriBuf>,

	/// Current IRI prefixes.
	iri_prefixes: HashMap<String, IriBuf>,

	/// Defined variables.
	variables: HashMap<String, u32>,

	/// Defined variable count.
	variable_count: u32,
}

impl Scope {
	/// Creates a new scope by combining this scope with the given layout
	/// `header` which may define a new base IRI, new IRI prefixes and new
	/// variables.
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

	/// Creates a new scope extending this scope with the given list of new
	/// variables.
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

	/// Creates a new scope based on this scope by clearing all defined
	/// variables.
	pub fn without_variables(&self) -> Self {
		Self {
			base_iri: self.base_iri.clone(),
			iri_prefixes: self.iri_prefixes.clone(),
			variables: HashMap::new(),
			variable_count: 0,
		}
	}

	/// Defines a new variable.
	///
	/// The new variable will be assigned a new unique index.
	/// If a variable with the same name already exists it will be shadowed.
	pub fn bind(&mut self, name: &str) {
		self.variables.insert(name.to_owned(), self.variable_count);
		self.variable_count += 1;
	}

	/// Returns the unique index of the given variable.
	pub fn variable(&self, name: &str) -> Result<u32, Error> {
		self.variables
			.get(name)
			.copied()
			.ok_or_else(|| Error::UndeclaredVariable(name.to_owned()))
	}
}

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
	) -> Result<Ref<LayoutType, C::Resource>, Error>
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

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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

#[derive(Debug, thiserror::Error)]
#[error("invalid variable name `{0}`")]
pub struct InvalidVariableName<T = String>(pub T);

/// Variable name.
///
/// Subset of `str` that can serve as a variable name in the abstract syntax.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VariableName(str);

impl VariableName {
	pub const SELF: &'static Self = unsafe { Self::new_unchecked("self") };

	pub const VALUE: &'static Self = unsafe { Self::new_unchecked("value") };

	/// Parses the given string and turns it into a variable name.
	pub fn new(s: &str) -> Result<&Self, InvalidVariableName<&str>> {
		if check_variable_name(s.chars()) {
			Ok(unsafe { Self::new_unchecked(s) })
		} else {
			Err(InvalidVariableName(s))
		}
	}

	/// Converts the given string into a variable name without parsing.
	///
	/// # Safety
	///
	/// The input string **must** be a valid variable name.
	pub const unsafe fn new_unchecked(s: &str) -> &Self {
		std::mem::transmute(s)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl ToOwned for VariableName {
	type Owned = VariableNameBuf;

	fn to_owned(&self) -> Self::Owned {
		VariableNameBuf(self.0.to_owned())
	}
}

impl PartialEq<str> for VariableName {
	fn eq(&self, other: &str) -> bool {
		&self.0 == other
	}
}

impl std::ops::Deref for VariableName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Borrow<str> for VariableName {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for VariableName {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

fn check_variable_name<C: Iterator<Item = char>>(mut chars: C) -> bool {
	match chars.next() {
		Some(c) if c.is_ascii_digit() || is_pn_char_u(c) => {
			for c in chars {
				if !is_pn_char(c) {
					return false;
				}
			}

			true
		}
		_ => false,
	}
}

fn is_pn_char_base(c: char) -> bool {
	matches!(c, 'A'..='Z' | 'a'..='z' | '\u{00c0}'..='\u{00d6}' | '\u{00d8}'..='\u{00f6}' | '\u{00f8}'..='\u{02ff}' | '\u{0370}'..='\u{037d}' | '\u{037f}'..='\u{1fff}' | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' | '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' | '\u{fdf0}'..='\u{fffd}' | '\u{10000}'..='\u{effff}')
}

fn is_pn_char_u(c: char) -> bool {
	is_pn_char_base(c) || matches!(c, '_' | ':')
}

fn is_pn_char(c: char) -> bool {
	is_pn_char_u(c)
		|| matches!(c, '-' | '0'..='9' | '\u{00b7}' | '\u{0300}'..='\u{036f}' | '\u{203f}'..='\u{2040}')
}

/// Variable name buffer.
///
/// Subset of [`String`] that can serve as a variable name in the abstract syntax.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableNameBuf(String);

impl VariableNameBuf {
	/// Parses the given string to create a new variable name.
	pub fn new(s: String) -> Result<Self, InvalidVariableName> {
		if check_variable_name(s.chars()) {
			Ok(Self(s))
		} else {
			Err(InvalidVariableName(s))
		}
	}

	/// Converts the given string into a variable name without parsing.
	///
	/// # Safety
	///
	/// The input string **must** be a valid variable name.
	pub unsafe fn new_unchecked(s: String) -> Self {
		Self(s)
	}

	pub fn as_variable_name(&self) -> &VariableName {
		unsafe { VariableName::new_unchecked(&self.0) }
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl std::ops::Deref for VariableNameBuf {
	type Target = VariableName;

	fn deref(&self) -> &Self::Target {
		self.as_variable_name()
	}
}

impl Borrow<VariableName> for VariableNameBuf {
	fn borrow(&self) -> &VariableName {
		self.as_variable_name()
	}
}

impl AsRef<VariableName> for VariableNameBuf {
	fn as_ref(&self) -> &VariableName {
		self.as_variable_name()
	}
}

impl PartialEq<str> for VariableNameBuf {
	fn eq(&self, other: &str) -> bool {
		self.0 == other
	}
}

impl fmt::Display for VariableNameBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pattern {
	Var(VariableNameBuf),
	Iri(CompactIri),
}

impl Pattern {
	pub fn is_variable(&self, name: &str) -> bool {
		match self {
			Self::Var(x) => x == name,
			Self::Iri(_) => false,
		}
	}

	pub fn default_head() -> Self {
		Self::Var(VariableNameBuf("self".to_string()))
	}

	pub fn is_default_head(&self) -> bool {
		match self {
			Self::Var(x) => x == "self",
			_ => false,
		}
	}

	pub fn default_tail() -> Self {
		Self::Iri(CompactIri(RDF_NIL.to_owned().into()))
	}

	pub fn is_default_tail(&self) -> bool {
		match self {
			Self::Iri(CompactIri(iri_ref)) => iri_ref == RDF_NIL,
			_ => false,
		}
	}

	pub fn to_term(&self, scope: &Scope) -> Result<Term, Error> {
		match self {
			Self::Var(name) => Ok(Term::blank(BlankIdBuf::from_suffix(name).unwrap())),
			Self::Iri(compact_iri) => compact_iri.resolve(scope).map(Term::iri),
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
					Ok(blank_id) => Ok(Pattern::Var(VariableNameBuf(blank_id.suffix().to_owned()))),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl<T> From<Vec<T>> for OneOrMany<T> {
	fn from(value: Vec<T>) -> Self {
		if value.len() == 1 {
			Self::One(value.into_iter().next().unwrap())
		} else {
			Self::Many(value)
		}
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

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let mut result = BTreeMap::new();

		for (prop, value) in &self.0 {
			let prop = prop.build(context, scope)?;
			let value = value.build(context, scope)?;
			result.insert(prop, value);
		}

		Ok(result)
	}
}

/// RDF Resource description.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Resource {
	/// Boolean value.
	Boolean(bool),

	/// Decimal number value.
	Number(i64),

	/// Simple string literal.
	String(String),

	/// Typed string.
	TypedString(TypedString),
}

impl<C: Context> Build<C> for Resource {
	type Target = C::Resource;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			&Self::Boolean(b) => {
				let value = if b { "true" } else { "false" };

				Ok(context.literal_resource(value, XSD_BOOLEAN))
			}
			&Self::Number(n) => {
				let value: xsd_types::Decimal = n.into();
				let type_ = value.decimal_type();

				Ok(context.literal_resource(value.lexical_representation(), type_.iri()))
			}
			Self::String(value) => Ok(context.literal_resource(value, XSD_STRING)),
			Self::TypedString(t) => t.build(context, scope),
		}
	}
}

/// Typed string literal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypedString {
	/// Literal value.
	pub value: String,

	/// Literal type.
	#[serde(rename = "type")]
	pub type_: CompactIri,
}

impl<C: Context> Build<C> for TypedString {
	type Target = C::Resource;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let type_ = self.type_.resolve(scope)?;
		Ok(context.literal_resource(&self.value, &type_))
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dataset(Vec<Quad>);

impl Dataset {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl From<Vec<Quad>> for Dataset {
	fn from(value: Vec<Quad>) -> Self {
		Self(value)
	}
}

impl<C: Context> Build<C> for Dataset
where
	C::Resource: Clone,
{
	type Target = crate::Dataset<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let mut dataset = crate::Dataset::new();
		for quad in &self.0 {
			dataset.insert(quad.build(context, scope)?);
		}

		Ok(dataset)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Quad(
	pub Pattern,
	pub Pattern,
	pub Pattern,
	#[serde(default, skip_serializing_if = "Option::is_none")] pub Option<Pattern>,
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

impl<C: Context> Build<C> for LayoutHeader
where
	C::Resource: Clone,
{
	type Target = (BuiltLayoutHeader<C::Resource>, Scope);

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariantInput(OneOrMany<Pattern>);

impl VariantInput {
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
		slice.len() == 1 && slice[0].is_variable("self")
	}
}

impl Default for VariantInput {
	fn default() -> Self {
		Self(OneOrMany::One(Pattern::Var(VariableName::SELF.to_owned())))
	}
}

impl From<Vec<Pattern>> for VariantInput {
	fn from(value: Vec<Pattern>) -> Self {
		Self(value.into())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueFormatOrLayout {
	Format(ValueFormat),
	Layout(LayoutRef),
}

impl<C: Context> Build<C> for ValueFormatOrLayout
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Format(f) => f.build(context, scope),
			Self::Layout(layout) => Ok(crate::ValueFormat {
				layout: layout.build(context, scope)?,
				input: vec![Pattern::Var(VariableName::VALUE.to_owned()).build(context, scope)?],
				graph: None,
			}),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueFormat {
	pub layout: LayoutRef,

	#[serde(default, skip_serializing_if = "ValueInput::is_default")]
	pub input: ValueInput,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub graph: Option<Option<Pattern>>,
}

impl<C: Context> Build<C> for ValueFormat
where
	C::Resource: Clone,
{
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariantFormatOrLayout {
	Format(VariantFormat),
	Layout(LayoutRef),
}

impl<C: Context> Build<C> for VariantFormatOrLayout
where
	C::Resource: Clone,
{
	type Target = crate::ValueFormat<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Format(f) => f.build(context, scope),
			Self::Layout(layout) => Ok(crate::ValueFormat {
				layout: layout.build(context, scope)?,
				input: vec![Pattern::Var(VariableName::SELF.to_owned()).build(context, scope)?],
				graph: None,
			}),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantFormat {
	pub layout: LayoutRef,

	#[serde(default, skip_serializing_if = "VariantInput::is_default")]
	pub input: VariantInput,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub graph: Option<Option<Pattern>>,
}

impl<C: Context> Build<C> for VariantFormat
where
	C::Resource: Clone,
{
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
			#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl<C: Context> Build<C> for LiteralLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::LiteralLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Data(l) => Ok(abs::layout::LiteralLayout::Data(l.build(context, scope)?)),
			Self::Id(l) => Ok(abs::layout::LiteralLayout::Id(l.build(context, scope)?)),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl<C: Context> Build<C> for DataLayout
where
	C::Resource: Clone,
{
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitLayout {
	#[serde(rename = "type")]
	pub type_: UnitLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(rename = "const", default, skip_serializing_if = "Value::is_unit")]
	pub const_: Value,
}

impl<C: Context> Build<C> for UnitLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::UnitLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, _) = self.header.build(context, scope)?;
		Ok(abs::layout::UnitLayout {
			input: header.input,
			intro: header.intro,
			dataset: header.dataset,
			const_: self.const_.clone(),
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanLayout {
	#[serde(rename = "type")]
	pub type_: BooleanLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub resource: Option<Pattern>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub datatype: Option<CompactIri>,
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

impl<C: Context> Build<C> for BooleanLayout
where
	C::Resource: Clone,
{
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
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberLayout {
	#[serde(rename = "type")]
	pub type_: NumberLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub resource: Option<Pattern>,

	pub datatype: CompactIri,
}

impl<C: Context> Build<C> for NumberLayout
where
	C::Resource: Clone,
{
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
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByteStringLayout {
	#[serde(rename = "type")]
	pub type_: ByteStringLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub resource: Option<Pattern>,

	pub datatype: CompactIri,
}

impl<C: Context> Build<C> for ByteStringLayout
where
	C::Resource: Clone,
{
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
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextStringLayout {
	#[serde(rename = "type")]
	pub type_: TextStringLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pattern: Option<RegExp>,

	pub resource: Option<Pattern>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub datatype: Option<CompactIri>,
}

impl<C: Context> Build<C> for TextStringLayout
where
	C::Resource: Clone,
{
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
			properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdLayout {
	#[serde(rename = "type")]
	pub type_: IdLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pattern: Option<RegExp>,

	pub resource: Option<Pattern>,
}

impl<C: Context> Build<C> for IdLayout
where
	C::Resource: Clone,
{
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
			properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductLayout {
	#[serde(rename = "type")]
	pub type_: ProductLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub fields: BTreeMap<String, Field>,
}

impl<C: Context> Build<C> for ProductLayout
where
	C::Resource: Clone,
{
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
					required: field.required,
				},
			);
		}

		Ok(abs::layout::ProductLayout {
			input: header.input,
			intro: header.intro,
			fields,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl From<Vec<String>> for ValueIntro {
	fn from(value: Vec<String>) -> Self {
		Self(value.into())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	pub intro: ValueIntro,

	pub value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub property: Option<Pattern>,

	#[serde(default, skip_serializing_if = "super::is_false")]
	pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SumLayout {
	#[serde(rename = "type")]
	pub type_: SumLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub variants: BTreeMap<String, Variant>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
	#[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
	pub intro: OneOrMany<String>,

	pub value: VariantFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,
}

impl<C: Context> Build<C> for SumLayout
where
	C::Resource: Clone,
{
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
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl<C: Context> Build<C> for ListLayout
where
	C::Resource: Clone,
{
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderedListLayout {
	#[serde(rename = "type")]
	pub type_: OrderedListLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub node: ListNodeOrLayout,

	#[serde(
		default = "Pattern::default_head",
		skip_serializing_if = "Pattern::is_default_head"
	)]
	pub head: Pattern,

	#[serde(
		default = "Pattern::default_tail",
		skip_serializing_if = "Pattern::is_default_tail"
	)]
	pub tail: Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListNodeOrLayout {
	ListNode(ListNode),
	Layout(LayoutRef),
}

fn default_list_dataset<C: Context>(
	context: &mut C,
	head: u32,
	rest: u32,
	first: crate::Pattern<C::Resource>,
) -> BTreeDataset<crate::Pattern<C::Resource>>
where
	C::Resource: Clone,
{
	let mut dataset = BTreeDataset::new();

	dataset.insert(rdf_types::Quad(
		crate::Pattern::Var(head),
		crate::Pattern::Resource(context.iri_resource(RDF_FIRST)),
		first,
		None,
	));

	dataset.insert(rdf_types::Quad(
		crate::Pattern::Var(head),
		crate::Pattern::Resource(context.iri_resource(RDF_REST)),
		crate::Pattern::Var(rest),
		None,
	));

	dataset
}

impl<C: Context> Build<C> for ListNodeOrLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::list::ordered::NodeLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let head = scope.variable_count;
		let rest = head + 1;
		let first = rest + 1;
		match self {
			Self::ListNode(n) => n.build(context, scope),
			Self::Layout(layout_ref) => Ok(abs::layout::list::ordered::NodeLayout {
				intro: 1u32,
				value: crate::ValueFormat {
					layout: layout_ref.build(context, scope)?,
					input: vec![crate::Pattern::Var(first)],
					graph: None,
				},
				dataset: default_list_dataset(context, head, rest, crate::Pattern::Var(first)),
			}),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListNode {
	#[serde(
		default = "ListNode::default_head",
		skip_serializing_if = "ListNode::is_default_head"
	)]
	pub head: String,

	#[serde(
		default = "ListNode::default_rest",
		skip_serializing_if = "ListNode::is_default_rest"
	)]
	pub rest: String,

	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	pub intro: ValueIntro,

	pub value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub dataset: Option<Dataset>,
}

impl ListNode {
	pub fn default_head() -> String {
		"head".to_string()
	}

	pub fn is_default_head(value: &str) -> bool {
		value == "head"
	}

	pub fn default_rest() -> String {
		"rest".to_string()
	}

	pub fn is_default_rest(value: &str) -> bool {
		value == "rest"
	}
}

impl<C: Context> Build<C> for OrderedListLayout
where
	C::Resource: Clone,
{
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
			extra_properties: header.properties,
		})
	}
}

impl<C: Context> Build<C> for ListNode
where
	C::Resource: Clone,
{
	type Target = abs::layout::list::ordered::NodeLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let head = scope.variable_count;
		let rest = head + 1;
		let scope = scope.with_intro(
			[&self.head, &self.rest]
				.into_iter()
				.chain(self.intro.as_slice()),
		)?;
		Ok(abs::layout::list::ordered::NodeLayout {
			intro: self.intro.len() as u32,
			value: self.value.build(context, &scope)?,
			dataset: match &self.dataset {
				Some(dataset) => dataset.build(context, &scope)?,
				None => match &self.value {
					ValueFormatOrLayout::Format(f) => {
						if f.input.len() == 1 {
							let first = f.input.first().unwrap().build(context, &scope)?;
							default_list_dataset(context, head, rest, first)
						} else {
							BTreeDataset::new()
						}
					}
					ValueFormatOrLayout::Layout(_) => {
						let first = crate::Pattern::Var(scope.variable("value")?);
						default_list_dataset(context, head, rest, first)
					}
				},
			},
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnorderedListLayout {
	#[serde(rename = "type")]
	pub type_: UnorderedListLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub item: ListItem,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListItem {
	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	pub intro: ValueIntro,

	pub value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub property: Option<Pattern>,
}

impl<C: Context> Build<C> for UnorderedListLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::UnorderedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let subject = if self.header.input.is_empty() {
			None
		} else {
			Some(0)
		};

		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::UnorderedListLayout {
			input: header.input,
			intro: header.intro,
			item: self.item.build(context, &scope, subject)?,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}

impl ListItem {
	fn build<C: Context>(
		&self,
		context: &mut C,
		scope: &Scope,
		subject: Option<u32>,
	) -> Result<abs::layout::list::ItemLayout<C::Resource>, Error>
	where
		C::Resource: Clone,
	{
		let object = if self.intro.is_empty() {
			None
		} else {
			Some(scope.variable_count)
		};

		let scope = scope.with_intro(self.intro.as_slice())?;

		let mut dataset = self.dataset.build(context, &scope)?;
		if let Some(prop) = &self.property {
			match subject {
				Some(subject) => match object {
					Some(object) => {
						let prop = prop.build(context, &scope)?;
						dataset.insert(rdf_types::Quad(
							crate::Pattern::Var(subject),
							prop,
							crate::Pattern::Var(object),
							None,
						));
					}
					None => return Err(Error::NoPropertyObject),
				},
				None => return Err(Error::NoPropertySubject),
			}
		}

		Ok(abs::layout::list::ItemLayout {
			intro: self.intro.len() as u32,
			value: self.value.build(context, &scope)?,
			dataset,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizedListLayout {
	#[serde(rename = "type")]
	pub type_: SizedListLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub items: Vec<ListItem>,
}

impl<C: Context> Build<C> for SizedListLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::SizedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let subject = if self.header.input.is_empty() {
			None
		} else {
			Some(0)
		};

		let (header, scope) = self.header.build(context, scope)?;

		let mut items = Vec::with_capacity(self.items.len());
		for item in &self.items {
			items.push(item.build(context, &scope, subject)?)
		}

		Ok(abs::layout::SizedListLayout {
			input: header.input,
			intro: header.intro,
			items,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntersectionLayout {
	#[serde(rename = "type")]
	pub type_: IntersectionLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,
}

impl<C: Context> Build<C> for IntersectionLayout {
	type Target = Vec<Ref<LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, Error> {
		unimplemented!()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnionLayout {
	#[serde(rename = "type")]
	pub type_: UnionLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,
}

impl<C: Context> Build<C> for UnionLayout {
	type Target = Vec<Ref<LayoutType, C::Resource>>;

	fn build(&self, _context: &mut C, _scope: &Scope) -> Result<Self::Target, Error> {
		unimplemented!()
	}
}
