use std::collections::HashMap;

use iref::{Iri, IriBuf, IriRefBuf};
use rdf_types::{
	interpretation::{IriInterpretationMut, LiteralInterpretationMut},
	Id, InterpretationMut, LexicalLiteralTypeRef, Term, VocabularyMut,
};

use crate::{
	abs::{self, InsertResult},
	layout::LayoutType,
	Ref,
};

use super::LayoutHeader;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
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

pub trait Context {
	type Resource: Ord + std::fmt::Debug;

	fn insert_layout(
		&mut self,
		id: Self::Resource,
		layout: abs::Layout<Self::Resource>,
	) -> InsertResult<Self::Resource>;

	fn iri_resource(&mut self, iri: &Iri) -> Self::Resource;

	fn literal_resource(&mut self, value: &str, type_: LexicalLiteralTypeRef) -> Self::Resource;

	fn anonymous_resource(&mut self) -> Self::Resource;
}

impl<G> Context for abs::BuilderWithGeneratorMut<'_, G>
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

	fn literal_resource(&mut self, value: &str, type_: LexicalLiteralTypeRef) -> Self::Resource {
		use rdf_types::{Literal, LiteralType};
		Term::Literal(Literal::new(
			value.to_owned(),
			match type_ {
				LexicalLiteralTypeRef::Any(iri) => LiteralType::Any(iri.to_owned()),
				LexicalLiteralTypeRef::LangString(tag) => LiteralType::LangString(tag.to_owned()),
			},
		))
	}

	fn anonymous_resource(&mut self) -> Self::Resource {
		Term::Id(self.generator.next(&mut ()))
	}
}

impl<V, I> Context for abs::BuilderWithInterpretationMut<'_, V, I>
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

	fn literal_resource(&mut self, value: &str, type_: LexicalLiteralTypeRef) -> Self::Resource {
		use rdf_types::{Literal, LiteralType};
		let type_ = match type_ {
			LexicalLiteralTypeRef::Any(iri) => LiteralType::Any(self.vocabulary.insert(iri)),
			LexicalLiteralTypeRef::LangString(tag) => LiteralType::LangString(tag.to_owned()),
		};
		let l = self
			.vocabulary
			.insert_owned_literal(Literal::new(value.to_owned(), type_));
		self.interpretation.interpret_literal(l)
	}

	fn anonymous_resource(&mut self) -> Self::Resource {
		self.interpretation.new_resource(self.vocabulary)
	}
}

pub trait Build<C> {
	type Target;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError>;
}

impl<C, T: Build<C>> Build<C> for Box<T> {
	type Target = T::Target;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		T::build(&**self, context, scope)
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
	pub fn with_header(&self, header: &LayoutHeader) -> Result<Self, BuildError> {
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
	) -> Result<Self, BuildError> {
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

	pub fn variable_count(&self) -> u32 {
		self.variable_count
	}

	pub fn base_iri(&self) -> Option<&Iri> {
		self.base_iri.as_deref()
	}

	pub fn iri_prefix(&self, prefix: &str) -> Option<&Iri> {
		self.iri_prefixes.get(prefix).map(IriBuf::as_iri)
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
	pub fn variable(&self, name: &str) -> Result<u32, BuildError> {
		self.variables
			.get(name)
			.copied()
			.ok_or_else(|| BuildError::UndeclaredVariable(name.to_owned()))
	}
}
