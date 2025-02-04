use disjoint_collections::DisjointHashMap;
use iref::IriBuf;
use rdf_types::{
	generator,
	interpretation::{
		IriInterpretation, IriInterpretationMut, LiteralInterpretation, LiteralInterpretationMut,
		ReverseIriInterpretation, ReverseLiteralInterpretation,
	},
	BlankIdBuf, Id, Interpretation, InterpretationMut, Literal, Term,
};

pub trait TryUnifyInterpretation: Interpretation {
	type UnificationError;

	fn try_unify(
		&mut self,
		a: &Self::Resource,
		b: &Self::Resource,
	) -> Result<(), Self::UnificationError>;

	fn substitute_resource(&self, r: Self::Resource) -> Self::Resource;
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

#[derive(Debug, thiserror::Error)]
#[error("resource unification failed")]
pub struct ResourceUnificationFailed;

impl TryUnifyInterpretation for UnifyLiteralInterpretation {
	type UnificationError = ResourceUnificationFailed;

	fn try_unify(
		&mut self,
		a: &Self::Resource,
		b: &Self::Resource,
	) -> Result<(), ResourceUnificationFailed> {
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
							Err(ResourceUnificationFailed)
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
						Err(ResourceUnificationFailed)
					}
				}
				Err(_) => Err(ResourceUnificationFailed),
			},
			(a, Term::Id(Id::Blank(b))) => match self.classes.replace(b, a.clone()) {
				Ok(Term::Id(Id::Blank(_))) => Ok(()),
				Ok(c) => {
					if c == *a {
						Ok(())
					} else {
						Err(ResourceUnificationFailed)
					}
				}
				Err(_) => Err(ResourceUnificationFailed),
			},
			(a, b) => {
				if a == b {
					Ok(())
				} else {
					Err(ResourceUnificationFailed)
				}
			}
		}
	}

	fn substitute_resource(&self, r: Self::Resource) -> Self::Resource {
		self.map_term(r)
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

	type Iris<'a>
		= std::option::IntoIter<&'a IriBuf>
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

	type Literals<'a>
		= std::option::IntoIter<&'a Literal>
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
