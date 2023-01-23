use contextual::WithContext;
use rdf_types::Vocabulary;
use treeldr::{layout, BlankIdIndex, IriIndex, Name, TId};

mod command;
pub mod embedding;
pub mod import;
pub mod schema;

pub use command::Command;
pub use embedding::Embedding;
pub use import::import_schema;
pub use schema::Schema;

#[derive(Debug)]
pub enum Error {
	NoLayoutName(TId<treeldr::Layout>),
	InfiniteSchema(TId<treeldr::Layout>),
	Serialization(serde_json::Error),
}

/// Generate a JSON Schema from a TreeLDR model.
pub fn generate<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	layout_ref: TId<treeldr::Layout>,
) -> Result<serde_json::Value, Error> {
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
			"$schema".into(),
			"https://json-schema.org/draft/2020-12/schema".into(),
		);

		let title = match layout.preferred_label() {
			Some(label) => label.to_string(),
			None => name.to_pascal_case(),
		};
		json_schema.insert("title".into(), title.into());

		// Generate the `$defs` section.
		let mut defs = serde_json::Map::new();
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

			defs.insert(name, json_schema);
		}
		if !defs.is_empty() {
			json_schema.insert("$defs".into(), defs.into());
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
) -> Result<serde_json::Value, Error> {
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
			"$id".into(),
			layout.id().with(vocabulary).to_string().into(),
		);

		if let Some(description) = layout.comment().short_description() {
			schema.insert(
				"description".into(),
				remove_newlines(description.trim()).into(),
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
) -> Result<serde_json::Value, Error> {
	if let Some(required) = required.as_mut() {
		**required = layout.as_layout().description().value().is_required()
	}

	use treeldr::layout::Description;
	match layout.as_layout().description().value() {
		Description::Never => Ok(serde_json::Value::Bool(false)),
		Description::Reference(_) => {
			let mut json = serde_json::Map::new();
			json.insert("type".into(), "string".into());
			Ok(json.into())
		}
		Description::Struct(s) => {
			let name = layout.as_component().name().expect("missing struct name");
			generate_struct(vocabulary, model, embedding, type_property, name, s)
		}
		Description::Enum(enm) => {
			generate_enum_type(vocabulary, model, embedding, type_property, enm)
		}
		Description::Primitive(n) => Ok(generate_primitive_type(n)),
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
			let mut json = serde_json::Map::new();
			let alias = model.get(*alias_ref).unwrap();
			json.insert(
				"$ref".into(),
				alias.id().with(vocabulary).to_string().into(),
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
) -> Result<serde_json::Value, Error> {
	let mut json = serde_json::Map::new();
	let mut properties = serde_json::Map::new();
	let mut required_properties = Vec::new();

	if let Some(type_prop) = type_property {
		let mut type_schema = serde_json::Map::new();

		type_schema.insert("type".into(), "string".into());
		type_schema.insert("pattern".into(), name.to_pascal_case().into());

		properties.insert(type_prop.into(), type_schema.into());
		required_properties.push(type_prop.into());
	}

	for field_id in s.fields() {
		let field = model.get(**field_id).unwrap();
		let field_layout_ref = **field
			.as_formatted()
			.format()
			.as_ref()
			.expect("missing field layout");

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
					"description".into(),
					remove_newlines(description.trim()).into(),
				);
			}
		}

		properties.insert(field.name().unwrap().to_camel_case(), layout_schema);

		if required {
			required_properties.push(serde_json::Value::from(
				field.name().unwrap().to_camel_case(),
			));
		}
	}

	json.insert("type".into(), "object".into());

	if !properties.is_empty() {
		json.insert("properties".into(), properties.into());
	}

	if !required_properties.is_empty() {
		json.insert("required".into(), required_properties.into());
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
) -> Result<serde_json::Value, Error> {
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
			let mut json = serde_json::Map::new();
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
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::MutableModel<F>,
	layout_ref: TId<treeldr::Layout>,
) -> Result<(), Error> {
	json.insert(
		"$ref".into(),
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
) -> Result<serde_json::Value, Error> {
	let layout = model.get(layout_ref).unwrap();

	if let Some(required) = required.as_mut() {
		**required = layout.as_layout().description().value().is_required()
	}

	use treeldr::layout::Description;
	match layout.as_layout().description().value() {
		Description::Never => Ok(serde_json::Value::Bool(false)),
		Description::Reference(_) => {
			let mut json = serde_json::Map::new();
			json.insert("type".into(), "string".into());
			Ok(json.into())
		}
		Description::Enum(enm) => {
			generate_enum_type(vocabulary, model, embedding, type_property, enm)
		}
		Description::Primitive(n) => Ok(generate_primitive_type(n)),
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
			let mut json = serde_json::Map::new();
			let layout = model.get(layout_ref).unwrap();
			json.insert(
				"$ref".into(),
				layout.id().with(vocabulary).to_string().into(),
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
) -> Result<serde_json::Value, Error> {
	let mut def = serde_json::Map::new();

	let mut null_schema = serde_json::Map::new();
	null_schema.insert("type".into(), "null".into());

	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;

	def.insert("anyOf".into(), vec![null_schema.into(), item_schema].into());
	Ok(def.into())
}

fn generate_set_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	item_layout_ref: TId<treeldr::Layout>,
	restrictions: &treeldr::layout::ContainerRestrictions<F>,
) -> Result<serde_json::Value, Error> {
	let mut def = serde_json::Map::new();
	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;
	def.insert("type".into(), "array".into());
	def.insert("items".into(), item_schema);
	def.insert("uniqueItems".into(), true.into());

	if !restrictions.cardinal().min().is_zero() {
		let m: u64 = restrictions
			.cardinal()
			.min()
			.try_into()
			.expect("minimum is too large");
		def.insert("minItems".into(), m.into());
	}

	if let Some(m) = restrictions.cardinal().max() {
		let m: u64 = m.clone().try_into().expect("maximum is too large");
		def.insert("maxItems".into(), m.into());
	}

	Ok(def.into())
}

fn generate_one_or_many_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	item_layout_ref: TId<treeldr::Layout>,
	restrictions: &treeldr::layout::ContainerRestrictions<F>,
) -> Result<serde_json::Value, Error> {
	let mut def = serde_json::Map::new();

	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;

	def.insert(
		"oneOf".into(),
		vec![
			item_schema,
			generate_set_type(
				vocabulary,
				model,
				embedding,
				type_property,
				item_layout_ref,
				restrictions,
			)?,
		]
		.into(),
	);

	Ok(def.into())
}

fn generate_list_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	item_layout_ref: TId<treeldr::Layout>,
	restrictions: &treeldr::layout::ContainerRestrictions<F>,
) -> Result<serde_json::Value, Error> {
	let mut def = serde_json::Map::new();
	let item_schema = generate_layout_ref(
		vocabulary,
		model,
		embedding,
		type_property,
		None,
		item_layout_ref,
	)?;
	def.insert("type".into(), "array".into());
	def.insert("items".into(), item_schema);

	if !restrictions.cardinal().min().is_zero() {
		let m: u64 = restrictions
			.cardinal()
			.min()
			.try_into()
			.expect("minimum is too large");
		def.insert("minItems".into(), m.into());
	}

	if let Some(m) = restrictions.cardinal().max() {
		let m: u64 = m.clone().try_into().expect("maximum is too large");
		def.insert("maxItems".into(), m.into());
	}

	Ok(def.into())
}

fn generate_enum_type<F>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	embedding: &embedding::Configuration,
	type_property: Option<&str>,
	enm: &layout::Enum<F>,
) -> Result<serde_json::Value, Error> {
	let mut def = serde_json::Map::new();
	let mut variants = Vec::with_capacity(enm.variants().len());
	for variant_id in enm.variants() {
		let variant = model.get(**variant_id).unwrap();
		let layout_ref = **variant
			.as_formatted()
			.format()
			.as_ref()
			.expect("missing variant layout");
		let variant_json = embed_layout(
			vocabulary,
			model,
			embedding,
			type_property,
			None,
			layout_ref,
		)?;
		variants.push(variant_json)
	}

	def.insert("oneOf".into(), variants.into());

	Ok(def.into())
}

fn generate_primitive_type<M>(n: &treeldr::layout::primitive::Restricted<M>) -> serde_json::Value {
	use treeldr::layout::RestrictedPrimitive;
	let mut def = serde_json::Map::new();

	match n {
		RestrictedPrimitive::Boolean => {
			def.insert("type".into(), "bool".into());
		}
		RestrictedPrimitive::Integer(_) => {
			def.insert("type".into(), "integer".into());
		}
		RestrictedPrimitive::UnsignedInteger(_) => {
			def.insert("type".into(), "integer".into());
			def.insert("minimum".into(), 0.into());
		}
		RestrictedPrimitive::Float(_) => {
			def.insert("type".into(), "number".into());
		}
		RestrictedPrimitive::Double(_) => {
			def.insert("type".into(), "number".into());
		}
		RestrictedPrimitive::String(s) => {
			def.insert("type".into(), "string".into());
			if let Some(pattern) = s.pattern() {
				match pattern.as_singleton() {
					Some(singleton) => {
						def.insert("const".into(), singleton.into());
					}
					None => {
						def.insert("pattern".into(), pattern.to_string().into());
					}
				}
			}
		}
		RestrictedPrimitive::Time => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "time".into());
		}
		RestrictedPrimitive::Date => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "date".into());
		}
		RestrictedPrimitive::DateTime => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "date-time".into());
		}
		RestrictedPrimitive::Iri => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "iri".into());
		}
		RestrictedPrimitive::Uri => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "uri".into());
		}
		RestrictedPrimitive::Url => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "uri".into());
		}
	}

	def.into()
}
