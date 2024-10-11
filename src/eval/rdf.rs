// use educe::Educe;

use std::borrow::Cow;

use iref::Iri;
use langtag::LangTagBuf;
use rdf_types::{
	interpretation::IriInterpretation, vocabulary::IriVocabulary, Interpretation,
	InterpretationMut, Vocabulary,
};

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

// /// RDF context, providing the RDF vocabulary and interpretation.
// #[derive(Educe)]
// #[educe(Clone, Copy)]
// pub struct RdfContext<'a, V, I> {
// 	/// Vocabulary storing the lexical representations of terms.
// 	pub vocabulary: &'a V,

// 	/// RDF interpretation, mapping resources to terms.
// 	pub interpretation: &'a I,
// }

// impl Default for RdfContext<'static, (), ()> {
// 	fn default() -> Self {
// 		RdfContext {
// 			vocabulary: &(),
// 			interpretation: &(),
// 		}
// 	}
// }

// impl<'a, V, I> RdfContext<'a, V, I> {
// 	/// Creates a new RDF context.
// 	pub fn new(vocabulary: &'a V, interpretation: &'a I) -> Self {
// 		Self {
// 			vocabulary,
// 			interpretation,
// 		}
// 	}
// }

// /// Mutable RDF context, providing the mutable RDF vocabulary and
// /// interpretation.
// pub struct RdfContextMut<'a, V, I> {
// 	/// Vocabulary storing the lexical representations of terms.
// 	pub vocabulary: &'a mut V,

// 	/// RDF interpretation, mapping resources to terms.
// 	pub interpretation: &'a mut I,
// }

// impl<'a, V, I> RdfContextMut<'a, V, I> {
// 	/// Creates a new mutable RDF context.
// 	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
// 		Self {
// 			vocabulary,
// 			interpretation,
// 		}
// 	}
// }
