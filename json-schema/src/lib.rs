use treeldr::{layout, vocab::Display, Ref, Vocabulary};

mod command;
pub mod embedding;

pub use command::Command;
pub use embedding::Embedding;

pub enum Error<F> {
	NoLayoutName(Ref<layout::Definition<F>>),
	InfiniteSchema(Ref<layout::Definition<F>>),
	Serialization(serde_json::Error),
}

/// Generate a JSON Schema from a TreeLDR model.
pub fn generate<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<(), Error<F>> {
	// Check there are no cycles induced by the embedded layouts.
	let strongly_connected_layouts = treeldr::layout::StronglyConnectedLayouts::with_filter(
		model.layouts(),
		|_, sub_layout_ref| embedding.get(sub_layout_ref).is_direct(),
	);
	for (layout_ref, _) in model.layouts().iter() {
		if strongly_connected_layouts
			.is_recursive_with_filter(layout_ref, |sub_layout_ref| {
				embedding.get(sub_layout_ref).is_direct()
			})
			.unwrap_or(false)
		{
			return Err(Error::InfiniteSchema(layout_ref));
		}
	}

	let layout = model.layouts().get(layout_ref).unwrap();
	let name = layout.name().ok_or(Error::NoLayoutName(layout_ref))?;

	let mut json_schema = generate_layout(vocabulary, model, embedding, type_property, layout_ref)?;

	if let Some(json_schema) = json_schema.as_object_mut() {
		json_schema.insert(
			"$schema".into(),
			"https://json-schema.org/draft/2020-12/schema".into(),
		);

		let title = match layout.preferred_label(model) {
			Some(label) => label.to_string(),
			None => name.to_pascal_case(),
		};
		json_schema.insert("title".into(), title.into());

		// Generate the `$defs` section.
		let mut defs = serde_json::Map::new();
		for layout_ref in embedding.indirect_layouts() {
			let name = model
				.layouts()
				.get(layout_ref)
				.unwrap()
				.name()
				.ok_or(Error::NoLayoutName(layout_ref))?
				.to_string();

			let json_schema =
				generate_layout(vocabulary, model, embedding, type_property, layout_ref)?;

			defs.insert(name, json_schema);
		}
		if !defs.is_empty() {
			json_schema.insert("$defs".into(), defs.into());
		}
	}

	println!(
		"{}",
		serde_json::to_string_pretty(&json_schema).map_err(Error::Serialization)?
	);

	Ok(())
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
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<serde_json::Value, Error<F>> {
	let layout = model.layouts().get(layout_ref).unwrap();
	let mut schema = generate_layout_schema(vocabulary, model, embedding, type_property, layout)?;

	if let Some(schema) = schema.as_object_mut() {
		schema.insert(
			"$id".into(),
			layout.id().display(vocabulary).to_string().into(),
		);

		if let Some(description) = layout.preferred_documentation(model).short_description() {
			schema.insert(
				"description".into(),
				remove_newlines(description.trim()).into(),
			);
		}
	}

	Ok(schema)
}

fn generate_layout_schema<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	layout: &layout::Definition<F>,
) -> Result<serde_json::Value, Error<F>> {
	use treeldr::layout::Description;
	match layout.description() {
		Description::Never(_) => Ok(serde_json::Value::Bool(false)),
		Description::Reference(_, _) => {
			let mut json = serde_json::Map::new();
			json.insert("type".into(), "string".into());
			Ok(json.into())
		}
		Description::Struct(s) => generate_struct(vocabulary, model, embedding, type_property, s),
		Description::Enum(enm) => {
			generate_enum_type(vocabulary, model, embedding, type_property, enm)
		}
		Description::Literal(lit) => Ok(generate_literal_type(lit)),
		Description::Native(n, _) => Ok(generate_native_type(*n)),
		Description::Set(s) => {
			generate_set_type(vocabulary, model, embedding, type_property, s.item_layout())
		}
		Description::Array(a) => {
			generate_list_type(vocabulary, model, embedding, type_property, a.item_layout())
		}
		Description::Alias(_, alias_ref) => {
			let mut json = serde_json::Map::new();
			let alias = model.layouts().get(*alias_ref).unwrap();
			json.insert(
				"$ref".into(),
				alias.id().display(vocabulary).to_string().into(),
			);
			Ok(json.into())
		}
	}
}

fn generate_struct<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	s: &treeldr::layout::Struct<F>,
) -> Result<serde_json::Value, Error<F>> {
	let mut json = serde_json::Map::new();
	let mut properties = serde_json::Map::new();
	let mut required_properties = Vec::new();

	if let Some(name) = type_property {
		let mut type_schema = serde_json::Map::new();

		type_schema.insert("type".into(), "string".into());
		type_schema.insert("pattern".into(), s.name().to_pascal_case().into());

		properties.insert(name.into(), type_schema.into());
		required_properties.push(name.into());
	}

	for field in s.fields() {
		let field_layout_ref = field.layout();

		let mut layout_schema = embed_layout(
			vocabulary,
			model,
			embedding,
			type_property,
			field_layout_ref,
		)?;

		if let Some(obj) = layout_schema.as_object_mut() {
			if let Some(description) = field.preferred_label(model) {
				obj.insert(
					"description".into(),
					remove_newlines(description.trim()).into(),
				);
			}
		}

		properties.insert(field.name().to_camel_case(), layout_schema);

		if field.is_required() {
			required_properties.push(serde_json::Value::from(field.name().to_camel_case()));
		}
	}

	json.insert("type".into(), "object".into());
	json.insert("properties".into(), properties.into());

	if !required_properties.is_empty() {
		json.insert("required".into(), required_properties.into());
	}

	Ok(json.into())
}

fn embed_layout<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<serde_json::Value, Error<F>> {
	match embedding.get(layout_ref) {
		Embedding::Reference => {
			generate_layout_ref(vocabulary, model, embedding, type_property, layout_ref)
		}
		Embedding::Indirect => {
			let mut json = serde_json::Map::new();
			generate_layout_defs_ref(&mut json, model, layout_ref)?;
			Ok(json.into())
		}
		Embedding::Direct => {
			generate_layout(vocabulary, model, embedding, type_property, layout_ref)
		}
	}
}

fn generate_layout_defs_ref<F>(
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<(), Error<F>> {
	json.insert(
		"$ref".into(),
		format!(
			"#/$defs/{}",
			model
				.layouts()
				.get(layout_ref)
				.unwrap()
				.name()
				.ok_or(Error::NoLayoutName(layout_ref))?
		)
		.into(),
	);
	Ok(())
}

fn generate_layout_ref<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<serde_json::Value, Error<F>> {
	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Never(_) => Ok(serde_json::Value::Bool(false)),
		Description::Reference(_, _) => {
			let mut json = serde_json::Map::new();
			json.insert("type".into(), "string".into());
			Ok(json.into())
		}
		Description::Enum(enm) => {
			generate_enum_type(vocabulary, model, embedding, type_property, enm)
		}
		Description::Literal(lit) => Ok(generate_literal_type(lit)),
		Description::Native(n, _) => Ok(generate_native_type(*n)),
		Description::Set(s) => {
			generate_set_type(vocabulary, model, embedding, type_property, s.item_layout())
		}
		Description::Array(a) => {
			generate_list_type(vocabulary, model, embedding, type_property, a.item_layout())
		}
		Description::Struct(_) | Description::Alias(_, _) => {
			let mut json = serde_json::Map::new();
			let layout = model.layouts().get(layout_ref).unwrap();
			json.insert(
				"$ref".into(),
				layout.id().display(vocabulary).to_string().into(),
			);
			Ok(json.into())
		}
	}
}

fn generate_set_type<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	item_layout_ref: Ref<layout::Definition<F>>,
) -> Result<serde_json::Value, Error<F>> {
	let mut def = serde_json::Map::new();
	let item_schema =
		generate_layout_ref(vocabulary, model, embedding, type_property, item_layout_ref)?;
	def.insert("type".into(), "array".into());
	def.insert("items".into(), item_schema);
	def.insert("uniqueItems".into(), true.into());
	Ok(def.into())
}

fn generate_list_type<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	item_layout_ref: Ref<layout::Definition<F>>,
) -> Result<serde_json::Value, Error<F>> {
	let mut def = serde_json::Map::new();
	let item_schema =
		generate_layout_ref(vocabulary, model, embedding, type_property, item_layout_ref)?;
	def.insert("type".into(), "array".into());
	def.insert("items".into(), item_schema);
	Ok(def.into())
}

fn generate_enum_type<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	enm: &layout::Enum<F>,
) -> Result<serde_json::Value, Error<F>> {
	let mut def = serde_json::Map::new();
	let mut variants = Vec::with_capacity(enm.variants().len());
	for variant in enm.variants() {
		let layout_ref = variant.layout().unwrap();
		let variant_json = embed_layout(vocabulary, model, embedding, type_property, layout_ref)?;
		variants.push(variant_json)
	}

	def.insert("oneOf".into(), variants.into());

	Ok(def.into())
}

fn generate_literal_type<F>(lit: &layout::Literal<F>) -> serde_json::Value {
	let mut def = serde_json::Map::new();

	def.insert("type".into(), "string".into());
	match lit.regexp().as_singleton() {
		Some(singleton) => {
			def.insert("const".into(), singleton.into());
		}
		None => {
			// TODO: convert to ECMA-262 regular expression?
			def.insert("pattern".into(), lit.regexp().to_string().into());
		}
	}

	def.into()
}

fn generate_native_type(n: treeldr::layout::Native) -> serde_json::Value {
	use treeldr::layout::Native;
	let mut def = serde_json::Map::new();

	match n {
		Native::Boolean => {
			def.insert("type".into(), "bool".into());
		}
		Native::Integer => {
			def.insert("type".into(), "integer".into());
		}
		Native::PositiveInteger => {
			def.insert("type".into(), "integer".into());
			def.insert("minimum".into(), 0.into());
		}
		Native::Float => {
			def.insert("type".into(), "number".into());
		}
		Native::Double => {
			def.insert("type".into(), "number".into());
		}
		Native::String => {
			def.insert("type".into(), "string".into());
		}
		Native::Time => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "time".into());
		}
		Native::Date => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "date".into());
		}
		Native::DateTime => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "date-time".into());
		}
		Native::Iri => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "iri".into());
		}
		Native::Uri => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "uri".into());
		}
		Native::Url => {
			def.insert("type".into(), "string".into());
			def.insert("format".into(), "uri".into());
		}
	}

	def.into()
}
