use iref::{IriBuf, IriRef};
use std::fmt;
use treeldr::{layout, Ref};

mod command;

pub use command::Command;

pub enum Error {
	InvalidLayoutIri(IriBuf),
	UndefinedLayout(IriBuf),
	UnimplementedLayout(IriBuf),
	NotALayout(IriBuf, treeldr::node::CausedTypes),
	Serialization(serde_json::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidLayoutIri(iri) => write!(f, "invalid layout IRI `{}`", iri),
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{}`", iri),
			Self::UnimplementedLayout(iri) => write!(f, "unimplemented layout `{}`", iri),
			Self::NotALayout(iri, _) => write!(f, "node `{}` is not a layout", iri),
			Self::Serialization(e) => write!(f, "JSON serialization failed: {}", e),
		}
	}
}

/// Generate a JSON Schema from a TreeLDR model.
pub fn generate(model: &treeldr::Model, iri_ref: IriRef) -> Result<(), Error> {
	let iri = iri_ref.resolved(model.base_iri());
	let name = iri
		.path()
		.file_name()
		.ok_or_else(|| Error::InvalidLayoutIri(iri.clone()))?;
	let id = model
		.vocabulary()
		.id(&iri)
		.ok_or_else(|| Error::UndefinedLayout(iri.clone()))?;
	let layout_ref = model
		.require_layout(id, None)
		.map_err(|e| match e.inner() {
			treeldr::Error::UnknownNode { .. } => Error::UndefinedLayout(iri.clone()),
			treeldr::Error::InvalidNodeType { found, .. } => Error::NotALayout(iri.clone(), *found),
			_ => unreachable!(),
		})?;

	let mut json_schema = serde_json::Map::new();
	json_schema.insert(
		"$schema".into(),
		"https://json-schema.org/draft/2020-12/schema".into(),
	);
	json_schema.insert("$id".into(), iri.as_str().into());
	json_schema.insert("title".into(), name.into());
	generate_layout(&mut json_schema, model, layout_ref)?;

	println!(
		"{}",
		serde_json::to_string_pretty(&json_schema).map_err(Error::Serialization)?
	);

	Ok(())
}

fn generate_layout(
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model,
	layout_ref: Ref<layout::Definition>,
) -> Result<(), Error> {
	let layout = model.layouts().get(layout_ref).unwrap();
	if let Some(description) = layout.preferred_documentation(model).short_description() {
		json.insert("description".into(), description.trim().into());
	}

	use treeldr::layout::Description;
	match layout.description().expect("unimplemented layout").inner() {
		Description::Struct(fields) => generate_struct(json, model, fields),
		Description::Native(n) => {
			generate_native_type(json, *n);
			Ok(())
		}
	}
}

fn generate_struct(
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model,
	fields: &treeldr::layout::Fields,
) -> Result<(), Error> {
	let mut properties = serde_json::Map::new();
	let mut required_properties = Vec::new();

	for field in fields {
		let field_layout_ref = field.layout().layout();

		let mut layout_schema = serde_json::Map::new();
		generate_layout_ref(&mut layout_schema, model, field_layout_ref)?;

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

fn generate_layout_ref(
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model,
	layout_ref: Ref<layout::Definition>,
) -> Result<(), Error> {
	let layout = model.layouts().get(layout_ref).unwrap();
	// if let Some(description) = layout.preferred_documentation(model).short_description() {
	// 	json.insert("description".into(), description.trim().into());
	// }

	use treeldr::layout::Description;
	match layout.description().expect("unimplemented layout").inner() {
		Description::Struct(_) => {
			let layout = model.layouts().get(layout_ref).unwrap();
			let iri = model.vocabulary().get(layout.id()).unwrap();
			json.insert("$ref".into(), iri.as_str().into());
			Ok(())
		}
		Description::Native(n) => {
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
