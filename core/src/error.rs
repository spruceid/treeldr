use crate::{reporting::Diagnose, Caused, Vocabulary};

pub type Error<F> = Caused<Description<F>, F>;

pub trait AnyError<F> {
	fn message(&self, vocab: &Vocabulary) -> String;

	fn primary_label(&self, _vocab: &Vocabulary) -> Option<String> {
		Some("here".to_string())
	}

	fn other_labels(&self, _vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		Vec::new()
	}

	fn notes(&self, _vocab: &Vocabulary) -> Vec<String> {
		Vec::new()
	}
}

macro_rules! errors {
	{ $( $mod_id:ident :: $id:ident $(<$arg:ident>)? ),* } => {
		$(
			pub mod $mod_id;
			pub use $mod_id::$id;
		)*

		#[derive(Debug)]
		pub enum Description<F> {
			$(
				$id( $id $(<$arg>)? )
			),*
		}

		$(
			impl<F> From<$id $(<$arg>)?> for Description<F> {
				fn from(e: $id $(<$arg>)?) -> Self {
					Self::$id(e)
				}
			}
		)*

		impl<'c, 'a, F: Clone> Diagnose<F> for WithVocabulary<'c, 'a, F> {
			fn message(&self) -> String {
				match self.error().inner() {
					$(
						Description::$id(e) => AnyError::<F>::message(e, self.vocabulary())
					),*
				}
			}

			fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
				match self.error().inner() {
					$(
						Description::$id(e) => {
							let mut labels = e.other_labels(self.vocabulary());

							if let Some(cause) = self.error().cause() {
								let label = cause.clone().into_primary_label();
								let label = match AnyError::<F>::primary_label(e, self.vocabulary()) {
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

			fn notes(&self) -> Vec<String> {
				match self.error().inner() {
					$(
						Description::$id(e) => AnyError::<F>::notes(e, self.vocabulary())
					),*
				}
			}
		}
	};
}

errors! {
	unimplemented_feature::UnimplementedFeature,
	node_unknown::NodeUnknown,
	node_invalid_type::NodeInvalidType<F>,
	type_mismatch_kind::TypeMismatchKind<F>,
	type_mismatch_union::TypeMismatchUnion<F>,
	type_union_literal_option::TypeUnionLiteralOption,
	property_mismatch_functional::PropertyMismatchFunctional<F>,
	property_mismatch_required::PropertyMismatchRequired<F>,
	property_mismatch_type::PropertyMismatchType<F>,
	property_missing_type::PropertyMissingType,
	layout_mismatch_description::LayoutMismatchDescription<F>,
	layout_mismatch_name::LayoutMismatchName<F>,
	layout_mismatch_type::LayoutMismatchType<F>,
	layout_missing_name::LayoutMissingName,
	layout_missing_description::LayoutMissingDescription,
	layout_missing_type::LayoutMissingType,
	layout_literal_field::LayoutLiteralField,
	layout_literal_intersection::LayoutLiteralIntersection,
	layout_intersection_failed::LayoutIntersectionFailed,
	layout_field_mismatch_functional::LayoutFieldMismatchFunctional<F>,
	layout_field_mismatch_layout::LayoutFieldMismatchLayout<F>,
	layout_field_mismatch_name::LayoutFieldMismatchName<F>,
	layout_field_mismatch_property::LayoutFieldMismatchProperty<F>,
	layout_field_mismatch_required::LayoutFieldMismatchRequired<F>,
	layout_field_missing_layout::LayoutFieldMissingLayout,
	layout_field_missing_property::LayoutFieldMissingProperty,
	layout_field_missing_name::LayoutFieldMissingName,
	layout_variant_missing_name::LayoutVariantMissingName,
	list_mismatch_item::ListMismatchItem<F>,
	list_mismatch_rest::ListMismatchRest<F>,
	list_missing_item::ListMissingItem,
	list_missing_rest::ListMissingRest,
	regexp_invalid::RegExpInvalid,
	name_invalid::NameInvalid
}

impl<F> Caused<Description<F>, F> {
	pub fn with_vocabulary<'c>(&self, vocab: &'c Vocabulary) -> WithVocabulary<'c, '_, F> {
		WithVocabulary(vocab, self)
	}
}

/// Caused error with contextual information.
pub struct WithVocabulary<'c, 'a, F>(&'c Vocabulary, &'a Caused<Description<F>, F>);

impl<'c, 'a, F> WithVocabulary<'c, 'a, F> {
	fn vocabulary(&self) -> &'c Vocabulary {
		self.0
	}

	fn error(&self) -> &'a Caused<Description<F>, F> {
		self.1
	}
}
