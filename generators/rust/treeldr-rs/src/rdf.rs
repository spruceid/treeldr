use educe::Educe;
use iref::{Iri, IriBuf};
use langtag::{LanguageTag, LanguageTagBuf};
use rdf_types::{
	literal, BlankId, BlankIdInterpretationMut, BlankIdVocabularyMut, IriInterpretationMut,
	IriVocabulary, IriVocabularyMut, LanguageTagVocabulary, LanguageTagVocabularyMut, Literal,
	LiteralInterpretationMut, LiteralVocabularyMut,
};

pub type RdfType<V> =
	literal::Type<<V as IriVocabulary>::Iri, <V as LanguageTagVocabulary>::LanguageTag>;

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

	pub fn vocabulary_literal(
		&mut self,
		literal: Literal<literal::Type<&Iri, LanguageTag>, &str>,
	) -> V::Literal
	where
		V: IriVocabularyMut
			+ LanguageTagVocabularyMut
			+ LiteralVocabularyMut<Value = String, Type = RdfType<V>>,
		I: IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal>,
	{
		let value = (*literal.value()).to_owned();
		let type_ = match literal.type_() {
			literal::Type::Any(iri) => literal::Type::Any(self.vocabulary.insert(iri)),
			literal::Type::LangString(tag) => {
				literal::Type::LangString(self.vocabulary.insert_language_tag(*tag))
			}
		};

		self.vocabulary
			.insert_owned_literal(Literal::new(value, type_))
	}

	pub fn vocabulary_literal_owned(
		&mut self,
		literal: Literal<literal::Type<IriBuf, LanguageTagBuf>, String>,
	) -> V::Literal
	where
		V: IriVocabularyMut
			+ LanguageTagVocabularyMut
			+ LiteralVocabularyMut<Value = String, Type = RdfType<V>>,
		I: IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal>,
	{
		let (value, type_) = literal.into_parts();
		let type_ = match type_ {
			literal::Type::Any(iri) => literal::Type::Any(self.vocabulary.insert_owned(iri)),
			literal::Type::LangString(tag) => {
				literal::Type::LangString(self.vocabulary.insert_owned_language_tag(tag))
			}
		};

		self.vocabulary
			.insert_owned_literal(Literal::new(value, type_))
	}

	pub fn interpret_literal(
		&mut self,
		literal: Literal<literal::Type<&Iri, LanguageTag>, &str>,
	) -> I::Resource
	where
		V: IriVocabularyMut
			+ LanguageTagVocabularyMut
			+ LiteralVocabularyMut<Value = String, Type = RdfType<V>>,
		I: IriInterpretationMut<V::Iri> + LiteralInterpretationMut<V::Literal>,
	{
		let l = self.vocabulary_literal(literal);
		self.interpretation.interpret_literal(l)
	}
}
