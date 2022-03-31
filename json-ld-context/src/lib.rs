use treeldr::{layout, vocab::Display, Ref};

mod command;
pub use command::Command;

pub enum Error {
	Serialization(serde_json::Error),
}

/// Generate a JSON Schema from a TreeLDR model.
pub fn generate<F>(
	model: &treeldr::Model<F>,
	layouts: Vec<Ref<layout::Definition<F>>>,
	type_property: Option<String>,
) -> Result<(), Error> {
	let mut ld_context = serde_json::Map::new();

	for layout_ref in layouts {
		generate_layout_term_definition(model, layout_ref, &mut ld_context)?;
	}

	if let Some(name) = type_property {
		ld_context.insert(name, "@type".into());
	}

	println!(
		"{}",
		serde_json::to_string_pretty(&ld_context).map_err(Error::Serialization)?
	);

	Ok(())
}

fn generate_layout_term_definition<F>(
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
	ld_context: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<(), Error> {
	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Struct(s) => {
			let ty_ref = layout.ty();
			let ty = model.types().get(ty_ref).unwrap();

			let mut def = serde_json::Map::new();
			def.insert(
				"@id".into(),
				ty.id().display(model.vocabulary()).to_string().into(),
			);
			def.insert(
				"@context".into(),
				generate_struct_context(model, s.fields())?.into(),
			);

			ld_context.insert(s.name().to_pascal_case(), def.into());
		}
		Description::Enum(_) => {
			todo!("ld-context enum layout")
		}
		Description::Literal(lit) => {
			let ty_ref = layout.ty();
			let ty = model.types().get(ty_ref).unwrap();

			if !lit.should_inline() {
				let mut def = serde_json::Map::new();
				def.insert(
					"@id".into(),
					ty.id().display(model.vocabulary()).to_string().into(),
				);
				ld_context.insert(lit.name().to_pascal_case(), def.into());
			}
		}
		Description::Reference(_, _) => (),
		Description::Native(_, _) => (),
	}

	Ok(())
}

fn generate_layout_type<F>(
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
) -> Option<serde_json::Value> {
	let layout = model.layouts().get(layout_ref).unwrap();
	use treeldr::layout::Description;
	match layout.description() {
		Description::Struct(_) => {
			let ty_ref = layout.ty();
			let ty = model.types().get(ty_ref).unwrap();
			Some(ty.id().display(model.vocabulary()).to_string().into())
		}
		Description::Enum(_) => {
			todo!("ld-context enum layout")
		}
		Description::Literal(_) => {
			let ty_ref = layout.ty();
			let ty = model.types().get(ty_ref).unwrap();
			if ty.id().is_blank() {
				None
			} else {
				Some(ty.id().display(model.vocabulary()).to_string().into())
			}
		}
		Description::Reference(_, _) => Some("@id".into()),
		Description::Native(n, _) => Some(generate_native_type(*n)),
	}
}

fn generate_struct_context<F>(
	model: &treeldr::Model<F>,
	fields: &[treeldr::layout::Field<F>],
) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
	let mut json = serde_json::Map::new();

	for field in fields {
		let property_ref = field.property();
		let property = model.properties().get(property_ref).unwrap();

		let field_layout_ref = field.layout();
		let field_type = generate_layout_type(model, field_layout_ref);
		let field_def: serde_json::Value = if field_type.is_none() {
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

			field_def.into()
		};

		json.insert(field.name().to_camel_case(), field_def);
	}

	Ok(json)
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
