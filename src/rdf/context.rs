use std::borrow::Cow;

use iref::Iri;
use langtag::LangTagBuf;
use rdf_types::{
	interpretation::{
		IriInterpretation, IriInterpretationMut, LiteralInterpretation, LiteralInterpretationMut,
		ReverseIriInterpretation, ReverseLiteralInterpretation,
	},
	vocabulary::{ExtractFromVocabulary, IriVocabulary, LiteralVocabulary},
	Interpretation, InterpretationMut, Literal, LiteralType, Term, Vocabulary, VocabularyMut,
};
use serde_seeded::de::Seed;

use super::{TryUnifyInterpretation, UnifyLiteralInterpretation};

pub trait RdfContext {
	type Resource;
	type Vocabulary: Vocabulary;
	type Interpretation: Interpretation<Resource = Self::Resource>
		+ IriInterpretation<<Self::Vocabulary as IriVocabulary>::Iri>
		+ ReverseIriInterpretation<Iri = <Self::Vocabulary as IriVocabulary>::Iri>
		+ ReverseLiteralInterpretation<Literal = <Self::Vocabulary as LiteralVocabulary>::Literal>;

	fn vocabulary(&self) -> &Self::Vocabulary;

	fn interpretation(&self) -> &Self::Interpretation;

	fn get(&self, iri: &Iri) -> Option<Self::Resource> {
		self.interpretation()
			.lexical_iri_interpretation(self.vocabulary(), iri)
	}

	fn term_of(&self, r: &Self::Resource) -> Option<Term> {
		match self.interpretation().iris_of(r).next() {
			Some(i) => Some(Term::iri(self.vocabulary().iri(i).unwrap().to_owned())),
			None => self.interpretation().literals_of(r).next().map(|l| {
				Term::Literal(
					self.vocabulary()
						.literal(l)
						.unwrap()
						.extract_from_vocabulary(self.vocabulary()),
				)
			}),
		}
	}
}

pub trait RdfContextMut:
	RdfContext<Interpretation: InterpretationMut<Self::Vocabulary> + TryUnifyInterpretation>
{
	fn new_resource(&mut self) -> Self::Resource;

	fn unify(
		&mut self,
		a: &Self::Resource,
		b: &Self::Resource,
	) -> Result<(), <Self::Interpretation as TryUnifyInterpretation>::UnificationError>;

	fn insert_iri(&mut self, iri: Cow<Iri>) -> Self::Resource;

	fn insert_literal(&mut self, value: String, r#type: Cow<Iri>) -> Self::Resource;

	fn insert_lang_string(&mut self, value: String, lang: LangTagBuf) -> Self::Resource;

	fn deserialize_seed<T>(&mut self) -> Seed<Self, T> {
		Seed::new(self)
	}
}

/// RDF context, providing the RDF vocabulary and interpretation.
pub struct Rdf<V = (), I = UnifyLiteralInterpretation> {
	/// Vocabulary storing the lexical representations of terms.
	pub vocabulary: V,

	/// RDF interpretation, mapping resources to terms.
	pub interpretation: I,
}

impl Default for Rdf {
	fn default() -> Self {
		Self {
			vocabulary: (),
			interpretation: UnifyLiteralInterpretation::new(),
		}
	}
}

impl<V, I, R> RdfContext for Rdf<V, I>
where
	V: Vocabulary,
	I: Interpretation<Resource = R>
		+ IriInterpretation<V::Iri>
		+ LiteralInterpretation<V::Literal>
		+ ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Resource = I::Resource;
	type Interpretation = I;
	type Vocabulary = V;

	fn vocabulary(&self) -> &V {
		&self.vocabulary
	}

	fn interpretation(&self) -> &I {
		&self.interpretation
	}
}

impl<V, I, R> RdfContextMut for Rdf<V, I>
where
	V: VocabularyMut,
	I: InterpretationMut<V, Resource = R>
		+ IriInterpretation<V::Iri>
		+ IriInterpretationMut<V::Iri>
		+ ReverseIriInterpretation<Iri = V::Iri>
		+ LiteralInterpretation<V::Literal>
		+ LiteralInterpretationMut<V::Literal>
		+ ReverseLiteralInterpretation<Literal = V::Literal>
		+ TryUnifyInterpretation,
{
	fn new_resource(&mut self) -> R {
		self.interpretation.new_resource(&mut self.vocabulary)
	}

	fn unify(&mut self, a: &R, b: &R) -> Result<(), I::UnificationError> {
		self.interpretation.try_unify(a, b)
	}

	fn insert_iri(&mut self, iri: Cow<Iri>) -> R {
		let iri = match iri {
			Cow::Owned(iri) => self.vocabulary.insert_owned(iri),
			Cow::Borrowed(iri) => self.vocabulary.insert(iri),
		};

		self.interpretation.interpret_iri(iri)
	}

	fn insert_literal(&mut self, value: String, r#type: Cow<Iri>) -> R {
		let type_ = match r#type {
			Cow::Owned(iri) => self.vocabulary.insert_owned(iri),
			Cow::Borrowed(iri) => self.vocabulary.insert(iri),
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
