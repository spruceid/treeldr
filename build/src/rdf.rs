use crate::{error, list::ListMut, Context, Descriptions, Document, Error};
use locspan::Meta;
use treeldr::{
	metadata::Merge,
	vocab::{self, GraphLabel, Object, Term},
	Id, Name, Vocabulary,
};

fn expect_id<M>(Meta(value, loc): Meta<vocab::Object<M>, M>) -> Result<Meta<Id, M>, Error<M>> {
	match value {
		vocab::Object::Literal(_) => panic!("expected IRI or blank node label"),
		vocab::Object::Blank(id) => Ok(Meta(Id::Blank(id), loc)),
		vocab::Object::Iri(id) => Ok(Meta(Id::Iri(id), loc)),
	}
}

fn expect_boolean<M>(
	Meta(value, loc): Meta<vocab::Object<M>, M>,
) -> Result<Meta<bool, M>, Error<M>> {
	match value {
		vocab::Object::Iri(vocab::Term::Schema(vocab::Schema::True)) => Ok(Meta(true, loc)),
		vocab::Object::Iri(vocab::Term::Schema(vocab::Schema::False)) => Ok(Meta(false, loc)),
		_ => panic!("expected a boolean value"),
	}
}

fn expect_raw_string<M>(
	Meta(value, _): Meta<vocab::Object<M>, M>,
) -> Result<Meta<rdf_types::StringLiteral, M>, Error<M>> {
	match value {
		vocab::Object::Literal(rdf_types::meta::Literal::String(s)) => Ok(s),
		_ => panic!("expected a untyped and untagged string literal"),
	}
}

impl<M: Clone + Ord + Merge, D: Descriptions<M>> Document<M, D>
	for grdf::meta::BTreeDataset<Id, Term, Object<M>, GraphLabel, M>
{
	type LocalContext = ();
	type Error = Error<M>;

	fn declare(
		&self,
		_: &mut (),
		context: &mut Context<M, D>,
		_vocabulary: &mut Vocabulary,
	) -> Result<(), Error<M>> {
		// Step 1: find out the type of each node.
		for Meta(quad, loc) in self.loc_quads() {
			let Meta(id, _) = quad.subject().cloned_value();

			if let Term::Rdf(vocab::Rdf::Type) = quad.predicate().value() {
				match quad.object().value() {
					Object::Iri(Term::Rdf(vocab::Rdf::Property)) => {
						context.declare_property(id, loc.clone());
					}
					Object::Iri(Term::Rdf(vocab::Rdf::List)) => {
						context.declare_list(id, loc.clone());
					}
					Object::Iri(Term::Rdfs(vocab::Rdfs::Class)) => {
						context.declare_type(id, loc.clone());
					}
					Object::Iri(Term::Rdfs(vocab::Rdfs::Datatype)) => {
						context.declare_type(id, loc.clone());
						let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
						ty.declare_datatype(loc.clone())?;
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Layout)) => {
						context.declare_layout(id, loc.clone());
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Field)) => {
						context.declare_layout_field(id, loc.clone());
					}
					Object::Iri(Term::TreeLdr(vocab::TreeLdr::Variant)) => {
						context.declare_layout_variant(id, loc.clone());
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
		context: &mut Context<M, D>,
		_vocabulary: &mut Vocabulary,
	) -> Result<(), Error<M>> {
		// Step 2: find out the properties of each node.
		for Meta(rdf_types::Quad(subject, predicate, object, _graph), loc) in self.into_meta_quads()
		{
			let Meta(id, id_loc) = subject;

			match predicate.into_value() {
				Term::Rdf(vocab::Rdf::First) => match context.require_list_mut(id, &id_loc)? {
					ListMut::Cons(list) => list.set_first(object.into_value(), loc)?,
					ListMut::Nil => {
						panic!("nil first")
					}
				},
				Term::Rdf(vocab::Rdf::Rest) => match context.require_list_mut(id, &id_loc)? {
					ListMut::Cons(list) => {
						let Meta(object, _) = expect_id(object)?;
						list.set_rest(object, loc)?
					}
					ListMut::Nil => {
						panic!("nil rest")
					}
				},
				Term::Rdfs(vocab::Rdfs::Label) => match object.as_literal() {
					Some(label) => context.add_label(
						id,
						label.string_literal().value().as_str().to_owned(),
						loc,
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
							loc,
						);
					}
					None => {
						panic!("comment is not a string literal")
					}
				},
				Term::Rdfs(vocab::Rdfs::Domain) => {
					let (prop, field) =
						context.require_property_or_layout_field_mut(id, &id_loc)?;
					let Meta(object, object_loc) = expect_id(object)?;

					if let Some(field) = field {
						field.set_layout(object, loc.clone())?
					}

					if let Some(prop) = prop {
						prop.set_domain(object, loc.clone());
						let ty = context.require_type_mut(object, &object_loc)?;
						ty.declare_property(id, loc)?
					}
				}
				Term::Rdfs(vocab::Rdfs::Range) => {
					let prop = context.require_property_mut(id, &id_loc)?;
					let Meta(object, _) = expect_id(object)?;
					prop.set_range(object, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::Format) => {
					let (field, variant) =
						context.require_layout_field_or_variant_mut(id, &id_loc)?;
					let Meta(object, _) = expect_id(object)?;

					if let Some(field) = field {
						field.set_layout(object, loc.clone())?
					}

					if let Some(variant) = variant {
						variant.set_layout(object, loc)?
					}
				}
				Term::Schema(vocab::Schema::ValueRequired) => {
					let prop = context.require_property_mut(id, &id_loc)?;
					let Meta(required, _) = expect_boolean(object)?;
					prop.set_required(required, loc.clone())?
				}
				Term::Schema(vocab::Schema::MultipleValues) => {
					let prop = context.require_property_mut(id, &id_loc)?;
					let Meta(multiple, _) = expect_boolean(object)?;
					prop.set_functional(!multiple, loc.clone())?
				}
				Term::Owl(vocab::Owl::UnionOf) => {
					let ty = context.require_type_mut(id, &id_loc)?;
					let Meta(options_id, options_loc) = expect_id(object)?;
					ty.declare_union(options_id, options_loc)?
				}
				Term::Owl(vocab::Owl::IntersectionOf) => {
					let ty = context.require_type_mut(id, &id_loc)?;
					let Meta(types_id, types_loc) = expect_id(object)?;
					ty.declare_intersection(types_id, types_loc)?
				}
				Term::Owl(vocab::Owl::OnDatatype) => {
					todo!()
				}
				Term::Owl(vocab::Owl::WithRestrictions) => {
					todo!()
				}
				Term::TreeLdr(vocab::TreeLdr::Name) => {
					let node = context.require_mut(id, &id_loc)?;
					let Meta(name, name_loc) = expect_raw_string(object)?;

					let name = Name::new(&name).map_err(|treeldr::name::InvalidName| {
						Error::new(error::NameInvalid(name.into()).into(), name_loc)
					})?;

					if node.is_layout() || node.is_layout_field() || node.is_layout_variant() {
						if let Some(layout) = node.as_layout_mut() {
							layout.set_name(name.clone(), loc.clone())?
						}

						if let Some(field) = node.as_layout_field_mut() {
							field.set_name(name.clone(), loc.clone())?
						}

						if let Some(variant) = node.as_layout_variant_mut() {
							variant.set_name(name, loc)?
						}
					} else {
						log::warn!("unapplicable <treelrd:name> property")
					}
				}
				Term::TreeLdr(vocab::TreeLdr::LayoutFor) => {
					let Meta(ty_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_type(ty_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::Fields) => {
					let Meta(fields_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_fields(fields_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::FieldFor) => {
					let Meta(prop_id, _) = expect_id(object)?;
					let field = context.require_layout_field_mut(id, &id_loc)?;
					field.set_property(prop_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::Reference) => {
					let Meta(target_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_reference(target_id, loc)?
				}
				// Term::TreeLdr(vocab::TreeLdr::Singleton) => {
				// 	let Meta(string, _) = expect_raw_string(object)?;
				// 	let layout = context.require_layout_mut(id, &id_loc)?;
				// 	layout.set_literal(string.into(), loc)?
				// }
				// Term::TreeLdr(vocab::TreeLdr::Matches) => {
				// 	let Meta(regexp_string, regexp_loc) = expect_raw_string(object)?;
				// 	let regexp = treeldr::layout::literal::RegExp::parse(&regexp_string).map_err(
				// 		move |e| {
				// 			Error::new(
				// 				error::RegExpInvalid(regexp_string.into(), e).into(),
				// 				Some(regexp_loc),
				// 			)
				// 		},
				// 	)?;
				// 	let layout = context.require_layout_mut(id, &id_loc)?;
				// 	layout.set_literal(regexp, loc)?
				// }
				Term::TreeLdr(vocab::TreeLdr::DerivedFrom) => {
					todo!()
				}
				Term::TreeLdr(vocab::TreeLdr::WithRestrictions) => {
					todo!()
				}
				Term::TreeLdr(vocab::TreeLdr::Enumeration) => {
					let Meta(fields_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_enum(fields_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::Option) => {
					let Meta(item_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_option(item_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::Array) => {
					let Meta(item_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_array(item_id, None, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::ArrayListFirst) => {
					let Meta(prop_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_array(prop_id, None, loc.clone())?;
					layout.set_array_list_first(prop_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::ArrayListRest) => {
					let Meta(prop_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_array(prop_id, None, loc.clone())?;
					layout.set_array_list_rest(prop_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::ArrayListNil) => {
					let Meta(nil_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_array(nil_id, None, loc.clone())?;
					layout.set_array_list_nil(nil_id, loc)?
				}
				Term::TreeLdr(vocab::TreeLdr::Set) => {
					let Meta(item_id, _) = expect_id(object)?;
					let layout = context.require_layout_mut(id, &id_loc)?;
					layout.set_set(item_id, loc)?
				}
				_ => (),
			}
		}

		Ok(())
	}
}
