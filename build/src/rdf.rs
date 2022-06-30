use crate::{error, list::ListMut, Context, Descriptions, Document, Error};
use locspan::Loc;
use treeldr::{
	vocab::{self, GraphLabel, Object, Term},
	Id, Name, Vocabulary,
};

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

impl<F: Clone + Ord, D: Descriptions<F>> Document<F, D>
	for grdf::loc::BTreeDataset<Id, Term, Object<F>, GraphLabel, F>
{
	type LocalContext = ();
	type Error = Error<F>;

	fn declare(
		&self,
		_: &mut (),
		context: &mut Context<F, D>,
		_vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		// Step 1: find out the type of each node.
		for Loc(quad, loc) in self.loc_quads() {
			let Loc(id, _) = quad.subject().cloned_value();

			if let Term::Rdf(vocab::Rdf::Type) = quad.predicate().value() {
				match quad.object().value() {
					Object::Iri(Term::Rdf(vocab::Rdf::Property)) => {
						context.declare_property(id, Some(loc.cloned()));
					}
					Object::Iri(Term::Rdf(vocab::Rdf::List)) => {
						context.declare_list(id, Some(loc.cloned()));
					}
					Object::Iri(Term::Rdfs(vocab::Rdfs::Class)) => {
						context.declare_type(id, Some(loc.cloned()));
					}
					Object::Iri(Term::Rdfs(vocab::Rdfs::Datatype)) => {
						context.declare_type(id, Some(loc.cloned()));
						let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
						ty.declare_datatype(Some(loc.cloned()))?;
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Layout)) => {
						context.declare_layout(id, Some(loc.cloned()));
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Field)) => {
						context.declare_layout_field(id, Some(loc.cloned()));
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Variant)) => {
						context.declare_layout_variant(id, Some(loc.cloned()));
					}
					_ => (),
				}
			}
		}

		Ok(())
	}

	fn relate(
		self,
		_: &mut (),
		context: &mut Context<F, D>,
		_vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		// Step 2: find out the properties of each node.
		for Loc(rdf_types::Quad(subject, predicate, object, _graph), loc) in self.into_loc_quads() {
			let Loc(id, id_loc) = subject;

			match predicate.into_value() {
				Term::Rdf(vocab::Rdf::First) => match context.require_list_mut(id, Some(id_loc))? {
					ListMut::Cons(list) => list.set_first(object.into_value(), Some(loc))?,
					ListMut::Nil => {
						panic!("nil first")
					}
				},
				Term::Rdf(vocab::Rdf::Rest) => match context.require_list_mut(id, Some(id_loc))? {
					ListMut::Cons(list) => {
						let Loc(object, _) = expect_id(object)?;
						list.set_rest(object, Some(loc))?
					}
					ListMut::Nil => {
						panic!("nil rest")
					}
				},
				Term::Rdfs(vocab::Rdfs::Label) => match object.as_literal() {
					Some(label) => context.add_label(
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
						context.add_comment(
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
						context.require_property_or_layout_field_mut(id, Some(id_loc))?;
					let Loc(object, object_loc) = expect_id(object)?;

					if let Some(field) = field {
						field.set_layout(object, Some(loc.clone()))?
					}

					if let Some(prop) = prop {
						prop.set_domain(object, Some(loc.clone()));
						let ty = context.require_type_mut(object, Some(object_loc))?;
						ty.declare_property(id, Some(loc))?
					}
				}
				Term::Rdfs(vocab::Rdfs::Range) => {
					let prop = context.require_property_mut(id, Some(id_loc))?;
					let Loc(object, _) = expect_id(object)?;
					prop.set_range(object, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Format) => {
					let (field, variant) =
						context.require_layout_field_or_variant_mut(id, Some(id_loc))?;
					let Loc(object, _) = expect_id(object)?;

					if let Some(field) = field {
						field.set_layout(object, Some(loc.clone()))?
					}

					if let Some(variant) = variant {
						variant.set_layout(object, Some(loc))?
					}
				}
				Term::Schema(vocab::Schema::ValueRequired) => {
					let prop = context.require_property_mut(id, Some(id_loc))?;
					let Loc(required, _) = expect_boolean(object)?;
					prop.set_required(required, Some(loc.clone()))?
				}
				Term::Schema(vocab::Schema::MultipleValues) => {
					let prop = context.require_property_mut(id, Some(id_loc))?;
					let Loc(multiple, _) = expect_boolean(object)?;
					prop.set_functional(!multiple, Some(loc.clone()))?
				}
				Term::Owl(vocab::Owl::UnionOf) => {
					let ty = context.require_type_mut(id, Some(id_loc))?;
					let Loc(options_id, options_loc) = expect_id(object)?;
					ty.declare_union(options_id, Some(options_loc))?
				}
				Term::Owl(vocab::Owl::IntersectionOf) => {
					let ty = context.require_type_mut(id, Some(id_loc))?;
					let Loc(types_id, types_loc) = expect_id(object)?;
					ty.declare_intersection(types_id, Some(types_loc))?
				}
				Term::Owl(vocab::Owl::OnDatatype) => {
					todo!()
				}
				Term::Owl(vocab::Owl::WithRestrictions) => {
					todo!()
				}
				Term::TreeLdr(vocab::TreeLdr::Name) => {
					let node = context.require_mut(id, Some(id_loc))?;
					let Loc(name, name_loc) = expect_raw_string(object)?;

					let name = Name::new(&name).map_err(|treeldr::name::InvalidName| {
						Error::new(error::NameInvalid(name.into()).into(), Some(name_loc))
					})?;

					if node.is_layout() || node.is_layout_field() || node.is_layout_variant() {
						if let Some(layout) = node.as_layout_mut() {
							layout.set_name(name.clone(), Some(loc.clone()))?
						}

						if let Some(field) = node.as_layout_field_mut() {
							field.set_name(name.clone(), Some(loc.clone()))?
						}

						if let Some(variant) = node.as_layout_variant_mut() {
							variant.set_name(name, Some(loc))?
						}
					} else {
						log::warn!("unapplicable <treelrd:name> property")
					}
				}
				Term::TreeLdr(vocab::TreeLdr::LayoutFor) => {
					let Loc(ty_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_type(ty_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Fields) => {
					let Loc(fields_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_fields(fields_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::FieldFor) => {
					let Loc(prop_id, _) = expect_id(object)?;
					let field = context.require_layout_field_mut(id, Some(id_loc))?;
					field.set_property(prop_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Reference) => {
					let Loc(target_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_reference(target_id, Some(loc))?
				}
				// Term::TreeLdr(vocab::TreeLdr::Singleton) => {
				// 	let Loc(string, _) = expect_raw_string(object)?;
				// 	let layout = context.require_layout_mut(id, Some(id_loc))?;
				// 	layout.set_literal(string.into(), Some(loc))?
				// }
				// Term::TreeLdr(vocab::TreeLdr::Matches) => {
				// 	let Loc(regexp_string, regexp_loc) = expect_raw_string(object)?;
				// 	let regexp = treeldr::layout::literal::RegExp::parse(&regexp_string).map_err(
				// 		move |e| {
				// 			Error::new(
				// 				error::RegExpInvalid(regexp_string.into(), e).into(),
				// 				Some(regexp_loc),
				// 			)
				// 		},
				// 	)?;
				// 	let layout = context.require_layout_mut(id, Some(id_loc))?;
				// 	layout.set_literal(regexp, Some(loc))?
				// }
				Term::TreeLdr(vocab::TreeLdr::DerivedFrom) => {
					todo!()
				}
				Term::TreeLdr(vocab::TreeLdr::WithRestrictions) => {
					todo!()
				}
				Term::TreeLdr(vocab::TreeLdr::Enumeration) => {
					let Loc(fields_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_enum(fields_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Option) => {
					let Loc(item_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_option(item_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Array) => {
					let Loc(item_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_array(item_id, None, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::ArrayListFirst) => {
					let Loc(prop_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_array(prop_id, None, Some(loc.clone()))?;
					layout.set_array_list_first(prop_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::ArrayListRest) => {
					let Loc(prop_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_array(prop_id, None, Some(loc.clone()))?;
					layout.set_array_list_rest(prop_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::ArrayListNil) => {
					let Loc(nil_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_array(nil_id, None, Some(loc.clone()))?;
					layout.set_array_list_nil(nil_id, Some(loc))?
				}
				Term::TreeLdr(vocab::TreeLdr::Set) => {
					let Loc(item_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, Some(id_loc))?;
					layout.set_set(item_id, Some(loc))?
				}
				_ => (),
			}
		}

		Ok(())
	}
}
