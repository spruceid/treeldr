use std::borrow::Cow;
use interpretation::{IriInterpretationMut, LiteralInterpretationMut};
use langtag::LangTagBuf;
use rdf_types::{
	interpretation::{IriInterpretation, WithGenerator}, vocabulary::IriVocabulary
};

pub use iref;
pub use rdf_types::*;

pub use iref::{Iri, IriBuf};

pub trait RdfContext<R> {
	type Vocabulary: Vocabulary;
	type Interpretation: Interpretation<Resource = R>
		+ IriInterpretation<<Self::Vocabulary as IriVocabulary>::Iri>;

	fn vocabulary(&self) -> &Self::Vocabulary;

	fn interpretation(&self) -> &Self::Interpretation;

	fn get(&self, iri: &Iri) -> Option<R> {
		self.interpretation()
			.lexical_iri_interpretation(self.vocabulary(), iri)
	}
}

pub trait RdfContextMut<R>:
	RdfContext<R, Interpretation: InterpretationMut<Self::Vocabulary>>
{
	fn new_resource(&mut self) -> R;

	fn insert_iri(&mut self, iri: Cow<Iri>) -> R;

	fn insert_literal(&mut self, value: String, type_: Cow<Iri>) -> R;

	fn insert_lang_string(&mut self, value: String, lang: LangTagBuf) -> R;
}

/// RDF context, providing the RDF vocabulary and interpretation.
pub struct Rdf<V = (), I = WithGenerator<generator::Blank>> {
	/// Vocabulary storing the lexical representations of terms.
	pub vocabulary: V,

	/// RDF interpretation, mapping resources to terms.
	pub interpretation: I,
}

impl Default for Rdf {
	fn default() -> Self {
		Self {
			vocabulary: (),
			interpretation: WithGenerator::new((), generator::Blank::new())
		}
	}
}

impl<V, I, R> RdfContext<R> for Rdf<V, I>
where
	V: Vocabulary,
	I: Interpretation<Resource = R> + IriInterpretation<V::Iri>
{
	type Interpretation = I;
	type Vocabulary = V;

	fn vocabulary(&self) -> &V {
		&self.vocabulary
	}

	fn interpretation(&self) -> &I {
		&self.interpretation
	}
}

impl<V, I, R> RdfContextMut<R> for Rdf<V, I>
where
	V: VocabularyMut,
	I: InterpretationMut<V, Resource = R> + IriInterpretation<V::Iri> + IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal>
{
	fn new_resource(&mut self) -> R {
		self.interpretation.new_resource(&mut self.vocabulary)
	}

	fn insert_iri(&mut self, iri: Cow<Iri>) -> R {
		let iri = match iri {
			Cow::Owned(iri) => self.vocabulary.insert_owned(iri),
			Cow::Borrowed(iri) => self.vocabulary.insert(iri)
		};

		self.interpretation.interpret_iri(iri)
	}

	fn insert_literal(&mut self, value: String, type_: Cow<Iri>) -> R {
		let type_ = match type_ {
			Cow::Owned(iri) => self.vocabulary.insert_owned(iri),
			Cow::Borrowed(iri) => self.vocabulary.insert(iri)
		};

		let literal = self
			.vocabulary
			.insert_owned_literal(Literal::new(value, LiteralType::Any(type_)));

		self.interpretation.interpret_literal(literal)
	}

	fn insert_lang_string(&mut self, value: String, lang: LangTagBuf) -> R {
		let literal = self
			.vocabulary
			.insert_owned_literal(Literal::new(value, LiteralType::LangString(lang)));

		self.interpretation.interpret_literal(literal)
	}
}