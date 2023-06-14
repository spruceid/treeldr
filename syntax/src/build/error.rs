use iref::IriBuf;
use locspan::{MaybeLocated, Meta, Span};
use thiserror::Error;
use treeldr::{reporting, vocab::TldrVocabulary, Id};

#[derive(Debug)]
pub enum Error<M> {
	Global(treeldr_build::Error<M>),
	Local(Meta<LocalError<M>, M>),
}

impl<M: Clone + MaybeLocated<Span = Span>> reporting::DiagnoseWithVocabulary<M> for Error<M>
where
	M::File: Clone,
{
	fn message(&self, vocab: &TldrVocabulary) -> String {
		match self {
			Self::Global(e) => e.message(vocab),
			Self::Local(e) => reporting::Diagnose::message(e),
		}
	}

	fn labels(
		&self,
		vocab: &TldrVocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		match self {
			Self::Global(e) => e.labels(vocab),
			Self::Local(e) => reporting::Diagnose::labels(e),
		}
	}
}

impl<M> From<treeldr_build::Error<M>> for Error<M> {
	fn from(e: treeldr_build::Error<M>) -> Self {
		Self::Global(e)
	}
}

impl<M> From<Meta<LocalError<M>, M>> for Error<M> {
	fn from(e: Meta<LocalError<M>, M>) -> Self {
		Self::Local(e)
	}
}

#[derive(Error, Debug)]
pub enum LocalError<M> {
	#[error("`{0}` is not a valid IRI")]
	InvalidExpandedCompactIri(String),
	#[error("prefix `{0}` is undefined")]
	UndefinedPrefix(String),
	#[error("prefix `{0}` is already defined")]
	AlreadyDefinedPrefix(String, M),
	#[error("cannot resolve the IRI reference without a base IRI")]
	NoBaseIri,
	#[error("should be `{expected}`")]
	BaseIriMismatch {
		expected: Box<IriBuf>,
		found: Box<IriBuf>,
		because: M,
	},
	#[error("type aliases are not supported")]
	TypeAlias(Id, M),
	#[error("only inline layouts can be assigned a name")]
	Renaming(Id, M),
	#[error("cannot define restricted field layout outside an intersection")]
	PropertyRestrictionOutsideIntersection,
	#[error("field not found")]
	FieldRestrictionNoMatches,
	#[error("unexpected field restriction")]
	UnexpectedFieldRestriction,
	#[error("field restrictions lead to anonymous layout")]
	AnonymousFieldLayoutIntersection(Vec<Meta<Id, M>>),
}

impl<M: Clone + MaybeLocated<Span = Span>> reporting::DiagnoseWithMetadata<M> for LocalError<M>
where
	M::File: Clone,
{
	fn message(&self, _cause: &M) -> String {
		match self {
			Self::InvalidExpandedCompactIri(_) => "invalid expanded compact IRI".to_string(),
			Self::UndefinedPrefix(_) => "undefined prefix".to_string(),
			Self::AlreadyDefinedPrefix(_, _) => "already defined prefix".to_string(),
			Self::NoBaseIri => "no base IRI".to_string(),
			Self::BaseIriMismatch { .. } => "base IRI mismatch".to_string(),
			Self::TypeAlias(_, _) => "type aliases are not supported".to_string(),
			Self::Renaming(_, _) => "invalid layout renaming".to_string(),
			Self::PropertyRestrictionOutsideIntersection => {
				"cannot define restricted field layout outside an intersection".to_string()
			}
			Self::FieldRestrictionNoMatches => "no matches for field restriction".to_string(),
			Self::UnexpectedFieldRestriction => {
				"field restrictions can only be applied on structure layouts".to_string()
			}
			Self::AnonymousFieldLayoutIntersection(_) => "unexpected anonymous layout".to_string(),
		}
	}

	fn labels(&self, cause: &M) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		let mut labels = Vec::new();

		if let Some(loc) = cause.optional_location() {
			labels.push(
				loc.clone()
					.into_primary_label()
					.with_message(self.to_string()),
			)
		}

		match self {
			Self::AlreadyDefinedPrefix(_, original_meta) => {
				if let Some(loc) = original_meta.optional_location() {
					labels.push(
						loc.clone()
							.into_secondary_label()
							.with_message("original prefix defined here".to_string()),
					)
				}
			}
			Self::BaseIriMismatch { because, .. } => {
				if let Some(loc) = because.optional_location() {
					labels.push(
						loc.clone()
							.into_secondary_label()
							.with_message("original base IRI defined here".to_string()),
					)
				}
			}
			Self::AnonymousFieldLayoutIntersection(layouts) => {
				for layout in layouts {
					if let Some(loc) = layout.metadata().optional_location() {
						labels.push(
							loc.clone()
								.into_secondary_label()
								.with_message("part of the intersection".to_string()),
						)
					}
				}
			}
			_ => (),
		}

		labels
	}
}
