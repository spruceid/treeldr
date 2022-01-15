use crate::{
	Feature,
	Context,
	Ref,
	Id,
	source,
	Cause,
	Caused,
	node,
	ty
};

/// Error with diagnostic reporting support.
pub trait Diagnose {
	fn message(&self) -> String;

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<source::Id>> {
		Vec::new()
	}

	fn notes(&self) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<source::Id> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message())
			.with_labels(self.labels())
			.with_notes(self.notes())
	}
}

/// Error.
#[derive(Debug)]
pub enum Error {
	Unimplemented(Feature),
	InvalidNodeType {
		id: Id,
		expected: node::Type,
		found: node::Type,
		because: Option<Cause>
	},
	UnknownNode {
		id: Id,
		expected_ty: Option<node::Type>
	},
	TypeMismatch {
		expected: ty::Expr,
		found: ty::Expr,
		because: Option<Cause>
	},
	LayoutTypeMismatch {
		expected: Ref<ty::Definition>,
		found: Ref<ty::Definition>,
		because: Option<Cause>
	}
}

impl Caused<Error> {
	pub fn with_context<'c>(&self, context: &'c Context) -> WithContext<'c, '_> {
		WithContext(context, self)
	}
}

/// Caused error with contextual information.
pub struct WithContext<'c, 'a>(&'c Context, &'a Caused<Error>);

impl<'c, 'a> WithContext<'c, 'a> {
	fn context(&self) -> &'c Context {
		self.0
	}

	fn error(&self) -> &'a Caused<Error> {
		self.1
	}
}

impl node::Type {
	/// English name with determiner (article).
	fn en_determiner_name(&self) -> &'static str {
		match self {
			node::Type::Type => "a type",
			node::Type::Property => "a property",
			node::Type::Layout => "a layout"
		}
	}

	fn en_name(&self) -> &'static str {
		match self {
			node::Type::Type => "type",
			node::Type::Property => "property",
			node::Type::Layout => "layout"
		}
	}
}

// impl<'c, 'a> fmt::Display for WithContext<'c, 'a> {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		match self.error().inner() {
// 			Error::Unimplemented(feature) => write!(f, "unimplemented feature `{}`", feature),
// 			Error::InvalidNodeType { id, expected, found, .. } => {
// 				let iri = self.context().vocabulary().get(*id).unwrap();
// 				write!(f, "node <{}> is already {} and not {}", iri, found.en_determiner_name(), expected.en_determiner_name())
// 			},
// 			Error::UnknownNode { id, expected_ty } => {
// 				let iri = self.context().vocabulary().get(*id).unwrap();
// 				match expected_ty {
// 					Some(ty) => write!(f, "undefined {} <{}>", ty.en_name(), iri),
// 					None => write!(f, "undefined node <{}>", iri),
// 				}
// 			}
// 		}
// 	}
// }

impl<'c, 'a> Diagnose for WithContext<'c, 'a> {
	fn message(&self) -> String {
		match self.error().inner() {
			Error::Unimplemented(feature) => format!("unimplemented feature `{}`.", feature),
			Error::InvalidNodeType { id, .. } => {
				let iri = self.context().vocabulary().get(*id).unwrap();
				format!("invalid node type for <{}>.", iri)
			},
			Error::UnknownNode { id, expected_ty } => {
				let iri = self.context().vocabulary().get(*id).unwrap();
				match expected_ty {
					Some(ty) => format!("undefined {} <{}>", ty.en_name(), iri),
					None => format!("undefined node <{}>", iri),
				}
			}
			Error::TypeMismatch { .. } => {
				format!("type mismatch")
			}
			Error::LayoutTypeMismatch { .. } => {
				format!("layout type mismatch")
			}
		}
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<source::Id>> {
		let mut labels = Vec::new();
		use codespan_reporting::diagnostic::Label;
		match self.error().inner() {
			Error::Unimplemented(_) => {
				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(Label::primary(source.file(), source.span()).with_message("feature required here"))
				}
			},
			Error::InvalidNodeType { expected, found, because, .. } => {
				if let Some(cause) = self.error().cause() {
					let message = match cause {
						Cause::Explicit(_) => format!("declared as {} here", expected.en_determiner_name()),
						Cause::Implicit(_) => format!("implicitly declared as {} here", expected.en_determiner_name())
					};

					let source = cause.source();
					labels.push(Label::primary(source.file(), source.span()).with_message(message))
				}

				if let Some(cause) = because {
					let message = match cause {
						Cause::Explicit(_) => format!("already declared as {} here", found.en_determiner_name()),
						Cause::Implicit(_) => format!("already implicitly declared as {} here", found.en_determiner_name())
					};

					let source = cause.source();
					labels.push(Label::secondary(source.file(), source.span()).with_message(message))
				}
			}
			Error::UnknownNode { .. } => {
				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(Label::secondary(source.file(), source.span()).with_message("used here"))
				}
			}
			Error::TypeMismatch { because, .. } | Error::LayoutTypeMismatch { because, .. } => {
				if let Some(cause) = self.error().cause() {
					let message = match cause {
						Cause::Explicit(_) => format!("found type is declared here"),
						Cause::Implicit(_) => format!("found type is implicitly declared here")
					};

					let source = cause.source();
					labels.push(Label::primary(source.file(), source.span()).with_message(message))
				}

				if let Some(cause) = because {
					let message = match cause {
						Cause::Explicit(_) => format!("expected type is declared here"),
						Cause::Implicit(_) => format!("expected type is implicitly declared here")
					};

					let source = cause.source();
					labels.push(Label::secondary(source.file(), source.span()).with_message(message))
				}
			}
		}

		labels
	}

	fn notes(&self) -> Vec<String> {
		let mut notes = Vec::new();

		match self.error().inner() {
			Error::InvalidNodeType { id, expected, found, because, .. } => {
				let iri = self.context().vocabulary().get(*id).unwrap();
				if self.error().cause().is_none() {
					notes.push(format!("<{}> should be {}", iri, expected.en_determiner_name()))
				}

				if because.is_none() {
					notes.push(format!("...but <{}> is {}", iri, found.en_determiner_name()))
				}
			}
			Error::TypeMismatch { expected, found, .. } => {
				notes.push(format!("expected type `{}`", expected.with_context(self.context())));
				notes.push(format!("   found type `{}`", found.with_context(self.context())))
			},
			Error::LayoutTypeMismatch { expected, found, .. } => {
				let expected_id = self.context().vocabulary().get(self.context().types().get(*expected).unwrap().id()).unwrap();
				let found_id = self.context().vocabulary().get(self.context().types().get(*found).unwrap().id()).unwrap();
				notes.push(format!("expected type `{}`", expected_id));
				notes.push(format!("   found type `{}`", found_id))
			},
			_ => ()
		}

		notes
	}
}