use locspan::{MaybeLocated, Meta, Span};
use treeldr::{reporting::DiagnoseWithMetadataAndVocabulary, vocab::TldrVocabulary};

pub type Error<M> = Meta<Description<M>, M>;

pub trait AnyError<M: MaybeLocated<Span = Span>> {
	fn message(&self, vocab: &TldrVocabulary) -> String;

	fn primary_label(&self, _vocab: &TldrVocabulary) -> Option<String> {
		Some("here".to_string())
	}

	fn other_labels(
		&self,
		_vocab: &TldrVocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(&self, _vocab: &TldrVocabulary) -> Vec<String> {
		Vec::new()
	}
}

macro_rules! errors {
	{ $( $mod_id:ident :: $id:ident $(<$($arg:ident),+>)? ),* } => {
		$(
			pub mod $mod_id;
			pub use $mod_id::$id;
		)*

		#[derive(Debug)]
		pub enum Description<M> {
			$(
				$id( $id $(<$($arg),+>)? )
			),*
		}

		$(
			impl<M> From<$id $(<$($arg),+>)?> for Description<M> {
				fn from(e: $id $(<$($arg),+>)?) -> Self {
					Self::$id(e)
				}
			}
		)*

		impl<M: MaybeLocated<Span=Span>> DiagnoseWithMetadataAndVocabulary<M> for Description<M> where M::File: Clone {
			fn message(&self, _cause: &M, vocabulary: &TldrVocabulary) -> String {
				match self {
					$(
						Self::$id(e) => AnyError::<M>::message(e, vocabulary)
					),*
				}
			}

			fn labels(&self, metadata: &M, vocabulary: &TldrVocabulary) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
				match self {
					$(
						Self::$id(e) => {
							let mut labels = AnyError::<M>::other_labels(e, vocabulary);

							if let Some(loc) = metadata.optional_location().cloned() {
								let label = loc.into_primary_label();
								let label = match AnyError::<M>::primary_label(e, vocabulary) {
									Some(msg) => label.with_message(msg),
									None => label
								};

								labels.push(label)
							}

							labels
						}
					),*
				}
			}

			fn notes(&self, _cause: &M, vocabulary: &TldrVocabulary) -> Vec<String> {
				match self {
					$(
						Self::$id(e) => AnyError::<M>::notes(e, vocabulary)
					),*
				}
			}
		}
	};
}

errors! {
	node_type_invalid::NodeTypeInvalid<M>,
	node_binding_functional_conflict::NodeBindingFunctionalConflict<M>,
	node_binding_type_invalid::NodeBindingTypeInvalid<M>,
	node_binding_missing::NodeBindingMissing,
	node_unknown::NodeUnknown,
	layout_description_missing::LayoutDescriptionMissing,
	layout_description_mismatch::LayoutDescriptionMismatch<M>,
	layout_not_primitive::LayoutNotPrimitive,
	layout_datatype_restriction_invalid::LayoutDatatypeRestrictionInvalid,
	layout_datatype_restriction_conflict::LayoutDatatypeRestrictionConflict<M>,
	layout_container_restriction_conflict::LayoutContainerRestrictionConflict<M>,
	regexp_invalid::RegExpInvalid,
	name_invalid::NameInvalid,
	literal_unexpected::LiteralUnexpected,
	literal_expected::LiteralExpected,
	literal_invalid::LiteralInvalid,
	literal_type_mismatch::LiteralTypeMismatch,
	subclass_cycle::SubClassCycle<M>,
	subproperty_cycle::SubPropertyCycle<M>
}
