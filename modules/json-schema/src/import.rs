//! JSON Schema import functions.
use crate::schema::{self, RegularSchema, Schema};
use iref::{Iri, IriBuf};
use locspan::{Loc, Location};
use rdf_types::Quad;
use treeldr::{vocab, Caused, Id, Name, Vocabulary};
use treeldr_build::{Context, Descriptions};
use vocab::{LocQuad, Object, Term};

/// Import error.
#[derive(Debug)]
pub enum Error<F> {
	UnsupportedType,
	InvalidPropertyName(String),
	Build(treeldr_build::Error<F>),
}

impl<F> From<treeldr_build::Error<F>> for Error<F> {
	fn from(e: treeldr_build::Error<F>) -> Self {
		Self::Build(e)
	}
}

impl<F: Clone> treeldr::reporting::DiagnoseWithVocabulary<F> for Error<F> {
	fn message(&self, vocabulary: &Vocabulary) -> String {
		match self {
			Self::UnsupportedType => "unsupported schema `type` value.".to_string(),
			Self::InvalidPropertyName(name) => format!("invalid property name `{}`", name),
			Self::Build(e) => e.message(vocabulary),
		}
	}
}

pub fn import_schema<F: Clone + Ord, D: Descriptions<F>>(
	schema: &Schema,
	base_iri: Option<Iri>,
	context: &mut Context<F, D>,
	vocabulary: &mut Vocabulary,
) -> Result<Id, Error<F>> {
	match schema {
		Schema::True => todo!(),
		Schema::False => {
			let id = Id::Blank(vocabulary.new_blank_label());
			context.declare_layout(id, None);
			Ok(id)
		}
		Schema::Ref(r) => {
			let iri = r.target.resolved(base_iri.unwrap());
			let id = Id::Iri(vocab::Term::from_iri(iri, vocabulary));
			Ok(id)
		}
		Schema::DynamicRef(_) => todo!(),
		Schema::Regular(schema) => {
			import_regular_schema(schema, true, base_iri, context, vocabulary)
		}
	}
}

pub fn import_sub_schema<F: Clone + Ord, D: Descriptions<F>>(
	schema: &Schema,
	base_iri: Option<Iri>,
	context: &mut Context<F, D>,
	vocabulary: &mut Vocabulary,
) -> Result<Id, Error<F>> {
	match schema {
		Schema::True => todo!(),
		Schema::False => {
			let id = Id::Blank(vocabulary.new_blank_label());
			context.declare_layout(id, None);
			Ok(id)
		}
		Schema::Ref(r) => {
			let iri = r.target.resolved(base_iri.unwrap());
			let id = Id::Iri(vocab::Term::from_iri(iri, vocabulary));
			Ok(id)
		}
		Schema::DynamicRef(_) => todo!(),
		Schema::Regular(schema) => {
			import_regular_schema(schema, false, base_iri, context, vocabulary)
		}
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
	// pub fn is_number(&self) -> bool {
	// 	matches!(self, Self::Integer | Self::Number)
	// }

	// pub fn is_string(&self) -> bool {
	// 	matches!(self, Self::String)
	// }

	// pub fn is_struct(&self) -> bool {
	// 	matches!(self, Self::Struct)
	// }

	pub fn refine<F>(&mut self, other: Self) -> Result<(), Error<F>> {
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

pub fn import_regular_schema<F: Clone + Ord, D: Descriptions<F>>(
	schema: &RegularSchema,
	_top_level: bool,
	base_iri: Option<Iri>,
	context: &mut Context<F, D>,
	vocabulary: &mut Vocabulary,
) -> Result<Id, Error<F>> {
	if let Some(defs) = &schema.defs {
		for schema in defs.values() {
			import_sub_schema(schema, base_iri, context, vocabulary)?;
		}
	}

	let base_iri = match &schema.id {
		Some(iri) => Some(iri.clone()),
		None => base_iri.map(IriBuf::from),
	};

	let desc = import_layout_description(
		schema,
		base_iri.as_ref().map(Iri::from),
		context,
		vocabulary,
	)?;

	if let treeldr_build::layout::Description::Primitive(primitive) = &desc {
		if schema.is_primitive()
			&& schema.id.is_none()
			&& schema.meta_schema.is_empty()
			&& schema.meta_data.title.is_none()
			&& schema.anchor.is_none()
			&& schema.dynamic_anchor.is_none()
		{
			return Ok(primitive.primitive().value().unwrap().id());
		}
	}

	let (id, mut name) = match &schema.id {
		Some(iri) => {
			let id = Id::Iri(vocab::Term::from_iri(iri.clone(), vocabulary));
			let name = Name::from_iri(iri.as_iri()).ok().flatten();

			(id, name)
		}
		None => {
			let id = Id::Blank(vocabulary.new_blank_label());
			(id, None)
		}
	};

	if name.is_none() {
		if let Some(title) = &schema.meta_data.title {
			if let Ok(n) = Name::new(title) {
				name = Some(n)
			}
		}
	}

	// Declare the layout.
	context.declare_layout(id, None);

	let node = context.get_mut(id).unwrap();
	if let Some(title) = &schema.meta_data.title {
		// The title of a schema is translated in an rdfs:label.
		node.add_label(title.clone());
	}

	if let Some(description) = &schema.meta_data.description {
		// The title of a schema is translated in an rdfs:comment.
		node.documentation_mut().add(description.clone());
	}

	let layout = node.as_layout_mut().unwrap();
	if let Some(name) = name {
		layout.set_name(name, None)?;
	}

	layout.set_description(desc.into(), None)?;

	Ok(id)
}

fn into_numeric(
	primitive: treeldr::layout::Primitive,
	n: &serde_json::Number,
) -> treeldr::value::Numeric {
	use treeldr::value;
	match primitive {
		treeldr::layout::Primitive::Float => match n.as_f64() {
			Some(d) => {
				treeldr::value::Numeric::Float(value::Float::new((d as f32).try_into().unwrap()))
			}
			None => todo!(),
		},
		treeldr::layout::Primitive::Double => match n.as_f64() {
			Some(d) => treeldr::value::Numeric::Double(value::Double::new(d.try_into().unwrap())),
			None => todo!(),
		},
		treeldr::layout::Primitive::Integer => match xsd_types::IntegerBuf::new(n.to_string()) {
			Ok(n) => treeldr::value::Numeric::Integer(n.into()),
			Err(_) => todo!(),
		},
		treeldr::layout::Primitive::UnsignedInteger => {
			match xsd_types::IntegerBuf::new(n.to_string()) {
				Ok(n) => {
					if n.is_negative() {
						todo!()
					} else {
						treeldr::value::Numeric::NonNegativeInteger(unsafe {
							value::NonNegativeInteger::new_unchecked(n.into())
						})
					}
				}
				Err(_) => todo!(),
			}
		}
		_ => todo!(),
	}
}

fn import_layout_description<F: Clone + Ord, D: Descriptions<F>>(
	schema: &RegularSchema,
	base_iri: Option<Iri>,
	context: &mut Context<F, D>,
	vocabulary: &mut Vocabulary,
) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
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

	let primitive_layout = if let Some(format) = schema.validation.format {
		kind.refine(LayoutKind::String)?;
		Some(format_layout(format)?)
	} else {
		match kind {
			LayoutKind::Boolean => Some(treeldr::layout::Primitive::Boolean),
			LayoutKind::Integer => Some(treeldr::layout::Primitive::Integer),
			LayoutKind::Number => Some(treeldr::layout::Primitive::Double),
			LayoutKind::String => Some(treeldr::layout::Primitive::String),
			_ => None,
		}
	};

	if let Some(layout) = primitive_layout {
		if schema.is_primitive() {
			return Ok(treeldr_build::layout::Description::Primitive(layout.into()));
		}
	}

	match &schema.desc {
		schema::Description::Definition {
			string,
			array,
			object,
		} => {
			if !string.is_empty() || !schema.validation.string.is_empty() {
				kind.refine(LayoutKind::String)?;
			}

			if !array.is_empty() || !schema.validation.array.is_empty() {
				kind.refine(LayoutKind::ArrayOrSet)?;
			}

			if !object.is_empty() || !schema.validation.object.is_empty() {
				kind.refine(LayoutKind::Struct)?;
			}

			match kind {
				LayoutKind::Unknown => todo!(),
				LayoutKind::Boolean => {
					todo!()
				}
				LayoutKind::Number | LayoutKind::Integer => {
					if schema.validation.numeric.is_empty() {
						Ok(treeldr_build::layout::Description::Primitive(
							primitive_layout.unwrap().into(),
						))
					} else {
						use treeldr_build::layout::primitive::{restriction::Numeric, Restriction};

						let primitive = primitive_layout.unwrap();
						let mut restricted =
							treeldr_build::layout::primitive::Restricted::unrestricted(
								primitive, None,
							);
						let restrictions = restricted.restrictions_mut();

						if let Some(min) = &schema.validation.numeric.minimum {
							restrictions.insert(
								Restriction::Numeric(Numeric::InclusiveMinimum(into_numeric(
									primitive, min,
								))),
								None,
							)
						}

						if let Some(min) = &schema.validation.numeric.exclusive_minimum {
							restrictions.insert(
								Restriction::Numeric(Numeric::ExclusiveMinimum(into_numeric(
									primitive, min,
								))),
								None,
							)
						}

						if let Some(max) = &schema.validation.numeric.maximum {
							restrictions.insert(
								Restriction::Numeric(Numeric::InclusiveMaximum(into_numeric(
									primitive, max,
								))),
								None,
							)
						}

						if let Some(max) = &schema.validation.numeric.exclusive_maximum {
							restrictions.insert(
								Restriction::Numeric(Numeric::ExclusiveMaximum(into_numeric(
									primitive, max,
								))),
								None,
							)
						}

						Ok(treeldr_build::layout::Description::Primitive(restricted))
					}
				}
				LayoutKind::String => {
					use treeldr_build::layout::primitive::{restriction::String, Restriction};

					let mut restricted = treeldr_build::layout::RestrictedPrimitive::unrestricted(
						treeldr::layout::Primitive::String,
						None,
					);
					let restrictions = restricted.restrictions_mut();

					if let Some(cnst) = &schema.validation.any.cnst {
						restrictions.insert(
							Restriction::String(String::Pattern(cnst.to_string().into())),
							None,
						);
					}

					if let Some(pattern) = &schema.validation.string.pattern {
						restrictions.insert(
							Restriction::String(String::Pattern(pattern.to_string().into())),
							None,
						);
					}

					// TODO: for now, most string constraints are ignored.
					Ok(treeldr_build::layout::Description::Primitive(restricted))
				}
				LayoutKind::ArrayOrSet | LayoutKind::Array | LayoutKind::Set => {
					import_array_schema(
						schema, false, array, &mut kind, base_iri, context, vocabulary,
					)
				}
				LayoutKind::Struct => {
					import_object_schema(schema, false, object, base_iri, context, vocabulary)
				}
			}
		}
		_ => todo!(),
	}
}

#[allow(clippy::too_many_arguments)]
fn import_array_schema<F: Clone + Ord, D: Descriptions<F>>(
	schema: &RegularSchema,
	_top_level: bool,
	array: &schema::ArraySchema,
	kind: &mut LayoutKind,
	base_iri: Option<Iri>,
	context: &mut Context<F, D>,
	vocabulary: &mut Vocabulary,
) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
	let item_type = match &array.items {
		Some(items) => import_sub_schema(items, base_iri, context, vocabulary)?,
		None => todo!(),
	};

	if matches!(schema.validation.array.unique_items, Some(true)) {
		kind.refine(LayoutKind::Set)?;
		Ok(treeldr_build::layout::Description::Set(item_type))
	} else {
		kind.refine(LayoutKind::Array)?;
		Ok(treeldr_build::layout::Description::Array(
			treeldr_build::layout::Array::new(item_type, None),
		))
	}
}

fn import_object_schema<F: Clone + Ord, D: Descriptions<F>>(
	schema: &RegularSchema,
	_top_level: bool,
	object: &schema::ObjectSchema,
	base_iri: Option<Iri>,
	context: &mut Context<F, D>,
	vocabulary: &mut Vocabulary,
) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
	let mut fields: Vec<Caused<Object<F>, F>> = Vec::new();

	if let Some(properties) = &object.properties {
		fields.reserve(properties.len());

		// First, we build each field.
		for (prop, prop_schema) in properties {
			let mut layout_id = import_sub_schema(prop_schema, base_iri, context, vocabulary)?;

			let mut is_required = false;
			if let Some(required) = &schema.validation.object.required {
				if required.contains(prop) {
					is_required = true;
				}
			}

			if !is_required {
				// Wrap inside option.
				let item_layout_id = layout_id;
				layout_id = Id::Blank(vocabulary.new_blank_label());
				context.declare_layout(layout_id, None);
				let container_layout = context.get_mut(layout_id).unwrap().as_layout_mut().unwrap();
				container_layout.set_option(item_layout_id, None)?;
			}

			let field_id = Id::Blank(vocabulary.new_blank_label());
			context.declare_layout_field(field_id, None);
			let field_node = context.get_mut(field_id).unwrap();

			if let Some(meta) = &prop_schema.meta_data() {
				if let Some(doc) = &meta.description {
					field_node.add_label(doc.clone())
				}
			}

			let field = field_node.as_layout_field_mut().unwrap();

			match Name::new(prop) {
				Ok(name) => field.set_name(name, None)?,
				Err(_) => return Err(Error::InvalidPropertyName(prop.to_string())),
			}

			field.set_layout(layout_id, None)?;

			fields.push(Caused::new(field_id.into_term(), None));
		}
	}

	let fields_id = context.create_list(vocabulary, fields)?;
	Ok(treeldr_build::layout::Description::Struct(fields_id))
}

fn format_layout<F>(format: schema::Format) -> Result<treeldr::layout::Primitive, Error<F>> {
	use treeldr::layout::Primitive;
	let layout = match format {
		schema::Format::DateTime => Primitive::DateTime,
		schema::Format::Date => Primitive::Date,
		schema::Format::Time => Primitive::Time,
		schema::Format::Duration => todo!(),
		schema::Format::Email => todo!(),
		schema::Format::IdnEmail => todo!(),
		schema::Format::Hostname => todo!(),
		schema::Format::IdnHostname => todo!(),
		schema::Format::Ipv4 => todo!(),
		schema::Format::Ipv6 => todo!(),
		schema::Format::Uri => Primitive::Uri,
		schema::Format::UriReference => todo!(),
		schema::Format::Iri => todo!(),
		schema::Format::IriReference => todo!(),
		schema::Format::Uuid => todo!(),
		schema::Format::UriTemplate => todo!(),
		schema::Format::JsonPointer => todo!(),
		schema::Format::RelativeJsonPointer => todo!(),
		schema::Format::Regex => todo!(),
	};

	Ok(layout)
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
