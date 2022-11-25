//! JSON Schema import functions.
use crate::schema::{self, RegularSchema, Schema};
use iref::{Iri, IriBuf};
use locspan::{MaybeLocated, Meta, Span};
use rdf_types::{Generator, Quad, Vocabulary, VocabularyMut};
use treeldr::{metadata::Merge, vocab, BlankIdIndex, Id, IriIndex, Name};
use treeldr_build::{layout::Restrictions, Context};
use vocab::{LocQuad, Object, Term};

/// Import error.
#[derive(Debug)]
pub enum Error<M> {
	UnsupportedType,
	InvalidPropertyName(String),
	Build(treeldr_build::Error<M>),
}

impl<M> From<treeldr_build::Error<M>> for Error<M> {
	fn from(e: treeldr_build::Error<M>) -> Self {
		Self::Build(e)
	}
}

impl<M: MaybeLocated<Span = Span>> treeldr::reporting::DiagnoseWithVocabulary<M> for Error<M>
where
	M::File: Clone,
{
	fn message(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> String {
		match self {
			Self::UnsupportedType => "unsupported schema `type` value.".to_string(),
			Self::InvalidPropertyName(name) => format!("invalid property name `{}`", name),
			Self::Build(e) => e.message(vocabulary),
		}
	}
}

pub fn import_schema<
	M: Default + Clone + Merge,
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
>(
	schema: &Schema,
	base_iri: Option<Iri>,
	context: &mut Context<M>,
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
) -> Result<Id, Error<M>> {
	match schema {
		Schema::True => todo!(),
		Schema::False => {
			let id = generator.next(vocabulary);
			context.declare_layout(id, M::default());
			Ok(id)
		}
		Schema::Ref(r) => {
			let iri = r.target.resolved(base_iri.unwrap());
			let id = Id::Iri(vocabulary.insert(iri.as_iri()));
			Ok(id)
		}
		Schema::DynamicRef(_) => todo!(),
		Schema::Regular(schema) => {
			import_regular_schema(schema, true, base_iri, context, vocabulary, generator)
		}
	}
}

pub fn import_sub_schema<
	M: Default + Clone + Merge,
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
>(
	schema: &Schema,
	base_iri: Option<Iri>,
	context: &mut Context<M>,
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
) -> Result<Id, Error<M>> {
	match schema {
		Schema::True => todo!(),
		Schema::False => {
			let id = generator.next(vocabulary);
			context.declare_layout(id, M::default());
			Ok(id)
		}
		Schema::Ref(r) => {
			let iri = r.target.resolved(base_iri.unwrap());
			let id = Id::Iri(vocabulary.insert(iri.as_iri()));
			Ok(id)
		}
		Schema::DynamicRef(_) => todo!(),
		Schema::Regular(schema) => {
			import_regular_schema(schema, false, base_iri, context, vocabulary, generator)
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

	pub fn refine<M>(&mut self, other: Self) -> Result<(), Error<M>> {
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

pub fn import_regular_schema<
	M: Default + Clone + Merge,
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
>(
	schema: &RegularSchema,
	_top_level: bool,
	base_iri: Option<Iri>,
	context: &mut Context<M>,
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
) -> Result<Id, Error<M>> {
	if let Some(defs) = &schema.defs {
		for schema in defs.values() {
			import_sub_schema(schema, base_iri, context, vocabulary, generator)?;
		}
	}

	let base_iri = match &schema.id {
		Some(iri) => Some(iri.clone()),
		None => base_iri.map(IriBuf::from),
	};

	let (desc, restrictions) = import_layout_description(
		schema,
		base_iri.as_ref().map(Iri::from),
		context,
		vocabulary,
		generator,
	)?;

	if let treeldr_build::layout::Description::Primitive(primitive) = &desc {
		if schema.is_primitive()
			&& schema.id.is_none()
			&& schema.meta_schema.is_empty()
			&& schema.meta_data.title.is_none()
			&& schema.anchor.is_none()
			&& schema.dynamic_anchor.is_none()
		{
			return Ok(primitive.id());
		}
	}

	let (id, mut name) = match &schema.id {
		Some(iri) => {
			let id = Id::Iri(vocabulary.insert(iri.as_iri()));
			let iri_without_ext = strip_json_schema_extension(iri.as_iri());
			let name = Name::from_iri(iri_without_ext).ok().flatten();

			(id, name)
		}
		None => {
			let id = generator.next(vocabulary);
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
	context.declare_layout(id, M::default());

	let node = context.get_mut(id).unwrap();
	if let Some(title) = &schema.meta_data.title {
		// The title of a schema is translated in an rdfs:label.
		node.label_mut().insert(Meta(title.clone(), M::default()));
	}

	if let Some(description) = &schema.meta_data.description {
		// The title of a schema is translated in an rdfs:comment.
		node.comment_mut()
			.insert(Meta(description.clone(), M::default()));
	}

	if let Some(name) = name {
		node.as_component_mut()
			.name_mut()
			.insert(Meta(name, M::default()));
	}

	node.as_layout_mut()
		.description_mut()
		.insert(Meta(desc, M::default()));

	if !restrictions.is_empty() {
		let list_id = context.create_list_with(
			vocabulary,
			generator,
			restrictions.into_iter(),
			|Meta(restriction, meta), context, vocabulary, generator| {
				let id = generator.next(vocabulary);
				let node = context.declare_layout_restriction(id, meta.clone());
				node.as_layout_restriction_mut()
					.restriction_mut()
					.insert(Meta(restriction, meta.clone()));
				Meta(id.into_term(), meta)
			},
		);

		let node = context.get_mut(id).unwrap();
		node.as_layout_mut()
			.restrictions_mut()
			.insert(Meta(list_id, M::default()));
	}

	Ok(id)
}

fn strip_json_schema_extension(iri: Iri) -> Iri {
	match iri.into_str().strip_suffix(".schema.json") {
		Some(s) => match Iri::new(s) {
			Ok(result) => result,
			Err(_) => iri,
		},
		None => iri,
	}
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

fn import_layout_description<
	M: Default + Clone + Merge,
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
>(
	schema: &RegularSchema,
	base_iri: Option<Iri>,
	context: &mut Context<M>,
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
) -> Result<(treeldr_build::layout::Description, Restrictions<M>), Error<M>> {
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
			return Ok((
				treeldr_build::layout::Description::Primitive(layout),
				Restrictions::default(),
			));
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
						Ok((
							treeldr_build::layout::Description::Primitive(
								primitive_layout.unwrap(),
							),
							Restrictions::default(),
						))
					} else {
						use treeldr_build::layout::restriction::primitive::{Numeric, Restriction};

						let primitive = primitive_layout.unwrap();
						let mut restrictions = treeldr_build::layout::Restrictions::default();

						if let Some(min) = &schema.validation.numeric.minimum {
							restrictions.primitive.insert(Meta(
								Restriction::Numeric(Numeric::InclusiveMinimum(into_numeric(
									primitive, min,
								))),
								M::default(),
							))
						}

						if let Some(min) = &schema.validation.numeric.exclusive_minimum {
							restrictions.primitive.insert(Meta(
								Restriction::Numeric(Numeric::ExclusiveMinimum(into_numeric(
									primitive, min,
								))),
								M::default(),
							))
						}

						if let Some(max) = &schema.validation.numeric.maximum {
							restrictions.primitive.insert(Meta(
								Restriction::Numeric(Numeric::InclusiveMaximum(into_numeric(
									primitive, max,
								))),
								M::default(),
							))
						}

						if let Some(max) = &schema.validation.numeric.exclusive_maximum {
							restrictions.primitive.insert(Meta(
								Restriction::Numeric(Numeric::ExclusiveMaximum(into_numeric(
									primitive, max,
								))),
								M::default(),
							))
						}

						Ok((
							treeldr_build::layout::Description::Primitive(primitive),
							restrictions,
						))
					}
				}
				LayoutKind::String => {
					use treeldr_build::layout::restriction::primitive::{Restriction, String};

					let mut restrictions = treeldr_build::layout::Restrictions::default();

					if let Some(cnst) = &schema.validation.any.cnst {
						restrictions.primitive.insert(Meta(
							Restriction::String(String::Pattern(cnst.to_string().into())),
							M::default(),
						));
					}

					if let Some(pattern) = &schema.validation.string.pattern {
						restrictions.primitive.insert(Meta(
							Restriction::String(String::Pattern(pattern.to_string().into())),
							M::default(),
						));
					}

					// TODO: for now, most string constraints are ignored.
					Ok((
						treeldr_build::layout::Description::Primitive(
							treeldr::layout::Primitive::String,
						),
						restrictions,
					))
				}
				LayoutKind::ArrayOrSet | LayoutKind::Array | LayoutKind::Set => {
					import_array_schema(
						schema, false, array, &mut kind, base_iri, context, vocabulary, generator,
					)
				}
				LayoutKind::Struct => import_object_schema(
					schema, false, object, base_iri, context, vocabulary, generator,
				),
			}
		}
		_ => todo!(),
	}
}

#[allow(clippy::too_many_arguments)]
fn import_array_schema<
	M: Default + Clone + Merge,
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
>(
	schema: &RegularSchema,
	_top_level: bool,
	array: &schema::ArraySchema,
	kind: &mut LayoutKind,
	base_iri: Option<Iri>,
	context: &mut Context<M>,
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
) -> Result<(treeldr_build::layout::Description, Restrictions<M>), Error<M>> {
	let item_type = match &array.items {
		Some(items) => import_sub_schema(items, base_iri, context, vocabulary, generator)?,
		None => todo!(),
	};

	if matches!(schema.validation.array.unique_items, Some(true)) {
		kind.refine(LayoutKind::Set)?;
		Ok((
			treeldr_build::layout::Description::Set(item_type),
			Restrictions::default(),
		))
	} else {
		kind.refine(LayoutKind::Array)?;
		Ok((
			treeldr_build::layout::Description::Array(item_type),
			Restrictions::default(),
		))
	}
}

fn import_object_schema<
	M: Default + Clone + Merge,
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
>(
	schema: &RegularSchema,
	_top_level: bool,
	object: &schema::ObjectSchema,
	base_iri: Option<Iri>,
	context: &mut Context<M>,
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
) -> Result<(treeldr_build::layout::Description, Restrictions<M>), Error<M>> {
	let mut fields: Vec<Meta<Object<M>, M>> = Vec::new();

	if let Some(properties) = &object.properties {
		fields.reserve(properties.len());

		// First, we build each field.
		for (prop, prop_schema) in properties {
			let layout_item_id =
				import_sub_schema(prop_schema, base_iri, context, vocabulary, generator)?;

			let mut is_required = false;
			if let Some(required) = &schema.validation.object.required {
				if required.contains(prop) {
					is_required = true;
				}
			}

			let layout_id = generator.next(vocabulary);
			context.declare_layout(layout_id, M::default());
			let layout = context.get_mut(layout_id).unwrap().as_layout_mut();

			if is_required {
				layout.set_required(Meta(layout_item_id, M::default()));
			} else {
				layout.set_option(Meta(layout_item_id, M::default()));
			}

			let field_id = generator.next(vocabulary);
			context.declare_layout_field(field_id, M::default());
			let field_node = context.get_mut(field_id).unwrap();

			if let Some(meta) = &prop_schema.meta_data() {
				if let Some(doc) = &meta.description {
					field_node
						.label_mut()
						.insert(Meta(doc.clone(), M::default()))
				}
			}

			match Name::new(prop) {
				Ok(name) => field_node
					.as_component_mut()
					.name_mut()
					.insert(Meta(name, M::default())),
				Err(_) => return Err(Error::InvalidPropertyName(prop.to_string())),
			}

			field_node
				.as_formatted_mut()
				.format_mut()
				.insert(Meta(layout_id, M::default()));

			fields.push(Meta(field_id.into_term(), M::default()));
		}
	}

	let fields_id = context.create_list(vocabulary, generator, fields);
	Ok((
		treeldr_build::layout::Description::Struct(fields_id),
		Restrictions::default(),
	))
}

fn format_layout<M>(format: schema::Format) -> Result<treeldr::layout::Primitive, Error<M>> {
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

pub trait TryIntoRdfList<M, C, T> {
	fn try_into_rdf_list<E, K, V, G>(
		self,
		ctx: &mut C,
		vocab: &mut V,
		generator: &mut G,
		quads: &mut Vec<LocQuad<M>>,
		meta: M,
		f: K,
	) -> Result<Meta<Object<M>, M>, E>
	where
		K: FnMut(T, &mut C, &mut V, &mut G, &mut Vec<LocQuad<M>>) -> Result<Meta<Object<M>, M>, E>,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		G: Generator<V>;
}

impl<M: Clone + Merge, C, I: DoubleEndedIterator> TryIntoRdfList<M, C, I::Item> for I {
	fn try_into_rdf_list<E, K, V, G>(
		self,
		ctx: &mut C,
		vocab: &mut V,
		generator: &mut G,
		quads: &mut Vec<LocQuad<M>>,
		meta: M,
		mut f: K,
	) -> Result<Meta<Object<M>, M>, E>
	where
		K: FnMut(
			I::Item,
			&mut C,
			&mut V,
			&mut G,
			&mut Vec<LocQuad<M>>,
		) -> Result<Meta<Object<M>, M>, E>,
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
		G: Generator<V>,
	{
		use vocab::Rdf;
		let mut head = Meta(
			Object::Iri(IriIndex::Iri(Term::Rdf(Rdf::Nil))),
			meta.clone(),
		);
		for item in self.rev() {
			let item = f(item, ctx, vocab, generator, quads)?;
			let item_label = generator.next(vocab);
			let item_loc = item.metadata().clone();

			quads.push(Meta(
				Quad(
					Meta(item_label, item_loc.clone()),
					Meta(IriIndex::Iri(Term::Rdf(Rdf::Type)), item_loc.clone()),
					Meta(
						Object::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
						item_loc.clone(),
					),
					None,
				),
				item_loc.clone(),
			));

			quads.push(Meta(
				Quad(
					Meta(item_label, item_loc.clone()),
					Meta(IriIndex::Iri(Term::Rdf(Rdf::First)), item_loc.clone()),
					item,
					None,
				),
				item_loc.clone(),
			));

			quads.push(Meta(
				Quad(
					Meta(item_label, meta.clone()),
					Meta(IriIndex::Iri(Term::Rdf(Rdf::Rest)), meta.clone()),
					head,
					None,
				),
				item_loc.clone(),
			));

			head = Meta(item_label.into_term(), meta.clone());
		}

		Ok(head)
	}
}
