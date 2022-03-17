use treeldr::{layout, vocab::Display, Ref};

mod command;
pub use command::Command;

pub enum Error {
	Serialization(serde_json::Error),
}

/// Generate a JSON Schema from a TreeLDR model.
pub fn generate<F>(
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<(), Error> {
	let ld_context = generate_layout_context(model, layout_ref)?;

	println!(
		"{}",
		serde_json::to_string_pretty(&ld_context).map_err(Error::Serialization)?
	);

	Ok(())
}

fn is_empty_context(context: &serde_json::Map<String, serde_json::Value>) -> bool {
	for (key, _) in context {
		if key != "@propagate" {
			return false;
		}
	}

	true
}

fn generate_layout_context<F>(
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
	let mut json = serde_json::Map::new();
	json.insert("@propagate".into(), false.into());

	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Struct(fields) => generate_struct(&mut json, model, fields)?,
		Description::Reference(_) => (),
		Description::Native(_) => (),
	}

	Ok(json)
}

fn generate_layout_type<F>(
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Option<serde_json::Value> {
	let layout = model.layouts().get(layout_ref).unwrap();
	use treeldr::layout::Description;
	match layout.description() {
		Description::Struct(_) => {
			// let ty_ref = *layout.ty().unwrap().inner();
			// let ty = model.types().get(ty_ref).unwrap();
			// let ty_iri = model.vocabulary().get(ty.id()).unwrap();
			// Some(ty_iri.as_str().into())
			None
		}
		Description::Reference(_) => Some("@id".into()),
		Description::Native(n) => Some(generate_native_type(*n)),
	}
}

fn generate_struct<F>(
	json: &mut serde_json::Map<String, serde_json::Value>,
	model: &treeldr::Model<F>,
	fields: &[treeldr::layout::Field<F>],
) -> Result<(), Error> {
	for field in fields {
		let property_ref = field.property();
		let property = model.properties().get(property_ref).unwrap();

		let field_layout_ref = field.layout();
		// let field_layout = model.layouts().get(field_layout_ref).unwrap();
		// let field_layout_iri = model.vocabulary().get(field_layout.id()).unwrap();

		let field_ld_context = generate_layout_context(model, field_layout_ref)?;
		let field_type = generate_layout_type(model, field_layout_ref);
		let field_def: serde_json::Value =
			if is_empty_context(&field_ld_context) && field_type.is_none() {
				property.id().display(model.vocabulary()).to_string().into()
			} else {
				let mut field_def = serde_json::Map::new();
				field_def.insert(
					"@id".into(),
					property.id().display(model.vocabulary()).to_string().into(),
				);

				if let Some(field_type) = field_type {
					field_def.insert("@type".into(), field_type);
				}

				if !is_empty_context(&field_ld_context) {
					field_def.insert("@context".into(), field_ld_context.into());
				}

				field_def.into()
			};

		json.insert(field.name().into(), field_def);
	}

	Ok(())
}

fn generate_native_type(n: treeldr::layout::Native) -> serde_json::Value {
	use treeldr::layout::Native;
	match n {
		Native::Boolean => "http://www.w3.org/2001/XMLSchema#boolean".into(),
		Native::Integer => "http://www.w3.org/2001/XMLSchema#integer".into(),
		Native::PositiveInteger => "http://www.w3.org/2001/XMLSchema#positiveInteger".into(),
		Native::Float => "http://www.w3.org/2001/XMLSchema#float".into(),
		Native::Double => "http://www.w3.org/2001/XMLSchema#double".into(),
		Native::String => "http://www.w3.org/2001/XMLSchema#string".into(),
		Native::Time => "http://www.w3.org/2001/XMLSchema#time".into(),
		Native::Date => "http://www.w3.org/2001/XMLSchema#date".into(),
		Native::DateTime => "http://www.w3.org/2001/XMLSchema#dateTime".into(),
		Native::Iri => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
		Native::Uri => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
		Native::Url => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
	}
}
