use contextual::WithContext;
use locspan::Meta;
use rdf_types::Vocabulary;
use treeldr::{layout, BlankIdIndex, IriIndex, MetaOption, Name, TId};

mod command;
pub mod embedding;
pub mod import;
pub mod schema;

pub use command::Command;
pub use embedding::Embedding;
pub use import::import_schema;
pub use schema::Schema;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("missing layout name")]
	NoLayoutName(TId<treeldr::Layout>),

	#[error("infinite schema")]
	InfiniteSchema(TId<treeldr::Layout>),
}

/// Generate a JSON Schema from a TreeLDR model.
pub fn generate<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	layout_ref: TId<treeldr::Layout>,
) -> Result<json_syntax::Value, Error> {
	// Check there are no cycles induced by the embedded layouts.
	let strongly_connected_layouts =
		treeldr::layout::StronglyConnectedLayouts::with_filter(model, |_, sub_layout_ref| {
			embedding.get(sub_layout_ref).is_direct()
		});
	for (layout_ref, _) in model.layouts() {
		if strongly_connected_layouts
			.is_recursive_with_filter(model, layout_ref, |sub_layout_ref| {
				embedding.get(sub_layout_ref).is_direct()
			})
			.unwrap_or(false)
		{
			return Err(Error::InfiniteSchema(layout_ref));
		}
	}

	let layout = model.get(layout_ref).unwrap();
	let name = layout
		.as_component()
		.name()
		.ok_or(Error::NoLayoutName(layout_ref))?;

	let mut json_schema = generate_layout(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		layout_ref,
	)?;

	if let Some(json_schema) = json_schema.as_object_mut() {
		json_schema.insert(
			Meta("$schema".into(), ()),
			Meta("https://json-schema.org/draft/2020-12/schema".into(), ()),
		);

		let title = match layout.preferred_label() {
			Some(label) => label.to_string(),
			None => name.to_pascal_case(),
		};
		json_schema.insert(Meta("title".into(), ()), Meta(title.into(), ()));

		// Generate the `$defs` section.
		let mut defs = json_syntax::Object::new();
		for layout_ref in embedding.indirect_layouts() {
			let name = model
				.get(layout_ref)
				.unwrap()
				.as_component()
				.name()
				.ok_or(Error::NoLayoutName(layout_ref))?
				.to_string();

			let json_schema = generate_layout(
				vocabulary,
				model,
				embedding,
				type_property,
				None,
				layout_ref,
			)?;

			defs.insert(Meta(name.into(), ()), Meta(json_schema, ()));
		}
		if !defs.is_empty() {
			json_schema.insert(Meta("$defs".into(), ()), Meta(defs.into(), ()));
		}
	}

	Ok(json_schema)
}

fn remove_newlines(s: &str) -> String {
	let mut result = String::new();

	for (i, line) in s.lines().enumerate() {
		if i > 0 {
			result.push(' ');
		}

		result.push_str(line);
	}

	result
}

fn generate_layout<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	required: Option<&mut bool>,
	layout_ref: TId<treeldr::Layout>,
) -> Result<json_syntax::Value, Error> {
	let layout = model.get(layout_ref).unwrap();
	let mut schema = generate_layout_schema(
		vocabulary,
		model,
		embedding,
		type_property,
		required,
		layout,
	)?;

	if let Some(schema) = schema.as_object_mut() {
		schema.insert(
			Meta("$id".into(), ()),
			Meta(layout.id().with(vocabulary).to_string().into(), ()),
		);

		if let Some(description) = layout.comment().short_description() {
			schema.insert(
				Meta("description".into(), ()),
				Meta(remove_newlines(description.trim()).into(), ()),
			);
		}
	}

	Ok(schema)
}

fn generate_layout_schema<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	mut required: Option<&mut bool>,
	layout: treeldr::Ref<treeldr::Layout, F>,
) -> Result<json_syntax::Value, Error> {
	if let Some(required) = required.as_mut() {
		**required = layout.as_layout().description().is_required()
	}

	use treeldr::layout::Description;
	match layout.as_layout().description() {
		Description::Never => Ok(json_syntax::Value::Boolean(false)),
		Description::Primitive(n) => Ok(generate_primitive_type(*n)),
		Description::Derived(d) => Ok(generate_derived_type(d.value())),
		Description::Reference(_) => {
			let mut json = json_syntax::Object::new();
			json.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			Ok(json.into())
		}
		Description::Struct(s) => {
			let name = layout.as_component().name().expect("missing struct name");
			generate_struct(vocabulary, model, embedding, type_property, name, s)
		}
		Description::Enum(enm) => {
			generate_enum_type(vocabulary, model, embedding, type_property, enm)
		}
		Description::Required(r) => {
			let item_layout = model.get(**r.item_layout()).unwrap();
			generate_layout_schema(
				vocabulary,
				model,
				embedding,
				type_property,
				None,
				item_layout,
			)
		}
		Description::Option(o) => {
			if required.is_some() {
				let item_layout = model.get(**o.item_layout()).unwrap();
				generate_layout_schema(
					vocabulary,
					model,
					embedding,
					type_property,
					None,
					item_layout,
				)
			} else {
				generate_option_type(
					vocabulary,
					model,
					embedding,
					type_property,
					**o.item_layout(),
				)
			}
		}
		Description::Set(s) => generate_set_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**s.item_layout(),
			s.restrictions(),
		),
		Description::Map(s) => generate_map_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**s.key_layout(),
			**s.value_layout(),
		),
		Description::OneOrMany(s) => generate_one_or_many_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**s.item_layout(),
			s.restrictions(),
		),
		Description::Array(a) => generate_list_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**a.item_layout(),
			a.restrictions(),
		),
		Description::Alias(alias_ref) => {
			let mut json = json_syntax::Object::new();
			let alias = model.get(*alias_ref.value()).unwrap();
			json.insert(
				Meta("$ref".into(), ()),
				Meta(alias.id().with(vocabulary).to_string().into(), ()),
			);
			Ok(json.into())
		}
	}
}

fn generate_struct<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	name: &Name,
	s: &treeldr::layout::Struct<F>,
) -> Result<json_syntax::Value, Error> {
	let mut json = json_syntax::Object::new();
	let mut properties = json_syntax::Object::new();
	let mut required_properties = Vec::new();

	if let Some(type_prop) = type_property {
		let mut type_schema = json_syntax::Object::new();

		type_schema.insert(Meta("type".into(), ()), Meta("string".into(), ()));
		type_schema.insert(
			Meta("pattern".into(), ()),
			Meta(name.to_pascal_case().into(), ()),
		);

		properties.insert(Meta(type_prop.into(), ()), Meta(type_schema.into(), ()));
		required_properties.push(Meta(type_prop.into(), ()));
	}

	for field_id in s.fields() {
		let field = model.get(**field_id).unwrap();
		let field_layout_ref = field.as_formatted().format().expect("missing field layout");

		let mut required = true;
		let mut layout_schema = embed_layout(
			vocabulary,
			model,
			embedding,
			type_property,
			Some(&mut required),
			field_layout_ref,
		)?;

		if let Some(obj) = layout_schema.as_object_mut() {
			if let Some(description) = field.preferred_label() {
				obj.insert(
					Meta("description".into(), ()),
					Meta(
						remove_newlines(description.lexical_form().trim()).into(),
						(),
					),
				);
			}
		}

		properties.insert(
			Meta(field.name().unwrap().to_camel_case().into(), ()),
			Meta(layout_schema, ()),
		);

		if required {
			required_properties.push(Meta(
				json_syntax::Value::from(field.name().unwrap().to_camel_case()),
				(),
			));
		}
	}

	json.insert(Meta("type".into(), ()), Meta("object".into(), ()));

	if !properties.is_empty() {
		json.insert(Meta("properties".into(), ()), Meta(properties.into(), ()));
	}

	if !required_properties.is_empty() {
		json.insert(
			Meta("required".into(), ()),
			Meta(required_properties.into(), ()),
		);
	}

	Ok(json.into())
}

fn embed_layout<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	required: Option<&mut bool>,
	layout_ref: TId<treeldr::Layout>,
) -> Result<json_syntax::Value, Error> {
	match embedding.get(layout_ref) {
		Embedding::Reference => generate_layout_ref(
			vocabulary,
			model,
			embedding,
			type_property,
			required,
			layout_ref,
		),
		Embedding::Indirect => {
			let mut json = json_syntax::Object::new();
			generate_layout_defs_ref(&mut json, model, layout_ref)?;
			Ok(json.into())
		}
		Embedding::Direct => generate_layout(
			vocabulary,
			model,
			embedding,
			type_property,
			required,
			layout_ref,
		),
	}
}

fn generate_layout_defs_ref<F>(
	json: &mut json_syntax::Object,
	model: &treeldr::MutableModel<F>,
	layout_ref: TId<treeldr::Layout>,
) -> Result<(), Error> {
	json.insert(
		Meta("$ref".into(), ()),
		Meta(
			format!(
				"#/$defs/{}",
				model
					.get(layout_ref)
					.unwrap()
					.as_component()
					.name()
					.ok_or(Error::NoLayoutName(layout_ref))?
			)
			.into(),
			(),
		),
	);
	Ok(())
}

fn generate_layout_ref<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	mut required: Option<&mut bool>,
	layout_ref: TId<treeldr::Layout>,
) -> Result<json_syntax::Value, Error> {
	let layout = model.get(layout_ref).unwrap();

	if let Some(required) = required.as_mut() {
		**required = layout.as_layout().description().is_required()
	}

	use treeldr::layout::Description;
	match layout.as_layout().description() {
		Description::Never => Ok(json_syntax::Value::Boolean(false)),
		Description::Reference(_) => {
			let mut json = json_syntax::Object::new();
			json.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			Ok(json.into())
		}
		Description::Enum(enm) => {
			generate_enum_type(vocabulary, model, embedding, type_property, enm)
		}
		Description::Primitive(n) => Ok(generate_primitive_type(*n)),
		Description::Derived(d) => Ok(generate_derived_type(d.value())),
		Description::Required(r) => generate_layout_ref(
			vocabulary,
			model,
			embedding,
			type_property,
			None,
			**r.item_layout(),
		),
		Description::Option(o) => {
			if required.is_some() {
				generate_layout_ref(
					vocabulary,
					model,
					embedding,
					type_property,
					None,
					**o.item_layout(),
				)
			} else {
				generate_option_type(
					vocabulary,
					model,
					embedding,
					type_property,
					**o.item_layout(),
				)
			}
		}
		Description::Set(s) => generate_set_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**s.item_layout(),
			s.restrictions(),
		),
		Description::Map(m) => generate_map_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**m.key_layout(),
			**m.value_layout(),
		),
		Description::OneOrMany(s) => generate_one_or_many_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**s.item_layout(),
			s.restrictions(),
		),
		Description::Array(a) => generate_list_type(
			vocabulary,
			model,
			embedding,
			type_property,
			**a.item_layout(),
			a.restrictions(),
		),
		Description::Struct(_) | Description::Alias(_) => {
			let mut json = json_syntax::Object::new();
			let layout = model.get(layout_ref).unwrap();
			json.insert(
				Meta("$ref".into(), ()),
				Meta(layout.id().with(vocabulary).to_string().into(), ()),
			);
			Ok(json.into())
		}
	}
}

fn generate_option_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	item_layout_ref: TId<treeldr::Layout>,
) -> Result<json_syntax::Value, Error> {
	let mut def = json_syntax::Object::new();

	let mut null_schema = json_syntax::Object::new();
	null_schema.insert(Meta("type".into(), ()), Meta("null".into(), ()));

	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;

	def.insert(
		Meta("anyOf".into(), ()),
		Meta(
			vec![Meta(null_schema.into(), ()), Meta(item_schema, ())].into(),
			(),
		),
	);
	Ok(def.into())
}

fn generate_set_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	item_layout_ref: TId<treeldr::Layout>,
	restrictions: &MetaOption<treeldr::layout::ContainerRestrictions<F>, F>,
) -> Result<json_syntax::Value, Error> {
	let mut def = json_syntax::Object::new();
	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;
	def.insert(Meta("type".into(), ()), Meta("array".into(), ()));
	def.insert(Meta("items".into(), ()), Meta(item_schema, ()));
	def.insert(Meta("uniqueItems".into(), ()), Meta(true.into(), ()));

	if let Some(restrictions) = restrictions.as_ref() {
		if !restrictions.cardinal().min().is_zero() {
			let m: u64 = restrictions
				.cardinal()
				.min()
				.try_into()
				.expect("minimum is too large");
			def.insert(Meta("minItems".into(), ()), Meta(m.into(), ()));
		}

		if let Some(m) = restrictions.cardinal().max() {
			let m: u64 = m.clone().try_into().expect("maximum is too large");
			def.insert(Meta("maxItems".into(), ()), Meta(m.into(), ()));
		}
	}

	Ok(def.into())
}

fn generate_map_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	key_layout_ref: TId<treeldr::Layout>,
	value_layout_ref: TId<treeldr::Layout>,
) -> Result<serde_json::Value, Error> {
	let mut def = serde_json::Map::new();

	let key_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		key_layout_ref,
	)?;

	let value_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		value_layout_ref,
	)?;

	def.insert("type".into(), "object".into());
	def.insert("propertyNames".into(), key_schema);
	def.insert("additionalProperties".into(), value_schema);

	Ok(def.into())
}

fn generate_one_or_many_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	item_layout_ref: TId<treeldr::Layout>,
	restrictions: &MetaOption<treeldr::layout::ContainerRestrictions<F>, F>,
) -> Result<json_syntax::Value, Error> {
	let mut def = json_syntax::Object::new();

	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;

	def.insert(
		Meta("oneOf".into(), ()),
		Meta(
			vec![
				Meta(item_schema, ()),
				Meta(
					generate_set_type(
						vocabulary,
						model,
						embedding,
						type_property,
						item_layout_ref,
						restrictions,
					)?,
					(),
				),
			]
			.into(),
			(),
		),
	);

	Ok(def.into())
}

fn generate_list_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	item_layout_ref: TId<treeldr::Layout>,
	restrictions: &MetaOption<treeldr::layout::ContainerRestrictions<F>, F>,
) -> Result<json_syntax::Value, Error> {
	let mut def = json_syntax::Object::new();
	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;
	def.insert(Meta("type".into(), ()), Meta("array".into(), ()));
	def.insert(Meta("items".into(), ()), Meta(item_schema, ()));

	if let Some(restrictions) = restrictions.as_ref() {
		if !restrictions.cardinal().min().is_zero() {
			let m: u64 = restrictions
				.cardinal()
				.min()
				.try_into()
				.expect("minimum is too large");
			def.insert(Meta("minItems".into(), ()), Meta(m.into(), ()));
		}

		if let Some(m) = restrictions.cardinal().max() {
			let m: u64 = m.clone().try_into().expect("maximum is too large");
			def.insert(Meta("maxItems".into(), ()), Meta(m.into(), ()));
		}
	}

	Ok(def.into())
}

fn generate_enum_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	enm: &layout::Enum<F>,
) -> Result<json_syntax::Value, Error> {
	let mut def = json_syntax::Object::new();
	let mut variants = Vec::with_capacity(enm.variants().len());
	for variant_id in enm.variants() {
		let variant = model.get(**variant_id).unwrap();
		let layout_ref = variant
			.as_formatted()
			.format()
			.expect("missing variant layout");
		let variant_json = embed_layout(
			vocabulary,
			model,
			embedding,
			type_property,
			None,
			layout_ref,
		)?;
		variants.push(Meta(variant_json, ()))
	}

	def.insert(Meta("oneOf".into(), ()), Meta(variants.into(), ()));

	Ok(def.into())
}

fn generate_primitive_type(p: treeldr::layout::Primitive) -> json_syntax::Value {
	use treeldr::layout::Primitive;
	let mut def = json_syntax::Object::new();

	match p {
		Primitive::Boolean => {
			def.insert(Meta("type".into(), ()), Meta("bool".into(), ()));
		}
		Primitive::Integer => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
		}
		Primitive::NonNegativeInteger => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(0.into(), ()));
		}
		Primitive::NonPositiveInteger => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(0.into(), ()));
		}
		Primitive::PositiveInteger => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(1.into(), ()));
		}
		Primitive::NegativeInteger => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta((-1).into(), ()));
		}
		Primitive::I64 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i64::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i64::MAX.into(), ()));
		}
		Primitive::I32 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i32::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i32::MAX.into(), ()));
		}
		Primitive::I16 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i16::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i16::MAX.into(), ()));
		}
		Primitive::I8 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i8::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i8::MAX.into(), ()));
		}
		Primitive::U64 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u64::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u64::MAX.into(), ()));
		}
		Primitive::U32 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u32::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u32::MAX.into(), ()));
		}
		Primitive::U16 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u16::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u16::MAX.into(), ()));
		}
		Primitive::U8 => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u8::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u8::MAX.into(), ()));
		}
		Primitive::Float => {
			def.insert(Meta("type".into(), ()), Meta("number".into(), ()));
		}
		Primitive::Double => {
			def.insert(Meta("type".into(), ()), Meta("number".into(), ()));
		}
		Primitive::Base64Bytes => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
		}
		Primitive::HexBytes => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
		}
		Primitive::String => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
		}
		Primitive::Time => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("time".into(), ()));
		}
		Primitive::Date => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("date".into(), ()));
		}
		Primitive::DateTime => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("date-time".into(), ()));
		}
		Primitive::Iri => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("iri".into(), ()));
		}
		Primitive::Uri => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("uri".into(), ()));
		}
		Primitive::Url => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("uri".into(), ()));
		}
	}

	def.into()
}

fn generate_derived_type<M>(n: &treeldr::layout::primitive::Restricted<M>) -> json_syntax::Value {
	use treeldr::layout::RestrictedPrimitive;
	let mut def = json_syntax::Object::new();

	match n {
		RestrictedPrimitive::Boolean(_) => {
			def.insert(Meta("type".into(), ()), Meta("bool".into(), ()));
		}
		RestrictedPrimitive::Integer(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
		}
		RestrictedPrimitive::NonNegativeInteger(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(0.into(), ()));
		}
		RestrictedPrimitive::NonPositiveInteger(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(0.into(), ()));
		}
		RestrictedPrimitive::PositiveInteger(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(1.into(), ()));
		}
		RestrictedPrimitive::NegativeInteger(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta((-1).into(), ()));
		}
		RestrictedPrimitive::I64(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i64::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i64::MAX.into(), ()));
		}
		RestrictedPrimitive::I32(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i32::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i32::MAX.into(), ()));
		}
		RestrictedPrimitive::I16(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i16::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i16::MAX.into(), ()));
		}
		RestrictedPrimitive::I8(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(i8::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(i8::MAX.into(), ()));
		}
		RestrictedPrimitive::U64(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u64::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u64::MAX.into(), ()));
		}
		RestrictedPrimitive::U32(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u32::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u32::MAX.into(), ()));
		}
		RestrictedPrimitive::U16(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u16::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u16::MAX.into(), ()));
		}
		RestrictedPrimitive::U8(_) => {
			def.insert(Meta("type".into(), ()), Meta("integer".into(), ()));
			def.insert(Meta("minimum".into(), ()), Meta(u8::MIN.into(), ()));
			def.insert(Meta("maximum".into(), ()), Meta(u8::MAX.into(), ()));
		}
		RestrictedPrimitive::Float(_) => {
			def.insert(Meta("type".into(), ()), Meta("number".into(), ()));
		}
		RestrictedPrimitive::Double(_) => {
			def.insert(Meta("type".into(), ()), Meta("number".into(), ()));
		}
		RestrictedPrimitive::Base64Bytes(restrictions) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			if let Some(r) = restrictions.as_ref() {
				if let Some(pattern) = r.pattern() {
					match pattern.as_singleton() {
						Some(singleton) => {
							def.insert(Meta("const".into(), ()), Meta(singleton.into(), ()));
						}
						None => {
							def.insert(
								Meta("pattern".into(), ()),
								Meta(pattern.to_string().into(), ()),
							);
						}
					}
				}
			}
		}
		RestrictedPrimitive::HexBytes(restrictions) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			if let Some(r) = restrictions.as_ref() {
				if let Some(pattern) = r.pattern() {
					match pattern.as_singleton() {
						Some(singleton) => {
							def.insert(Meta("const".into(), ()), Meta(singleton.into(), ()));
						}
						None => {
							def.insert(
								Meta("pattern".into(), ()),
								Meta(pattern.to_string().into(), ()),
							);
						}
					}
				}
			}
		}
		RestrictedPrimitive::String(restrictions) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			if let Some(r) = restrictions.as_ref() {
				if let Some(pattern) = r.pattern() {
					match pattern.as_singleton() {
						Some(singleton) => {
							def.insert(Meta("const".into(), ()), Meta(singleton.into(), ()));
						}
						None => {
							def.insert(
								Meta("pattern".into(), ()),
								Meta(pattern.to_string().into(), ()),
							);
						}
					}
				}
			}
		}
		RestrictedPrimitive::Time(_) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("time".into(), ()));
		}
		RestrictedPrimitive::Date(_) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("date".into(), ()));
		}
		RestrictedPrimitive::DateTime(_) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("date-time".into(), ()));
		}
		RestrictedPrimitive::Iri(_) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("iri".into(), ()));
		}
		RestrictedPrimitive::Uri(_) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("uri".into(), ()));
		}
		RestrictedPrimitive::Url(_) => {
			def.insert(Meta("type".into(), ()), Meta("string".into(), ()));
			def.insert(Meta("format".into(), ()), Meta("uri".into(), ()));
		}
	}

	def.into()
}
