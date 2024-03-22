use educe::Educe;
use iref::Iri;
use rdf_types::{
	interpretation::{
		BlankIdInterpretationMut, IriInterpretation, IriInterpretationMut, LiteralInterpretation,
		LiteralInterpretationMut,
	},
	vocabulary::{
		BlankIdVocabularyMut, IriVocabulary, IriVocabularyMut, LiteralVocabulary,
		LiteralVocabularyMut,
	},
	BlankId, Literal, LiteralType,
};

#[derive(Educe)]
#[educe(Clone, Copy)]
pub struct RdfContext<'a, V, I> {
	pub vocabulary: &'a V,
	pub interpretation: &'a I,
}

impl<'a, V, I> RdfContext<'a, V, I> {
	pub fn new(vocabulary: &'a V, interpretation: &'a I) -> Self {
		Self {
			vocabulary,
			interpretation,
		}
	}

	pub fn iri_interpretation(&self, iri: &Iri) -> Option<I::Resource>
	where
		V: IriVocabulary,
		I: IriInterpretation<V::Iri>,
	{
		self.interpretation
			.lexical_iri_interpretation(self.vocabulary, iri)
	}

	pub fn literal_interpretation(&self, literal: Literal<&Iri>) -> Option<I::Resource>
	where
		V: IriVocabulary + LiteralVocabulary,
		I: IriInterpretation<V::Iri> + LiteralInterpretation<V::Literal>,
	{
		let (value, type_) = literal.into_parts();
		let type_ = match type_ {
			LiteralType::Any(iri) => LiteralType::Any(self.vocabulary.get(iri)?),
			LiteralType::LangString(tag) => LiteralType::LangString(tag),
		};

		let lit = self
			.vocabulary
			.get_literal(Literal::new(value, type_).as_ref())?;
		self.interpretation.literal_interpretation(&lit)
	}
}

pub struct RdfContextMut<'a, V, I> {
	pub vocabulary: &'a mut V,
	pub interpretation: &'a mut I,
}

impl<'a, V, I> RdfContextMut<'a, V, I> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
		}
	}

	pub fn interpret_iri(&mut self, iri: &Iri) -> I::Resource
	where
		V: IriVocabularyMut,
		I: IriInterpretationMut<V::Iri>,
	{
		self.interpretation
			.interpret_iri(self.vocabulary.insert(iri))
	}

	pub fn interpret_blank_id(&mut self, blank_id: &BlankId) -> I::Resource
	where
		V: BlankIdVocabularyMut,
		I: BlankIdInterpretationMut<V::BlankId>,
	{
		self.interpretation
			.interpret_blank_id(self.vocabulary.insert_blank_id(blank_id))
	}

	pub fn vocabulary_literal(&mut self, literal: Literal<&Iri>) -> V::Literal
	where
		V: IriVocabularyMut + LiteralVocabularyMut,
		I: IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal>,
	{
		let literal = literal.insert_type_into_vocabulary(self.vocabulary);
		self.vocabulary.insert_owned_literal(literal)
	}

	pub fn vocabulary_literal_owned(&mut self, literal: Literal) -> V::Literal
	where
		V: IriVocabularyMut + LiteralVocabularyMut,
		I: IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal>,
	{
		let literal = literal.insert_type_into_vocabulary(self.vocabulary);
		self.vocabulary.insert_owned_literal(literal)
	}

	pub fn interpret_literal(&mut self, literal: Literal<&Iri>) -> I::Resource
	where
		V: IriVocabularyMut + LiteralVocabularyMut,
		I: IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal>,
	{
		let l = self.vocabulary_literal(literal);
		self.interpretation.interpret_literal(l)
	}
}
