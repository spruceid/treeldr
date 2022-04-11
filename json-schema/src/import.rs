//! JSON Schema import functions.
//!
//! Semantics follows <https://www.w3.org/2019/wot/json-schema>.
use crate::schema::{self, RegularSchema, Schema};
use iref::{Iri, IriBuf};
use locspan::{Loc, Location, Span};
use rdf_types::Quad;
use serde_json::Value;
use treeldr::{vocab, Id, Vocabulary};
use vocab::{LocQuad, Object, Term};
use std::fmt;

/// Import error.
#[derive(Debug)]
pub enum Error {
	UnsupportedType,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UnsupportedType => write!(f, "unsupported schema `type` value.")
		}
	}
}

/// Create a dummy location.
fn loc<F: Clone>(file: &F) -> Location<F> {
	Location::new(file.clone(), Span::default())
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
			let id = vocab::Term::from_iri(iri, vocabulary);
			Ok(Loc(Object::Iri(id), loc(file)))
		}
		Schema::DynamicRef(_) => todo!(),
		Schema::Regular(schema) => import_regular_schema(schema, file, base_iri, vocabulary, quads),
	}
}

#[derive(Clone, Copy)]
enum LayoutKind {
	Unknown,
	Boolean,
	Integer,
	Number,
	String,
	ArrayOrSet,
	Array,
	Set,
	Struct,
}

impl LayoutKind {
	pub fn is_struct(&self) -> bool {
		matches!(self, Self::Struct)
	}

	pub fn refine(&mut self, other: Self) -> Result<(), Error> {
		*self = match (*self, other) {
			(Self::Unknown, k) => k,
			(Self::Boolean, Self::Boolean) => Self::Boolean,
			(Self::Integer, Self::Integer) => Self::Integer,
			(Self::Number, Self::Integer) => Self::Number,
			(Self::Number, Self::Number) => Self::Number,
			(Self::ArrayOrSet, Self::Array) => Self::Array,
			(Self::ArrayOrSet, Self::Set) => Self::Set,
			(Self::ArrayOrSet, Self::ArrayOrSet) => Self::ArrayOrSet,
			(Self::Array, Self::Array) => Self::Array,
			(Self::Set, Self::Set) => Self::Set,
			(Self::Struct, Self::Struct) => Self::Struct,
			_ => return Err(Error::UnsupportedType),
		};

		Ok(())
	}
}

pub fn import_regular_schema<F: Clone>(
	schema: &RegularSchema,
	file: &F,
	base_iri: Option<Iri>,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>,
) -> Result<Loc<Object<F>, F>, Error> {
	let (id, mut name, base_iri) = match &schema.id {
		Some(iri) => {
			let id = Id::Iri(vocab::Term::from_iri(iri.clone(), vocabulary));
			let name = iri.path().file_name().and_then(|name| {
				match std::path::Path::new(name).file_stem() {
					Some(stem) => vocab::Name::new(stem.to_string_lossy()).ok(),
					None => vocab::Name::new(name.to_string()).ok()
				}
			});

			(id, name, Some(iri.clone()))
		}
		None => {
			let id = Id::Blank(vocabulary.new_blank_label());
			let base_iri = base_iri.map(IriBuf::from);
			(id, None, base_iri)
		}
	};

	if name.is_none() {
		if let Some(title) = &schema.meta_data.title {
			if let Ok(n) = vocab::Name::new(title) {
				name = Some(n)
			}
		}
	}

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

	if let Some(name) = name {
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Term::TreeLdr(vocab::TreeLdr::Name), loc(file)),
				Loc(
					Object::Literal(vocab::Literal::String(Loc(name.to_string().into(), loc(file)))),
					loc(file),
				),
				None,
			),
			loc(file),
		));
	}

	if let Some(title) = &schema.meta_data.title {
		// The title of a schema is translated in an rdfs:label.
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Term::Rdfs(vocab::Rdfs::Label), loc(file)),
				Loc(
					Object::Literal(vocab::Literal::String(Loc(title.clone().into(), loc(file)))),
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

	let mut kind = LayoutKind::Unknown;
	if let Some(types) = &schema.validation.any.ty {
		for ty in types {
			let k = match ty {
				schema::Type::Null => todo!(),
				schema::Type::Boolean => LayoutKind::Boolean,
				schema::Type::Integer => LayoutKind::Integer,
				schema::Type::Number => LayoutKind::Number,
				schema::Type::String => LayoutKind::String,
				schema::Type::Array => LayoutKind::ArrayOrSet,
				schema::Type::Object => LayoutKind::Struct,
			};

			kind.refine(k)?
		}
	}

	match &schema.desc {
		schema::Description::Definition {
			string,
			array,
			object,
		} => {
			if !string.is_empty() {
				todo!()
			}

			if !array.is_empty() || !schema.validation.array.is_empty() {
				kind.refine(LayoutKind::ArrayOrSet)?;
				import_array_schema(
					id,
					schema,
					array,
					&mut kind,
					file,
					base_iri.as_ref().map(IriBuf::as_iri),
					vocabulary,
					quads,
				)?;
			}

			if kind.is_struct() || !object.is_empty() || !schema.validation.object.is_empty() {
				kind.refine(LayoutKind::Struct)?;
				import_object_schema(
					id,
					schema,
					object,
					file,
					base_iri.as_ref().map(IriBuf::as_iri),
					vocabulary,
					quads,
				)?;
			}
		}
		// schema::Description::OneOf(schemas) => {
		// 	todo!()
		// }
		_ => todo!(),
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

	let native_layout = if let Some(format) = schema.validation.format {
		Some(format_layout(file, format)?)
	} else {
		match kind {
			LayoutKind::Boolean => {
				Some(Loc(Object::Iri(Term::Xsd(vocab::Xsd::Boolean)), loc(file)))
			}
			LayoutKind::Integer => {
				Some(Loc(Object::Iri(Term::Xsd(vocab::Xsd::Integer)), loc(file)))
			}
			LayoutKind::Number => Some(Loc(Object::Iri(Term::Xsd(vocab::Xsd::Double)), loc(file))),
			LayoutKind::String => Some(Loc(Object::Iri(Term::Xsd(vocab::Xsd::String)), loc(file))),
			_ => None,
		}
	};

	if let Some(layout) = native_layout {
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

#[allow(clippy::too_many_arguments)]
fn import_array_schema<F: Clone>(
	id: Id,
	schema: &RegularSchema,
	array: &schema::ArraySchema,
	kind: &mut LayoutKind,
	file: &F,
	base_iri: Option<Iri>,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>,
) -> Result<(), Error> {
	let layout_kind = if matches!(schema.validation.array.unique_items, Some(true)) {
		kind.refine(LayoutKind::Set)?;
		Loc(Term::TreeLdr(vocab::TreeLdr::Set), loc(file))
	} else {
		kind.refine(LayoutKind::Array)?;
		Loc(Term::TreeLdr(vocab::TreeLdr::List), loc(file))
	};

	let item_type = match &array.items {
		Some(items) => import_schema(items, file, base_iri, vocabulary, quads)?,
		None => todo!(),
	};

	quads.push(Loc(
		Quad(Loc(id, loc(file)), layout_kind, item_type, None),
		loc(file),
	));

	Ok(())
}

fn import_object_schema<F: Clone>(
	id: Id,
	schema: &RegularSchema,
	object: &schema::ObjectSchema,
	file: &F,
	base_iri: Option<Iri>,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>,
) -> Result<(), Error> {
	let mut fields: Vec<Loc<Object<F>, F>> = Vec::new();

	if let Some(properties) = &object.properties {
		fields.reserve(properties.len());

		// First, we build each field.
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

			let prop_schema = import_schema(prop_schema, file, base_iri, vocabulary, quads)?;
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

	Ok(())
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
		schema::Format::Regex => todo!(),
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
