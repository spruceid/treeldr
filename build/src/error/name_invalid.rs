use treeldr::Vocabulary;

#[derive(Debug)]
pub struct NameInvalid(pub rdf_types::StringLiteral);

impl<F> super::AnyError<F> for NameInvalid {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("invalid name {}", self.0)
	}
}