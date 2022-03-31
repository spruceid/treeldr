use crate::{
	error,
	vocab::{self, GraphLabel, Term, Object},
	Error, Id, Vocabulary,
};
use locspan::Loc;

mod context;
pub mod layout;
pub mod list;
pub mod node;
pub mod prop;
pub mod ty;

pub use context::Context;
pub use list::{ListMut, ListRef};
pub use node::Node;

fn expect_id<F>(Loc(value, loc): Loc<vocab::Object<F>, F>) -> Result<Loc<Id, F>, Error<F>> {
	match value {
		vocab::Object::Literal(_) => panic!("expected IRI or blank node label"),
		vocab::Object::Blank(id) => Ok(Loc(Id::Blank(id), loc)),
		vocab::Object::Iri(id) => Ok(Loc(Id::Iri(id), loc)),
	}
}

fn expect_boolean<F>(Loc(value, loc): Loc<vocab::Object<F>, F>) -> Result<Loc<bool, F>, Error<F>> {
	match value {
		vocab::Object::Iri(vocab::Term::Schema(vocab::Schema::True)) => Ok(Loc(true, loc)),
		vocab::Object::Iri(vocab::Term::Schema(vocab::Schema::False)) => Ok(Loc(false, loc)),
		_ => panic!("expected a boolean value"),
	}
}

fn expect_raw_string<F>(
	Loc(value, _): Loc<vocab::Object<F>, F>,
) -> Result<Loc<rdf_types::StringLiteral, F>, Error<F>> {
	match value {
		vocab::Object::Literal(rdf_types::loc::Literal::String(s)) => Ok(s),
		_ => panic!("expected a untyped and untagged string literal"),
	}
}

pub type ErrorWithVocabulary<F> = (Error<F>, Vocabulary);

impl<F: Clone + Ord> Context<F> {
	pub fn build_dataset(
		mut self,
		dataset: grdf::loc::BTreeDataset<Id, Term, Object<F>, GraphLabel, F>,
	) -> Result<crate::Model<F>, ErrorWithVocabulary<F>> {
		match self.add_dataset(dataset) {
			Ok(()) => self.build(),
			Err(e) => Err((e, self.into_vocabulary())),
		}
	}

	pub fn add_dataset(
		&mut self,
		dataset: grdf::loc::BTreeDataset<Id, Term, Object<F>, GraphLabel, F>,
	) -> Result<(), Error<F>> {
		// Step 1: find out the type of each node.
		for Loc(quad, loc) in dataset.loc_quads() {
			let Loc(id, _) = quad.subject().cloned_value();

			if let Term::Rdf(vocab::Rdf::Type) = quad.predicate().value() {
				match quad.object().value() {
					Object::Iri(Term::Rdf(vocab::Rdf::Property)) => {
						self.declare_property(id, Some(loc.cloned()));
					}
					Object::Iri(Term::Rdf(vocab::Rdf::List)) => {
						self.declare_list(id, Some(loc.cloned()));
					}
					Object::Iri(Term::Rdfs(vocab::Rdfs::Class)) => {
						self.declare_type(id, Some(loc.cloned()));
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Layout)) => {
						self.declare_layout(id, Some(loc.cloned()));
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Field)) => {
						self.declare_layout_field(id, Some(loc.cloned()));
					}
					_ => (),
				}
			}
		}

		// Step 2: find out the properties of each node.
		for Loc(rdf_types::Quad(subject, predicate, object, _graph), loc) in
			dataset.into_loc_quads()
		{
			let Loc(id, id_loc) = subject;

			match predicate.into_value() {
				Term::Rdf(vocab::Rdf::First) => match self.require_list_mut(id, Some(id_loc))? {
					ListMut::Cons(list) => list.set_first(object.into_value(), Some(loc))?,
					ListMut::Nil => {
						panic!("nil first")
					}
				},
				Term::Rdf(vocab::Rdf::Rest) => match self.require_list_mut(id, Some(id_loc))? {
					ListMut::Cons(list) => {
						let Loc(object, _) = expect_id(object)?;
						list.set_rest(object, Some(loc))?
					}
					ListMut::Nil => {
						panic!("nil rest")
					}
				},
				Term::Rdfs(vocab::Rdfs::Label) => match object.as_literal() {
					Some(label) => self.add_label(
						id,
						label.string_literal().value().as_str().to_owned(),
						Some(loc),
					),
					None => {
						panic!("label is not a string literal")
					}
				},
				Term::Rdfs(vocab::Rdfs::Comment) => match object.as_literal() {
					Some(literal) => {
						self.add_comment(
							id,
							literal.string_literal().value().as_str().to_owned(),
							Some(loc),
						);
					}
					None => {
						panic!("comment is not a string literal")
					}
				},
				Term::Rdfs(vocab::Rdfs::Domain) => {
					let (prop, field) =
						self.require_property_or_layout_field_mut(id, Some(id_loc))?;
					let Loc(object, object_loc) = expect_id(object)?;

					if let Some(field) = field {
						field.set_layout(object, Some(loc.clone()))?
					}

					if let Some(prop) = prop {
						prop.set_domain(object, Some(loc.clone()));
						let ty = self.require_type_mut(object, Some(object_loc))?;
						ty.declare_property(id, Some(loc))
					}
				}
				Term::Rdfs(vocab::Rdfs::Range) => {
					let (prop, field) =
						self.require_property_or_layout_field_mut(id, Some(id_loc))?;
					let Loc(object, _) = expect_id(object)?;

					if let Some(prop) = prop {
						prop.set_range(object, Some(loc.clone()))?
					}

					if let Some(field) = field {
						field.set_layout(object, Some(loc))?
					}
				}
				Term::Schema(vocab::Schema::ValueRequired) => {
					let (prop, field) =
						self.require_property_or_layout_field_mut(id, Some(id_loc))?;
					let Loc(required, _) = expect_boolean(object)?;

					if let Some(prop) = prop {
						prop.set_required(required, Some(loc.clone()))?
					}

					if let Some(field) = field {
						field.set_required(required, Some(loc))?
					}
				}
				Term::Schema(vocab::Schema::MultipleValues) => {
					let (prop, field) =
						self.require_property_or_layout_field_mut(id, Some(id_loc))?;
					let Loc(multiple, _) = expect_boolean(object)?;

					if let Some(prop) = prop {
						prop.set_functional(!multiple, Some(loc.clone()))?
					}

					if let Some(field) = field {
						field.set_functional(!multiple, Some(loc))?
					}
				}
				Term::TreeLdr(vocab::TreeLdr::Name) => {
					let node = self.require_mut(id, Some(id_loc))?;
					let Loc(name, name_loc) = expect_raw_string(object)?;

					let name = vocab::Name::new(&name).map_err(|vocab::InvalidName| Error::new(
						error::NameInvalid(name).into(),
						Some(name_loc),
					))?;

					if node.is_layout() || node.is_layout_field() {
						if let Some(layout) = node.as_layout_mut() {
							layout.set_name(name.clone().into(), Some(loc.clone()))?
						}

						if let Some(field) = node.as_layout_field_mut() {
							field.set_name(name.into(), Some(loc))?
						}
					} else {
						log::warn!("unapplicable <treelrd:name> property")
					}
				}
				Term::TreeLdr(vocab::TreeLdr::LayoutFor) => {
					let Loc(ty_id, _) = expect_id(object)?;
					let layout = self.require_layout_mut(id, Some(id_loc))?;
					layout.set_type(ty_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Fields) => {
					let Loc(fields_id, _) = expect_id(object)?;
					let layout = self.require_layout_mut(id, Some(id_loc))?;
					layout.set_fields(fields_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::FieldFor) => {
					let Loc(prop_id, _) = expect_id(object)?;
					let field = self.require_layout_field_mut(id, Some(id_loc))?;
					field.set_property(prop_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::DerefTo) => {
					let Loc(target_id, _) = expect_id(object)?;
					let layout = self.require_layout_mut(id, Some(id_loc))?;
					layout.set_deref_to(target_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Singleton) => {
					let Loc(string, _) = expect_raw_string(object)?;
					let layout = self.require_layout_mut(id, Some(id_loc))?;
					layout.set_literal(string.into(), Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Matches) => {
					let Loc(regexp_string, regexp_loc) = expect_raw_string(object)?;
					let regexp = crate::layout::literal::RegExp::parse(&regexp_string).map_err(
						move |e| {
							Error::new(
								error::RegExpInvalid(regexp_string, e).into(),
								Some(regexp_loc),
							)
						},
					)?;
					let layout = self.require_layout_mut(id, Some(id_loc))?;
					layout.set_literal(regexp, Some(loc))?
				}
				_ => (),
			}
		}

		Ok(())
	}
}
