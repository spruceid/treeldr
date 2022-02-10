use crate::{layout, node, prop, source, ty, Cause, Caused, Feature, Id, Model, Ref};

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
	PrefixRedefinition(String, Option<Cause>),
	UndefinedPrefix(String),
	InvalidExpandedCompactIri(String),
	InvalidNodeType {
		id: Id,
		expected: node::Type,
		found: node::CausedTypes,
	},
	UnknownNode {
		id: Id,
		expected_ty: Option<node::Type>,
	},
	TypeMismatch {
		expected: Ref<ty::Definition>,
		found: Ref<ty::Definition>,
		because: Option<Cause>,
	},
	ImplicitLayoutMismatch {
		expected: Ref<layout::Definition>,
		found: Ref<layout::Definition>,
		because: Option<Cause>,
	},
	LayoutTypeMismatch {
		expected: Ref<ty::Definition>,
		found: Ref<ty::Definition>,
		because: Option<Cause>,
	},
	LayoutMismatch(layout::Mismatch),
	/// A field is not required but its corresponding property is.
	FieldNotRequired {
		prop: Ref<prop::Definition>,
		because: Option<Cause>,
	},
	FieldNotFunctional {
		prop: Ref<prop::Definition>,
		because: Option<Cause>,
	},
	MissingPropertyField {
		prop: Ref<prop::Definition>,
		because: Option<Cause>,
	},
}

impl Caused<Error> {
	pub fn with_model<'c>(&self, context: &'c Model) -> WithModel<'c, '_> {
		WithModel(context, self)
	}
}

impl From<Caused<layout::Mismatch>> for Caused<Error> {
	fn from(e: Caused<layout::Mismatch>) -> Self {
		e.map(Error::LayoutMismatch)
	}
}

/// Caused error with contextual information.
pub struct WithModel<'c, 'a>(&'c Model, &'a Caused<Error>);

impl<'c, 'a> WithModel<'c, 'a> {
	fn context(&self) -> &'c Model {
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
			node::Type::Layout => "a layout",
		}
	}

	fn en_name(&self) -> &'static str {
		match self {
			node::Type::Type => "type",
			node::Type::Property => "property",
			node::Type::Layout => "layout",
		}
	}
}

impl layout::Type {
	fn en_name(&self) -> &'static str {
		match self {
			Self::Struct => "structure",
			Self::Native(n) => match n {
				layout::Native::Boolean => "boolean",
				layout::Native::Integer => "integer",
				layout::Native::PositiveInteger => "positive integer",
				layout::Native::Float => "float",
				layout::Native::Double => "double",
				layout::Native::String => "string",
				layout::Native::Time => "time",
				layout::Native::Date => "date",
				layout::Native::DateTime => "data and time",
				layout::Native::Iri => "IRI",
				layout::Native::Uri => "URI",
				layout::Native::Url => "URL",
				layout::Native::Reference(_) => "reference"
			},
		}
	}
}

// impl<'c, 'a> fmt::Display for WithModel<'c, 'a> {
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

impl<'c, 'a> Diagnose for WithModel<'c, 'a> {
	fn message(&self) -> String {
		match self.error().inner() {
			Error::Unimplemented(feature) => format!("unimplemented feature `{}`", feature),
			Error::PrefixRedefinition(prefix, _) => format!("prefix `{}` is already used", prefix),
			Error::UndefinedPrefix(prefix) => format!("prefix `{}` is undefined", prefix),
			Error::InvalidExpandedCompactIri(s) => format!("invalid expanded compact IRI `{}`", s),
			Error::InvalidNodeType { id, .. } => {
				let iri = self.context().vocabulary().get(*id).unwrap();
				format!("invalid node type for <{}>.", iri)
			}
			Error::UnknownNode { id, expected_ty } => {
				let iri = self.context().vocabulary().get(*id).unwrap();
				match expected_ty {
					Some(ty) => format!("undefined {} <{}>", ty.en_name(), iri),
					None => format!("undefined node <{}>", iri),
				}
			}
			Error::TypeMismatch { .. } => "type mismatch".to_string(),
			Error::ImplicitLayoutMismatch { .. } => "implicit layout mismatch".to_string(),
			Error::LayoutTypeMismatch { .. } => "layout for-type mismatch".to_string(),
			Error::LayoutMismatch(e) => match e {
				layout::Mismatch::Type { .. } => "layout type mismatch".to_string(),
				layout::Mismatch::FieldProperty { .. } => "field property mismatch".to_string(),
				layout::Mismatch::FieldName { .. } => "field name mismatch".to_string(),
				layout::Mismatch::FieldLayout { .. } => "field layout mismatch".to_string(),
				layout::Mismatch::AttributeRequired { .. } => {
					"field `required` attribute mismatch".to_string()
				}
				layout::Mismatch::AttributeFunctional { .. } => {
					"field `unique` attribute mismatch".to_string()
				}
				layout::Mismatch::MissingField { name, .. } => format!("missing field `{}`", name),
				layout::Mismatch::AdditionalField { name, .. } => {
					format!("unexpected field `{}`", name)
				}
			},
			Error::FieldNotRequired { .. } => {
				"required property has a non required field".to_string()
			}
			Error::FieldNotFunctional { .. } => {
				"functional (unique) property has a non functional field".to_string()
			}
			Error::MissingPropertyField { .. } => "missing field for required property".to_string(),
		}
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<source::Id>> {
		let mut labels = Vec::new();
		match self.error().inner() {
			Error::Unimplemented(_) => {
				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(
						source
							.into_primary_label()
							.with_message("feature required here"),
					)
				}
			}
			Error::PrefixRedefinition(_, because) => {
				if let Some(cause) = because {
					let source = cause.source();
					labels.push(
						source
							.into_secondary_label()
							.with_message("first defined here"),
					)
				}

				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(source.into_primary_label().with_message("redefined here"))
				}
			}
			Error::UndefinedPrefix(_) => {
				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(source.into_primary_label().with_message("undefined prefix"))
				}
			}
			Error::InvalidExpandedCompactIri(_) => {
				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(source.into_primary_label().with_message("used here"))
				}
			}
			Error::InvalidNodeType {
				expected, found, ..
			} => {
				if let Some(cause) = self.error().cause() {
					let message = match cause {
						Cause::Explicit(_) => {
							format!("used as {} here", expected.en_determiner_name())
						}
						Cause::Implicit(_) => {
							format!("implicitly used as {} here", expected.en_determiner_name())
						}
					};

					let source = cause.source();
					labels.push(source.into_primary_label().with_message(message))
				}

				for ty in found {
					if let Some(cause) = ty.cause() {
						let message = match cause {
							Cause::Explicit(_) => {
								format!("already declared as {} here", ty.en_determiner_name())
							}
							Cause::Implicit(_) => format!(
								"already implicitly declared as {} here",
								ty.en_determiner_name()
							),
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
			}
			Error::UnknownNode { .. } => {
				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(source.into_secondary_label().with_message("used here"))
				}
			}
			Error::TypeMismatch { because, .. } | Error::LayoutTypeMismatch { because, .. } => {
				if let Some(cause) = self.error().cause() {
					let message = match cause {
						Cause::Explicit(_) => "found type is declared here".to_string(),
						Cause::Implicit(_) => "found type is implicitly declared here".to_string(),
					};

					let source = cause.source();
					labels.push(source.into_primary_label().with_message(message))
				}

				if let Some(cause) = because {
					let message = match cause {
						Cause::Explicit(_) => "expected type is declared here".to_string(),
						Cause::Implicit(_) => {
							"expected type is implicitly declared here".to_string()
						}
					};

					let source = cause.source();
					labels.push(source.into_secondary_label().with_message(message))
				}
			}
			Error::ImplicitLayoutMismatch { because, .. } => {
				if let Some(cause) = self.error().cause() {
					let message = match cause {
						Cause::Explicit(_) => "found layout is declared here".to_string(),
						Cause::Implicit(_) => "found layout is implicitly declared here".to_string(),
					};

					let source = cause.source();
					labels.push(source.into_primary_label().with_message(message))
				}

				if let Some(cause) = because {
					let message = match cause {
						Cause::Explicit(_) => "expected layout is declared here".to_string(),
						Cause::Implicit(_) => {
							"expected layout is implicitly declared here".to_string()
						}
					};

					let source = cause.source();
					labels.push(source.into_secondary_label().with_message(message))
				}
			}
			Error::LayoutMismatch(e) => match e {
				layout::Mismatch::Type { because, .. } => {
					if let Some(cause) = because {
						let message = match cause {
							Cause::Explicit(_) => "expected layout type declared here".to_string(),
							Cause::Implicit(_) => {
								"expected layout type is implicitly declared here".to_string()
							}
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
				layout::Mismatch::FieldProperty { because, .. } => {
					if let Some(cause) = because {
						let message = match cause {
							Cause::Explicit(_) => "expected property is declared here".to_string(),
							Cause::Implicit(_) => {
								"expected property is implicitly declared here".to_string()
							}
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
				layout::Mismatch::FieldName { because, .. } => {
					if let Some(cause) = because {
						let message = match cause {
							Cause::Explicit(_) => "expected name is declared here".to_string(),
							Cause::Implicit(_) => {
								"expected name is implicitly declared here".to_string()
							}
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
				layout::Mismatch::FieldLayout { because, .. } => {
					if let Some(cause) = because {
						let message = match cause {
							Cause::Explicit(_) => "expected layout is declared here".to_string(),
							Cause::Implicit(_) => {
								"expected layout is implicitly declared here".to_string()
							}
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
				layout::Mismatch::AttributeRequired { required, because } => {
					if let Some(cause) = because {
						let message = if *required {
							match cause {
								Cause::Explicit(_) => "field is required here".to_string(),
								Cause::Implicit(_) => {
									"field is implicitly required here".to_string()
								}
							}
						} else {
							match cause {
								Cause::Explicit(_) => "field is not required here".to_string(),
								Cause::Implicit(_) => {
									"field is implicitly not required here".to_string()
								}
							}
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
				layout::Mismatch::AttributeFunctional {
					functional,
					because,
				} => {
					if let Some(cause) = because {
						let message = if *functional {
							match cause {
								Cause::Explicit(_) => "field is unique here".to_string(),
								Cause::Implicit(_) => "field is implicitly unique here".to_string(),
							}
						} else {
							match cause {
								Cause::Explicit(_) => "field is not unique here".to_string(),
								Cause::Implicit(_) => {
									"field is implicitly not unique here".to_string()
								}
							}
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
				layout::Mismatch::MissingField { because, .. } => {
					if let Some(cause) = because {
						let message = match cause {
							Cause::Explicit(_) => "missing field is declared here".to_string(),
							Cause::Implicit(_) => {
								"missing field is implicitly declared here".to_string()
							}
						};

						let source = cause.source();
						labels.push(source.into_secondary_label().with_message(message))
					}
				}
				layout::Mismatch::AdditionalField { because, .. } => {
					if let Some(cause) = because {
						let source = cause.source();
						labels.push(
							source
								.into_secondary_label()
								.with_message("this field is not declared here".to_string()),
						)
					}
				}
			},
			Error::FieldNotRequired { because, .. } => {
				if let Some(cause) = because {
					let message = match cause {
						Cause::Explicit(_) => "property is required here...".to_string(),
						Cause::Implicit(_) => "property is implicitly required here...".to_string(),
					};

					let source = cause.source();
					labels.push(source.into_secondary_label().with_message(message))
				}

				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(
						source
							.into_primary_label()
							.with_message("...but is not required here"),
					)
				}
			}
			Error::FieldNotFunctional { because, .. } => {
				if let Some(cause) = because {
					let message = match cause {
						Cause::Explicit(_) => "property is unique here...".to_string(),
						Cause::Implicit(_) => "property is implicitly unique here...".to_string(),
					};

					let source = cause.source();
					labels.push(source.into_secondary_label().with_message(message))
				}

				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(
						source
							.into_primary_label()
							.with_message("...but is not unique here"),
					)
				}
			}
			Error::MissingPropertyField { because, .. } => {
				if let Some(cause) = because {
					let message = match cause {
						Cause::Explicit(_) => "property is required here...".to_string(),
						Cause::Implicit(_) => "property is implicitly required here...".to_string(),
					};

					let source = cause.source();
					labels.push(source.into_secondary_label().with_message(message))
				}

				if let Some(cause) = self.error().cause() {
					let source = cause.source();
					labels.push(
						source
							.into_primary_label()
							.with_message("...but no field captures this property here"),
					)
				}
			}
		}

		labels
	}

	fn notes(&self) -> Vec<String> {
		let mut notes = Vec::new();

		match self.error().inner() {
			Error::InvalidNodeType {
				id,
				expected,
				found,
				..
			} => {
				let iri = self.context().vocabulary().get(*id).unwrap();
				if self.error().cause().is_none() {
					notes.push(format!(
						"<{}> should be {}",
						iri,
						expected.en_determiner_name()
					))
				}

				if found.is_empty() {
					notes.push("...but is unknown.".to_string())
				} else if found.iter().any(|ty| ty.cause().is_none()) {
					for (i, ty) in found.iter().enumerate() {
						if i == 0 {
							notes.push(format!("...but is {}", ty.en_determiner_name()))
						} else {
							notes.push(format!("...and {}", ty.en_determiner_name()))
						}
					}
				}
			}
			Error::TypeMismatch {
				expected, found, ..
			} => {
				notes.push(format!(
					"expected type `{}`",
					expected.with_model(self.context())
				));
				notes.push(format!(
					"   found type `{}`",
					found.with_model(self.context())
				))
			}
			Error::LayoutTypeMismatch {
				expected, found, ..
			} => {
				let expected_id = self
					.context()
					.vocabulary()
					.get(self.context().types().get(*expected).unwrap().id())
					.unwrap();
				let found_id = self
					.context()
					.vocabulary()
					.get(self.context().types().get(*found).unwrap().id())
					.unwrap();
				notes.push(format!("expected type `{}`", expected_id));
				notes.push(format!("   found type `{}`", found_id))
			}
			Error::LayoutMismatch(e) => match e {
				layout::Mismatch::Type {
					expected, found, ..
				} => {
					notes.push(format!("expected {}", expected.en_name()));
					notes.push(format!("   found {}", found.en_name()))
				}
				layout::Mismatch::FieldProperty {
					expected, found, ..
				} => {
					let expected_id = self
						.context()
						.vocabulary()
						.get(self.context().properties().get(*expected).unwrap().id())
						.unwrap();
					let found_id = self
						.context()
						.vocabulary()
						.get(self.context().properties().get(*found).unwrap().id())
						.unwrap();
					notes.push(format!("expected property `{}`", expected_id));
					notes.push(format!("   found property `{}`", found_id))
				}
				layout::Mismatch::FieldName {
					expected, found, ..
				} => {
					notes.push(format!("expected `{}`", expected));
					notes.push(format!("   found `{}`", found))
				}
				layout::Mismatch::FieldLayout {
					expected, found, ..
				} => {
					notes.push(format!(
						"expected `{}`",
						expected.with_model(self.context())
					));
					notes.push(format!("   found `{}`", found.with_model(self.context())))
				}
				_ => (),
			},
			_ => (),
		}

		notes
	}
}
