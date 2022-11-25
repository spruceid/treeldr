use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct TypeIntersectionLiteralType(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for TypeIntersectionLiteralType {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid literal value in type intersection `{}`", self.0.display(vocab))
	}
}