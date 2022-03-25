use treeldr::{layout, vocab::Display, Ref};

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

	let mut json_schema = serde_json::Map::new();
	json_schema.insert(
		"$schema".into(),
		"https://json-schema.org/draft/2020-12/schema".into(),
	);

	let title = match layout.preferred_label(model) {
		Some(label) => label,
		None => name,
	};
	json_schema.insert("title".into(), title.into());
	generate_layout(
		&mut json_schema,
		model,
		embedding,
		type_property,
		layout_ref,
	)?;

	// Generate the `$defs` section.
	let mut defs = serde_json::Map::new();
	for layout_ref in embedding.indirect_layouts() {
		let mut json_schema = serde_json::Map::new();
		let name = model
			.layouts()
			.get(layout_ref)
			.unwrap()
			.name()
			.ok_or(Error::NoLayoutName(layout_ref))?
			.to_string();
		generate_layout(
			&mut json_schema,
			model,
			embedding,
			type_property,
			layout_ref,
		)?;
		defs.insert(name, json_schema.into());
	}
	if !defs.is_empty() {
		json_schema.insert("$defs".into(), defs.into());
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
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<(), Error<F>> {
	let layout = model.layouts().get(layout_ref).unwrap();
	json.insert(
		"$id".into(),
		layout.id().display(model.vocabulary()).to_string().into(),
	);

	if let Some(description) = layout.preferred_documentation(model).short_description() {
		json.insert(
			"description".into(),
			remove_newlines(description.trim()).into(),
		);
	}

	use treeldr::layout::Description;
	match layout.description() {
		Description::Reference(_, _) => {
			json.insert("type".into(), "string".into());
			Ok(())
		}
		Description::Struct(s) => generate_struct(json, model, embedding, type_property, s),
		Description::Enum(_) => {
			todo!("json-schema enum layout")
		}
		Description::Sum(_) => {
			todo!("json-schema sum layout")
		}
		Description::Literal(_) => {
			todo!("json-schema literal layout")
		}
		Description::Native(n, _) => {
			generate_native_type(json, *n);
			Ok(())
		}
	}
}

fn generate_struct<F>(
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	type_property: Option<&str>,
	s: &treeldr::layout::Struct<F>,
) -> Result<(), Error<F>> {
	let mut properties = serde_json::Map::new();
	let mut required_properties = Vec::new();

	if let Some(name) = type_property {
		let mut type_schema = serde_json::Map::new();

		type_schema.insert("type".into(), "string".into());
		type_schema.insert("pattern".into(), s.name().into());

		properties.insert(name.into(), type_schema.into());
		required_properties.push(name.into());
	}

	for field in s.fields() {
		let field_layout_ref = field.layout();

		let mut layout_schema = serde_json::Map::new();

		match embedding.get(field_layout_ref) {
			Embedding::Reference => {
				generate_layout_ref(&mut layout_schema, model, field_layout_ref)?;
			}
			Embedding::Indirect => {
				generate_layout_defs_ref(&mut layout_schema, model, field_layout_ref)?;
			}
			Embedding::Direct => {
				generate_layout(
					&mut layout_schema,
					model,
					embedding,
					type_property,
					field_layout_ref,
				)?;
			}
		}

		let mut field_schema = if field.is_functional() {
			layout_schema
		} else {
			let mut field_schema = serde_json::Map::new();

			field_schema.insert("type".into(), "array".into());
			field_schema.insert("items".into(), layout_schema.into());

			if field.is_required() {
				field_schema.insert("minItems".into(), 1.into());
			}

			field_schema
		};

		if let Some(description) = field.preferred_label(model) {
			field_schema.insert(
				"description".into(),
				remove_newlines(description.trim()).into(),
			);
		}

		properties.insert(field.name().into(), field_schema.into());

		if field.is_required() {
			required_properties.push(serde_json::Value::from(field.name()));
		}
	}

	json.insert("type".into(), "object".into());
	json.insert("properties".into(), properties.into());

	if !required_properties.is_empty() {
		json.insert("required".into(), required_properties.into());
	}

	Ok(())
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
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<(), Error<F>> {
	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Reference(_, _) => {
			json.insert("type".into(), "string".into());
			Ok(())
		}
		Description::Struct(_) => {
			let layout = model.layouts().get(layout_ref).unwrap();
			json.insert(
				"$ref".into(),
				layout.id().display(model.vocabulary()).to_string().into(),
			);
			Ok(())
		}
		Description::Enum(_) => {
			todo!("json-schema enum layout")
		}
		Description::Sum(_) => {
			todo!("json-schema sum layout")
		}
		Description::Literal(_) => {
			todo!("json-schema literal layout")
		}
		Description::Native(n, _) => {
			generate_native_type(json, *n);
			Ok(())
		}
	}
}

fn generate_native_type(
	def: &mut serde_json::Map<String, serde_json::Value>,
	n: treeldr::layout::Native,
) {
	use treeldr::layout::Native;
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
}
