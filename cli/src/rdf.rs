use iref::Iri;
use rdf_types::{BlankId, Term};

#[derive(Debug, thiserror::Error)]
#[error("invalid RDF term `{0}`")]
pub struct InvalidTerm(String);

pub fn parse_term(input: &str) -> Result<Term, InvalidTerm> {
	match BlankId::new(input) {
		Ok(blank_id) => Ok(Term::blank(blank_id.to_owned())),
		Err(_) => match Iri::new(input) {
			Ok(iri) => Ok(Term::iri(iri.to_owned())),
			Err(_) => Err(InvalidTerm(input.to_owned())),
		},
	}
}
