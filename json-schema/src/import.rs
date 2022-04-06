//! JSON Schema import functions.
//!
//! Semantics follows <https://www.w3.org/2019/wot/json-schema>.
use iref::{Iri, IriBuf};
use locspan::{Loc, Location, Span};
use rdf_types::Quad;
use serde_json::Value;
use treeldr::{vocab, Id, Vocabulary};
use vocab::{LocQuad, Object, Term};
use crate::schema::{
	self,
	Schema,
	RegularSchema
};

/// Import error.
pub enum Error {
	InvalidJson(serde_json::error::Error),
	InvalidSchema(crate::schema::from_serde_json::Error),
}

impl From<serde_json::error::Error> for Error {
	fn from(e: serde_json::error::Error) -> Self {
		Self::InvalidJson(e)
	}
}

impl From<crate::schema::from_serde_json::Error> for Error {
	fn from(e: crate::schema::from_serde_json::Error) -> Self {
		Self::InvalidSchema(e)
	}
}

/// Create a dummy location.
fn loc<F: Clone>(file: &F) -> Location<F> {
	Location::new(file.clone(), Span::default())
}

pub fn import<F: Clone>(
	content: &str,
	file: F,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>,
) -> Result<(), Error> {
	let json: Value = serde_json::from_str(content)?;
	let schema: Schema = json.try_into()?;

	import_schema(&schema, &file, None, vocabulary, quads)?;

	Ok(())
}

pub fn import_schema<F: Clone>(
	schema: &Schema,
	file: &F,
	base_iri: Option<Iri>,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>,
) -> Result<Loc<Object<F>, F>, Error> {
	match schema {
		Schema::True => todo!(),
		Schema::False => todo!(),
		Schema::Ref(r) => {
			let iri = r.target.resolved(base_iri.unwrap());
			let id = vocab::Term::from_iri(iri.clone(), vocabulary);
			Ok(Loc(Object::Iri(id), loc(file)))
		}
		Schema::DynamicRef(_) => todo!(),
		Schema::Regular(schema) => {
			import_regular_schema(schema, file, base_iri, vocabulary, quads)
		}
	}
}

pub fn import_regular_schema<F: Clone>(
	schema: &RegularSchema,
	file: &F,
	base_iri: Option<Iri>,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>,
) -> Result<Loc<Object<F>, F>, Error> {
	let (id, base_iri) = match &schema.id {
		Some(iri) => {
			let id = Id::Iri(vocab::Term::from_iri(iri.clone(), vocabulary));
			(id, Some(iri.clone()))
		}
		None => {
			let id = Id::Blank(vocabulary.new_blank_label());
			let base_iri = base_iri.map(IriBuf::from);
			(id, base_iri)
		}
	};

	// Declare the layout.
	quads.push(Loc(
		Quad(
			Loc(id, loc(file)),
			Loc(Term::Rdf(vocab::Rdf::Type), loc(file)),
			Loc(
				Object::Iri(Term::TreeLdr(vocab::TreeLdr::Layout)),
				loc(file),
			),
			None,
		),
		loc(file),
	));

	if let Some(title) = &schema.meta_data.title {
		// The title of a schema is translated in an rdfs:label.
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Term::Rdfs(vocab::Rdfs::Label), loc(file)),
				Loc(
					Object::Literal(vocab::Literal::String(Loc(
						title.clone().into(),
						loc(file),
					))),
					loc(file),
				),
				None,
			),
			loc(file),
		));
	}

	if let Some(description) = &schema.meta_data.description {
		// The title of a schema is translated in an rdfs:comment.
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Term::Rdfs(vocab::Rdfs::Comment), loc(file)),
				Loc(
					Object::Literal(vocab::Literal::String(Loc(
						description.clone().into(),
						loc(file),
					))),
					loc(file),
				),
				None,
			),
			loc(file),
		));
	}

	match &schema.desc {
		schema::Description::Definition { string, array, object } => {
			if let Some(properties) = &object.properties {
				// The presence of this key means that the schema represents a TreeLDR structure
				// layout.
				// First, we build each field.
				let mut fields: Vec<Loc<Object<F>, F>> = Vec::with_capacity(properties.len());
				for (prop, prop_schema) in properties {
					let prop_label = vocabulary.new_blank_label();
					// <prop> rdf:type treeldr:Field
					quads.push(Loc(
						Quad(
							Loc(Id::Blank(prop_label), loc(file)),
							Loc(Term::Rdf(vocab::Rdf::Type), loc(file)),
							Loc(Object::Iri(Term::TreeLdr(vocab::TreeLdr::Field)), loc(file)),
							None,
						),
						loc(file),
					));
					// <prop> treeldr:name <name>
					quads.push(Loc(
						Quad(
							Loc(Id::Blank(prop_label), loc(file)),
							Loc(Term::TreeLdr(vocab::TreeLdr::Name), loc(file)),
							Loc(
								Object::Literal(vocab::Literal::String(Loc(
									prop.to_string().into(),
									loc(file),
								))),
								loc(file),
							),
							None,
						),
						loc(file),
					));

					let prop_schema = import_schema(
						prop_schema,
						file,
						base_iri.as_ref().map(IriBuf::as_iri),
						vocabulary,
						quads,
					)?;
					quads.push(Loc(
						Quad(
							Loc(Id::Blank(prop_label), loc(file)),
							Loc(Term::TreeLdr(vocab::TreeLdr::Format), loc(file)),
							prop_schema,
							None,
						),
						loc(file),
					));

					let field = Loc(Object::Blank(prop_label), loc(file));

					fields.push(field);

					// property_fields.insert(prop, Loc(Id::Blank(prop_label), loc(file)));
					if let Some(required) = &schema.validation.object.required {
						if required.contains(prop) {
							quads.push(Loc(
								Quad(
									Loc(Id::Blank(prop_label), loc(file)),
									Loc(Term::Schema(vocab::Schema::ValueRequired), loc(file)),
									Loc(Object::Iri(Term::Schema(vocab::Schema::True)), loc(file)),
									None,
								),
								loc(file),
							));
						}
					}
				}

				let fields = fields.into_iter().try_into_rdf_list::<Error, _>(
					&mut (),
					vocabulary,
					quads,
					loc(file),
					|field, _, _, _| Ok(field),
				)?;

				// Then we declare the structure content.
				quads.push(Loc(
					Quad(
						Loc(id, loc(file)),
						Loc(Term::TreeLdr(vocab::TreeLdr::Fields), loc(file)),
						fields,
						None,
					),
					loc(file),
				));
			}
		}
		schema::Description::OneOf(schemas) => {
			todo!()
		}
		_ => todo!()
	}

	if let Some(cnst) = &schema.validation.any.cnst {
		// The presence of this key means that the schema represents a TreeLDR
		// literal/singleton layout.
		let singleton = value_into_object(file, vocabulary, quads, cnst)?;
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Term::TreeLdr(vocab::TreeLdr::Singleton), loc(file)),
				singleton,
				None,
			),
			loc(file),
		));
	}

	if let Some(pattern) = &schema.validation.string.pattern {
		// The presence of this key means that the schema represents a TreeLDR literal
		// regular expression layout.
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Term::TreeLdr(vocab::TreeLdr::Matches), loc(file)),
				Loc(
					Object::Literal(vocab::Literal::String(Loc(
						pattern.clone().into(),
						loc(file),
					))),
					loc(file),
				),
				None,
			),
			loc(file),
		));
	}

	if let Some(format) = schema.validation.format {
		let layout = format_layout(file, format)?;
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Term::TreeLdr(vocab::TreeLdr::Native), loc(file)),
				layout,
				None,
			),
			loc(file),
		));
	}

	let result = match id {
		Id::Iri(id) => Object::Iri(id),
		Id::Blank(id) => Object::Blank(id),
	};

	Ok(Loc(result, loc(file)))
}

fn value_into_object<F: Clone>(
	file: &F,
	vocab: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>,
	value: &Value,
) -> Result<Loc<Object<F>, F>, Error> {
	match value {
		Value::Null => todo!(),
		Value::Bool(true) => Ok(Loc(
			Object::Iri(vocab::Term::Schema(vocab::Schema::True)),
			loc(file),
		)),
		Value::Bool(false) => Ok(Loc(
			Object::Iri(vocab::Term::Schema(vocab::Schema::False)),
			loc(file),
		)),
		Value::Number(n) => Ok(Loc(
			Object::Literal(vocab::Literal::TypedString(
				Loc(n.to_string().into(), loc(file)),
				Loc(vocab::Term::Xsd(vocab::Xsd::Integer), loc(file)),
			)),
			loc(file),
		)),
		Value::String(s) => Ok(Loc(
			Object::Literal(vocab::Literal::String(Loc(s.to_string().into(), loc(file)))),
			loc(file),
		)),
		Value::Array(items) => items.iter().try_into_rdf_list(
			&mut (),
			vocab,
			quads,
			loc(file),
			|item, _, vocab, quads| value_into_object(file, vocab, quads, item),
		),
		Value::Object(_) => todo!(),
	}
}

fn format_layout<F: Clone>(file: &F, format: schema::Format) -> Result<Loc<Object<F>, F>, Error> {
	let layout = match format {
		schema::Format::DateTime => Term::Xsd(vocab::Xsd::DateTime),
		schema::Format::Date => Term::Xsd(vocab::Xsd::Date),
		schema::Format::Time => Term::Xsd(vocab::Xsd::Time),
		schema::Format::Duration => todo!(),
		schema::Format::Email => todo!(),
		schema::Format::IdnEmail => todo!(),
		schema::Format::Hostname => todo!(),
		schema::Format::IdnHostname => todo!(),
		schema::Format::Ipv4 => todo!(),
		schema::Format::Ipv6 => todo!(),
		schema::Format::Uri => todo!(),
		schema::Format::UriReference => todo!(),
		schema::Format::Iri => Term::Xsd(vocab::Xsd::AnyUri),
		schema::Format::IriReference => todo!(),
		schema::Format::Uuid => todo!(),
		schema::Format::UriTemplate => todo!(),
		schema::Format::JsonPointer => todo!(),
		schema::Format::RelativeJsonPointer => todo!(),
		schema::Format::Regex => todo!()
	};

	Ok(Loc(Object::Iri(layout), loc(file)))
}

pub trait TryIntoRdfList<F, C, T> {
	fn try_into_rdf_list<E, K>(
		self,
		ctx: &mut C,
		vocab: &mut Vocabulary,
		quads: &mut Vec<LocQuad<F>>,
		loc: Location<F>,
		f: K,
	) -> Result<Loc<Object<F>, F>, E>
	where
		K: FnMut(T, &mut C, &mut Vocabulary, &mut Vec<LocQuad<F>>) -> Result<Loc<Object<F>, F>, E>;
}

impl<F: Clone, C, I: DoubleEndedIterator> TryIntoRdfList<F, C, I::Item> for I {
	fn try_into_rdf_list<E, K>(
		self,
		ctx: &mut C,
		vocab: &mut Vocabulary,
		quads: &mut Vec<LocQuad<F>>,
		loc: Location<F>,
		mut f: K,
	) -> Result<Loc<Object<F>, F>, E>
	where
		K: FnMut(
			I::Item,
			&mut C,
			&mut Vocabulary,
			&mut Vec<LocQuad<F>>,
		) -> Result<Loc<Object<F>, F>, E>,
	{
		use vocab::Rdf;
		let mut head = Loc(Object::Iri(Term::Rdf(Rdf::Nil)), loc);
		for item in self.rev() {
			let item = f(item, ctx, vocab, quads)?;
			let item_label = vocab.new_blank_label();
			let item_loc = item.location().clone();
			let list_loc = head.location().clone().with(item_loc.span());

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), list_loc.clone()),
					Loc(Term::Rdf(Rdf::Type), list_loc.clone()),
					Loc(Object::Iri(Term::Rdf(Rdf::List)), list_loc.clone()),
					None,
				),
				item_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), item_loc.clone()),
					Loc(Term::Rdf(Rdf::First), item_loc.clone()),
					item,
					None,
				),
				item_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), head.location().clone()),
					Loc(Term::Rdf(Rdf::Rest), head.location().clone()),
					head,
					None,
				),
				item_loc.clone(),
			));

			head = Loc(Object::Blank(item_label), list_loc);
		}

		Ok(head)
	}
}
