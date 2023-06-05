use iref::IriBuf;

use rdf_types::{BlankIdBuf, Id, Literal, Term};

pub trait Interpretation {
	type Resource;
}

pub trait IriInterpretation<I>: Interpretation {
	fn iri_interpretation(&self, iri: &I) -> Option<Self::Resource>;
}

pub trait BlankIdInterpretation<B>: Interpretation {
	fn blank_id_interpretation(&self, blank_id: &B) -> Option<Self::Resource>;
}

pub trait IdInterpretation<I>: Interpretation {
	fn id_interpretation(&self, id: &I) -> Option<Self::Resource>;
}

impl<I, B, T: IriInterpretation<I> + BlankIdInterpretation<B>> IdInterpretation<Id<I, B>> for T {
	fn id_interpretation(&self, id: &Id<I, B>) -> Option<Self::Resource> {
		match id {
			Id::Iri(i) => self.iri_interpretation(i),
			Id::Blank(b) => self.blank_id_interpretation(b),
		}
	}
}

pub trait LiteralInterpretation<L>: Interpretation {
	fn literal_interpretation(&self, literal: &L) -> Option<Self::Resource>;
}

pub trait TermInterpretation<I = Id<IriBuf, BlankIdBuf>, L = Literal>:
	IdInterpretation<I> + LiteralInterpretation<L>
{
	fn term_interpretation(&self, term: &Term<I, L>) -> Option<Self::Resource> {
		match term {
			Term::Id(id) => self.id_interpretation(id),
			Term::Literal(l) => self.literal_interpretation(l),
		}
	}
}

pub trait IriInterpretationMut<I = IriBuf>: Interpretation {
	fn interpret_iri(&mut self, iri: I) -> Self::Resource;
}

pub trait BlankIdInterpretationMut<B = BlankIdBuf>: Interpretation {
	fn interpret_blank_id(&mut self, blank_id: B) -> Self::Resource;
}

pub trait IdInterpretationMut<I>: Interpretation {
	fn interpret_id(&mut self, id: I) -> Self::Resource;
}

impl<I, B, T: IriInterpretationMut<I> + BlankIdInterpretationMut<B>> IdInterpretationMut<Id<I, B>>
	for T
{
	fn interpret_id(&mut self, id: Id<I, B>) -> T::Resource {
		match id {
			Id::Iri(i) => self.interpret_iri(i),
			Id::Blank(b) => self.interpret_blank_id(b),
		}
	}
}

pub trait LiteralInterpretationMut<L = Literal>: Interpretation {
	fn interpret_literal(&mut self, literal: L) -> Self::Resource;
}

pub trait TermInterpretationMut<I = Id<IriBuf, BlankIdBuf>, L = Literal>:
	IdInterpretationMut<I> + LiteralInterpretationMut<L>
{
	fn interpret_term(&mut self, term: Term<I, L>) -> Self::Resource {
		match term {
			Term::Id(id) => self.interpret_id(id),
			Term::Literal(l) => self.interpret_literal(l),
		}
	}
}

pub trait ReverseIdInterpretation: Interpretation {
	type Iri;
	type BlankId;

	type Iris<'a>: Iterator<Item = &'a Self::Iri>
	where
		Self: 'a,
		Self::Iri: 'a;
	type BlankIds<'a>: Iterator<Item = &'a Self::BlankId>
	where
		Self: 'a,
		Self::BlankId: 'a;

	fn iris_of(&self, id: &Self::Resource) -> Self::Iris<'_>;

	fn blank_ids_of(&self, id: &Self::Resource) -> Self::BlankIds<'_>;
}

pub trait ReverseTermInterpretation: Interpretation {
	type Id<'a>
	where
		Self: 'a;
	type Literal<'a>
	where
		Self: 'a;

	type Ids<'a>: Iterator<Item = Self::Id<'a>>
	where
		Self: 'a;
	type Literals<'a>: Iterator<Item = Self::Literal<'a>>
	where
		Self: 'a;

	fn ids_of(&self, id: &Self::Resource) -> Self::Ids<'_>;

	fn literals_of(&self, id: &Self::Resource) -> Self::Literals<'_>;

	fn terms_of(&self, id: &Self::Resource) -> TermsOf<Self> {
		TermsOf {
			ids: self.ids_of(id),
			literals: self.literals_of(id),
		}
	}
}

pub struct TermsOf<'a, I: 'a + ?Sized + ReverseTermInterpretation> {
	ids: I::Ids<'a>,
	literals: I::Literals<'a>,
}

impl<'a, I: 'a + ReverseTermInterpretation> Iterator for TermsOf<'a, I> {
	type Item = Term<I::Id<'a>, I::Literal<'a>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.ids
			.next()
			.map(Term::Id)
			.or_else(|| self.literals.next().map(Term::Literal))
	}
}

pub trait Interpret<I: Interpretation> {
	type Interpreted;

	fn interpret(self, interpretation: &mut I) -> Self::Interpreted;
}

impl<I, B, T: IdInterpretationMut<Self>> Interpret<T> for Id<I, B> {
	type Interpreted = T::Resource;

	fn interpret(self, interpretation: &mut T) -> Self::Interpreted {
		interpretation.interpret_id(self)
	}
}

impl<T, S, I: LiteralInterpretationMut<Self>> Interpret<I> for Literal<T, S> {
	type Interpreted = I::Resource;

	fn interpret(self, interpretation: &mut I) -> Self::Interpreted {
		interpretation.interpret_literal(self)
	}
}
