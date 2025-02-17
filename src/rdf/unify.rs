use std::borrow::Cow;

use disjoint_collections::DisjointVec;
use hashbrown::HashMap;
use iref::Iri;
use rdf_types::{BlankId, BlankIdBuf, CowLiteral, Interpretation, LocalTerm, Term};

#[derive(Debug, thiserror::Error)]
#[error("resource unification failed")]
pub struct ResourceUnificationFailed;

pub trait FallibleResourceUnifier<R> {
	type Error;

	fn try_unify(&mut self, a: &R, b: &R) -> Result<R, Self::Error>;

	fn insert_iri<'a>(&mut self, r: &R, iri: impl Into<Cow<'a, Iri>>) -> Result<(), Self::Error>;

	fn insert_literal<'a>(
		&mut self,
		r: &R,
		literal: impl Into<CowLiteral<'a>>,
	) -> Result<(), Self::Error>;

	fn substitute_resource(&self, r: R) -> R;
}

pub trait TryUnifyInterpretation: Interpretation {
	type Error;
	type Unifier<'a>: FallibleResourceUnifier<Self::Resource, Error = Self::Error>
	where
		Self: 'a;

	fn begin_unification(&mut self) -> Self::Unifier<'_>;

	fn try_unify(
		&mut self,
		a: &Self::Resource,
		b: &Self::Resource,
	) -> Result<Self::Resource, Self::Error> {
		let mut unifier = self.begin_unification();
		unifier.try_unify(a, b)
	}
}

impl<I> TryUnifyInterpretation for I
where
	I: Interpretation<Resource = LocalTerm>,
{
	type Unifier<'a>
		= LocalTermUnifier
	where
		Self: 'a;
	type Error = ResourceUnificationFailed;

	fn begin_unification(&mut self) -> Self::Unifier<'_> {
		LocalTermUnifier::new()
	}
}

#[derive(Debug, Default)]
pub struct LocalTermUnifier {
	map: HashMap<BlankIdBuf, usize>,
	classes: DisjointVec<LocalTerm>,
}

impl LocalTermUnifier {
	pub fn new() -> Self {
		Self::default()
	}

	fn import_blank_id(&mut self, id: &BlankId) -> usize {
		match self.map.get(id) {
			Some(i) => *i,
			None => {
				let i = self.classes.push(LocalTerm::Anonymous(id.to_owned()));
				self.map.insert(id.to_owned(), i);
				i
			}
		}
	}

	fn insert_term(&mut self, a: &LocalTerm, b: Term) -> Result<(), ResourceUnificationFailed> {
		match a {
			LocalTerm::Anonymous(a) => {
				let i = self.import_blank_id(a);
				let term = self.classes.get_mut(i).unwrap();
				unify_local_term(term, &b)
			}
			LocalTerm::Named(a) => {
				if *a == b {
					Ok(())
				} else {
					Err(ResourceUnificationFailed)
				}
			}
		}
	}
}

impl FallibleResourceUnifier<LocalTerm> for LocalTermUnifier {
	type Error = ResourceUnificationFailed;

	fn try_unify(&mut self, a: &LocalTerm, b: &LocalTerm) -> Result<LocalTerm, Self::Error> {
		eprintln!("unify {a} and {b}");
		match (a, b) {
			(LocalTerm::Anonymous(a), LocalTerm::Anonymous(b)) => {
				if a == b {
					Ok(LocalTerm::Anonymous(a.to_owned()))
				} else {
					let ia = self.import_blank_id(a);
					let ib = self.import_blank_id(b);
					let i = self
						.classes
						.merge(ia, ib, std::cmp::min)
						.unwrap();
					Ok(self.classes.get(i).unwrap().clone())
				}
			}
			(LocalTerm::Anonymous(a), LocalTerm::Named(b)) => {
				let i = self.import_blank_id(a);
				let term = self.classes.get_mut(i).unwrap();
				unify_local_term(term, b)?;
				Ok(LocalTerm::Named(b.clone()))
			}
			(LocalTerm::Named(a), LocalTerm::Anonymous(b)) => {
				let i = self.import_blank_id(b);
				let term = self.classes.get_mut(i).unwrap();
				unify_local_term(term, a)?;
				Ok(LocalTerm::Named(a.clone()))
			}
			(LocalTerm::Named(a), LocalTerm::Named(b)) => {
				if a == b {
					Ok(LocalTerm::Named(a.to_owned()))
				} else {
					Err(ResourceUnificationFailed)
				}
			}
		}
	}

	fn insert_iri<'a>(
		&mut self,
		r: &LocalTerm,
		iri: impl Into<Cow<'a, Iri>>,
	) -> Result<(), Self::Error> {
		self.insert_term(r, Term::Iri(iri.into().into_owned()))
	}

	fn insert_literal<'a>(
		&mut self,
		r: &LocalTerm,
		literal: impl Into<CowLiteral<'a>>,
	) -> Result<(), Self::Error> {
		self.insert_term(r, Term::Literal(literal.into().into_owned()))
	}

	fn substitute_resource(&self, r: LocalTerm) -> LocalTerm {
		match r {
			LocalTerm::Anonymous(b) => match self.map.get(&b) {
				Some(&i) => self.classes.get(i).unwrap().clone(),
				None => LocalTerm::Anonymous(b),
			},
			term => term,
		}
	}
}

fn unify_local_term(a: &mut LocalTerm, b: &Term) -> Result<(), ResourceUnificationFailed> {
	if let LocalTerm::Named(a) = a {
		return if a == b {
			Ok(())
		} else {
			Err(ResourceUnificationFailed)
		};
	}

	*a = LocalTerm::Named(b.clone());
	Ok(())
}
