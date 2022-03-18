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
	json_schema.insert("title".into(), name.into());
	generate_layout(&mut json_schema, model, embedding, layout_ref)?;

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
		generate_layout(&mut json_schema, model, embedding, layout_ref)?;
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

fn generate_layout<F>(
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model<F>,
	embedding: &embedding::Configuration<F>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<(), Error<F>> {
	let layout = model.layouts().get(layout_ref).unwrap();
	json.insert(
		"$id".into(),
		layout.id().display(model.vocabulary()).to_string().into(),
	);

	if let Some(description) = layout.preferred_documentation(model).short_description() {
		json.insert("description".into(), description.trim().into());
	}

	use treeldr::layout::Description;
	match layout.description() {
		Description::Reference(_, _) => {
			json.insert("type".into(), "string".into());
			Ok(())
		}
		Description::Struct(s) => generate_struct(json, model, embedding, s.fields()),
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
	fields: &[treeldr::layout::Field<F>],
) -> Result<(), Error<F>> {
	let mut properties = serde_json::Map::new();
	let mut required_properties = Vec::new();

	for field in fields {
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
				generate_layout(&mut layout_schema, model, embedding, field_layout_ref)?;
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

		if let Some(description) = field.preferred_documentation(model).short_description() {
			field_schema.insert("description".into(), description.trim().into());
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
	// if let Some(description) = layout.preferred_documentation(model).short_description() {
	// 	json.insert("description".into(), description.trim().into());
	// }

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
			def.insert(
				"pattern".into(),
				"\\d\\d:\\d\\d:\\d\\d(\\.\\d+)?(([+-]\\d\\d:\\d\\d)|Z)?$".into(),
			);
		}
		Native::Date => {
			def.insert("type".into(), "string".into());
			def.insert("pattern".into(), "^\\d{4}-\\d\\d-\\d\\d".into());
		}
		Native::DateTime => {
			def.insert("type".into(), "string".into());
			def.insert(
				"pattern".into(),
				"^\\d{4}-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\d(\\.\\d+)?(([+-]\\d\\d:\\d\\d)|Z)?$"
					.into(),
			);
		}
		Native::Iri => {
			def.insert("type".into(), "string".into());
		}
		Native::Uri => {
			def.insert("type".into(), "string".into());
		}
		Native::Url => {
			def.insert("type".into(), "string".into());
		}
	}
}
