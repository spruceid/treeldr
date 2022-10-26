use locspan::{MaybeLocated, Meta, Span};
use rdf_types::Vocabulary;
use treeldr::{reporting::DiagnoseWithMetadataAndVocabulary, BlankIdIndex, IriIndex};

pub type Error<M> = Meta<Description<M>, M>;

pub trait AnyError<M: MaybeLocated<Span = Span>> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String;

	fn primary_label(
		&self,
		_vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Option<String> {
		Some("here".to_string())
	}

	fn other_labels(
		&self,
		_vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(
		&self,
		_vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<String> {
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
			fn message(&self, _cause: &M, vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
				match self {
					$(
						Self::$id(e) => AnyError::<M>::message(e, vocabulary)
					),*
				}
			}

			fn labels(&self, metadata: &M, vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
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

			fn notes(&self, _cause: &M, vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> Vec<String> {
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
	unimplemented_feature::UnimplementedFeature,
	node_unknown::NodeUnknown,
	node_invalid_type::NodeInvalidType<M>,
	type_mismatch_kind::TypeMismatchKind<M>,
	type_mismatch_union::TypeMismatchUnion<M>,
	type_mismatch_intersection::TypeMismatchIntersection<M>,
	property_mismatch_functional::PropertyMismatchFunctional<M>,
	property_mismatch_required::PropertyMismatchRequired<M>,
	property_mismatch_type::PropertyMismatchType<M>,
	property_missing_type::PropertyMissingType,
	layout_mismatch_primitive::LayoutMismatchPrimitive<M>,
	layout_mismatch_description::LayoutMismatchDescription<M>,
	layout_mismatch_name::LayoutMismatchName<M>,
	layout_mismatch_type::LayoutMismatchType<M>,
	layout_missing_name::LayoutMissingName,
	layout_missing_description::LayoutMissingDescription,
	layout_missing_datatype_primitive::LayoutMissingDatatypePrimitive,
	layout_intersection_failed::LayoutIntersectionFailed,
	layout_field_mismatch_functional::LayoutFieldMismatchFunctional<M>,
	layout_field_mismatch_layout::LayoutFieldMismatchLayout<M>,
	layout_field_mismatch_name::LayoutFieldMismatchName<M>,
	layout_field_mismatch_property::LayoutFieldMismatchProperty<M>,
	layout_field_mismatch_required::LayoutFieldMismatchRequired<M>,
	layout_field_missing_layout::LayoutFieldMissingLayout,
	layout_field_missing_name::LayoutFieldMissingName,
	layout_infinite_size::LayoutInfiniteSize,
	layout_variant_missing_name::LayoutVariantMissingName,
	layout_datatype_restriction_invalid::LayoutDatatypeRestrictionInvalid,
	layout_datatype_restriction_integer_conflict::LayoutDatatypeRestrictionIntegerConflict<M>,
	layout_datatype_restriction_unsigned_conflict::LayoutDatatypeRestrictionUnsignedConflict<M>,
	layout_datatype_restriction_float_conflict::LayoutDatatypeRestrictionFloatConflict<M>,
	layout_datatype_restriction_double_conflict::LayoutDatatypeRestrictionDoubleConflict<M>,
	layout_container_restriction_conflict::LayoutContainerRestrictionConflict<M>,
	list_mismatch_item::ListMismatchItem<M>,
	list_mismatch_rest::ListMismatchRest<M>,
	list_missing_item::ListMissingItem,
	list_missing_rest::ListMissingRest,
	regexp_invalid::RegExpInvalid,
	name_invalid::NameInvalid,
	literal_unexpected::LiteralUnexpected<M>
}
