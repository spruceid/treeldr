use treeldr::Vocabulary;

#[derive(Debug)]
pub struct NameInvalid(pub String);

impl<F> super::AnyError<F> for NameInvalid {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("invalid name `{}`", self.0)
	}
}