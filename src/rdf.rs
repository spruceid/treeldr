use disjoint_collections::DisjointHashMap;
use educe::Educe;
use interpretation::{
	IriInterpretationMut, LiteralInterpretation, LiteralInterpretationMut,
	ReverseIriInterpretation, ReverseLiteralInterpretation,
};
use langtag::LangTagBuf;
use rdf_types::{interpretation::IriInterpretation, vocabulary::IriVocabulary};
use std::{borrow::Cow, collections::BTreeMap};

pub use iref;
pub use rdf_types::*;

pub use iref::{Iri, IriBuf};
use vocabulary::{ExtractFromVocabulary, LiteralVocabulary};

pub trait RdfContext<R> {
	type Vocabulary: Vocabulary;
	type Interpretation: Interpretation<Resource = R>
		+ IriInterpretation<<Self::Vocabulary as IriVocabulary>::Iri>
		+ ReverseIriInterpretation<Iri = <Self::Vocabulary as IriVocabulary>::Iri>
		+ ReverseLiteralInterpretation<Literal = <Self::Vocabulary as LiteralVocabulary>::Literal>;

	fn vocabulary(&self) -> &Self::Vocabulary;

	fn interpretation(&self) -> &Self::Interpretation;

	fn get(&self, iri: &Iri) -> Option<R> {
		self.interpretation()
			.lexical_iri_interpretation(self.vocabulary(), iri)
	}

	fn term_of(&self, r: &R) -> Option<Term> {
		match self.interpretation().iris_of(r).next() {
			Some(i) => Some(Term::iri(self.vocabulary().iri(i).unwrap().to_owned())),
			None => match self.interpretation().literals_of(r).next() {
				Some(l) => Some(Term::Literal(
					self.vocabulary()
						.literal(l)
						.unwrap()
						.extract_from_vocabulary(self.vocabulary()),
				)),
				None => None,
			},
		}
	}
}

#[derive(Educe)]
#[educe(Default)]
pub struct DebugSubstitution<R> {
	generator: generator::Blank,
	resources: BTreeMap<R, BlankIdBuf>,
	variables: BTreeMap<u32, BlankIdBuf>,
}

impl<R> DebugSubstitution<R> {
	pub fn new() -> Self {
		Self {
			generator: generator::Blank::new(),
			resources: BTreeMap::new(),
			variables: BTreeMap::new(),
		}
	}

	pub fn new_resource(&mut self) -> Term {
		Term::blank(self.generator.next_blank_id())
	}
}

impl<R: Ord> DebugSubstitution<R> {
	pub fn insert_resource(&mut self, rdf: &impl RdfContext<R>, r: &R) -> Term
	where
		R: Clone,
	{
		rdf.term_of(r).unwrap_or_else(|| {
			Term::blank(
				self.resources
					.entry(r.clone())
					.or_insert_with(|| self.generator.next_blank_id())
					.clone(),
			)
		})
	}

	pub fn insert_var(&mut self, x: u32) -> Term {
		Term::blank(
			self.variables
				.entry(x)
				.or_insert_with(|| self.generator.next_blank_id())
				.clone(),
		)
	}
}

pub trait RdfContextMut<R>:
	RdfContext<R, Interpretation: InterpretationMut<Self::Vocabulary> + TryUnifyInterpretation>
{
	fn new_resource(&mut self) -> R;

	fn unify(
		&mut self,
		a: &R,
		b: &R,
	) -> Result<(), <Self::Interpretation as TryUnifyInterpretation>::UnificationError>;

	fn insert_iri(&mut self, iri: Cow<Iri>) -> R;

	fn insert_literal(&mut self, value: String, type_: Cow<Iri>) -> R;

	fn insert_lang_string(&mut self, value: String, lang: LangTagBuf) -> R;
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

impl<V, I, R> RdfContext<R> for Rdf<V, I>
where
	V: Vocabulary,
	I: Interpretation<Resource = R>
		+ IriInterpretation<V::Iri>
		+ LiteralInterpretation<V::Literal>
		+ ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
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

	fn insert_literal(&mut self, value: String, type_: Cow<Iri>) -> R {
		let type_ = match type_ {
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

pub trait TryUnifyInterpretation: Interpretation {
	type UnificationError;

	fn try_unify(
		&mut self,
		a: &Self::Resource,
		b: &Self::Resource,
	) -> Result<(), Self::UnificationError>;
}

#[derive(Default)]
pub struct UnifyLiteralInterpretation {
	generator: generator::Blank,
	classes: DisjointHashMap<BlankIdBuf, Term>,
}

impl UnifyLiteralInterpretation {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn map_term(&self, t: Term) -> Term {
		match t {
			Term::Id(Id::Blank(b)) => match self.classes.get(&b) {
				Some(t) => t.clone(),
				_ => Term::Id(Id::Blank(b)),
			},
			t => t,
		}
	}
}

impl Interpretation for UnifyLiteralInterpretation {
	type Resource = Term;
}

pub struct ResourceUnificationError;

impl TryUnifyInterpretation for UnifyLiteralInterpretation {
	type UnificationError = ResourceUnificationError;

	fn try_unify(
		&mut self,
		a: &Self::Resource,
		b: &Self::Resource,
	) -> Result<(), ResourceUnificationError> {
		match (a, b) {
			(Term::Id(Id::Blank(a)), Term::Id(Id::Blank(b))) => {
				self.classes.try_merge(a, b, |a, b| match (a, b) {
					(Term::Id(Id::Blank(a)), Term::Id(Id::Blank(_))) => Ok(Term::Id(Id::Blank(a))),
					(Term::Id(Id::Blank(_)), b) => Ok(b),
					(a, Term::Id(Id::Blank(_))) => Ok(a),
					(a, b) => {
						if a == b {
							Ok(a)
						} else {
							Err(ResourceUnificationError)
						}
					}
				})?;

				Ok(())
			}
			(Term::Id(Id::Blank(a)), b) => match self.classes.replace(a, b.clone()) {
				Ok(Term::Id(Id::Blank(_))) => Ok(()),
				Ok(c) => {
					if c == *b {
						Ok(())
					} else {
						Err(ResourceUnificationError)
					}
				}
				Err(_) => Err(ResourceUnificationError),
			},
			(a, Term::Id(Id::Blank(b))) => match self.classes.replace(b, a.clone()) {
				Ok(Term::Id(Id::Blank(_))) => Ok(()),
				Ok(c) => {
					if c == *a {
						Ok(())
					} else {
						Err(ResourceUnificationError)
					}
				}
				Err(_) => Err(ResourceUnificationError),
			},
			(a, b) => {
				if a == b {
					Ok(())
				} else {
					Err(ResourceUnificationError)
				}
			}
		}
	}
}

impl InterpretationMut<()> for UnifyLiteralInterpretation {
	fn new_resource(&mut self, _vocabulary: &mut ()) -> Self::Resource {
		let id = self.generator.next_blank_id();
		self.classes.insert(id.clone(), Term::blank(id.clone()));
		Term::blank(id)
	}
}

impl IriInterpretation<IriBuf> for UnifyLiteralInterpretation {
	fn iri_interpretation(&self, iri: &IriBuf) -> Option<Self::Resource> {
		Some(Term::iri(iri.clone()))
	}
}

impl IriInterpretationMut for UnifyLiteralInterpretation {
	fn interpret_iri(&mut self, iri: IriBuf) -> Self::Resource {
		Term::iri(iri)
	}
}

impl ReverseIriInterpretation for UnifyLiteralInterpretation {
	type Iri = IriBuf;

	type Iris<'a> = std::option::IntoIter<&'a IriBuf>
	where
		Self: 'a;

	fn iris_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Iris<'a> {
		match id {
			Term::Id(Id::Blank(b)) => match self.classes.get(b) {
				Some(Term::Id(Id::Iri(iri))) => Some(iri).into_iter(),
				_ => None.into_iter(),
			},
			Term::Id(Id::Iri(iri)) => Some(iri).into_iter(),
			Term::Literal(_) => None.into_iter(),
		}
	}
}

impl LiteralInterpretation<Literal> for UnifyLiteralInterpretation {
	fn literal_interpretation(&self, l: &Literal) -> Option<Self::Resource> {
		Some(Term::Literal(l.clone()))
	}
}

impl LiteralInterpretationMut for UnifyLiteralInterpretation {
	fn interpret_literal(&mut self, l: Literal) -> Self::Resource {
		Term::Literal(l)
	}
}

impl ReverseLiteralInterpretation for UnifyLiteralInterpretation {
	type Literal = Literal;

	type Literals<'a> = std::option::IntoIter<&'a Literal>
	where
		Self: 'a;

	fn literals_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Literals<'a> {
		match id {
			Term::Id(Id::Blank(b)) => match self.classes.get(b) {
				Some(Term::Literal(l)) => Some(l).into_iter(),
				_ => None.into_iter(),
			},
			Term::Id(Id::Iri(_)) => None.into_iter(),
			Term::Literal(l) => Some(l).into_iter(),
		}
	}
}
