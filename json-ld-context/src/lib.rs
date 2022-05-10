use treeldr::{layout, vocab::Display, Ref, Vocabulary};

mod command;
pub use command::Command;

pub enum Error {
	Serialization(serde_json::Error),
}

/// Generate a JSON Schema from a TreeLDR model.
pub fn generate<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layouts: Vec<Ref<layout::Definition<F>>>,
	type_property: Option<String>,
) -> Result<(), Error> {
	let mut ld_context = serde_json::Map::new();

	for layout_ref in layouts {
		generate_layout_term_definition(vocabulary, model, layout_ref, &mut ld_context)?;
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
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
	ld_context: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<(), Error> {
	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Never(_) => (),
		Description::Struct(s) => {
			let mut def = serde_json::Map::new();

			if let Some(ty_ref) = layout.ty() {
				let ty = model.types().get(ty_ref).unwrap();
				def.insert("@id".into(), ty.id().display(vocabulary).to_string().into());
			}

			def.insert(
				"@context".into(),
				generate_struct_context(vocabulary, model, s.fields())?.into(),
			);

			ld_context.insert(s.name().to_pascal_case(), def.into());
		}
		Description::Enum(_) => (),
		Description::Literal(lit) => {
			if let Some(ty_ref) = layout.ty() {
				let ty = model.types().get(ty_ref).unwrap();

				if !lit.should_inline() {
					let mut def = serde_json::Map::new();
					def.insert("@id".into(), ty.id().display(vocabulary).to_string().into());
					ld_context.insert(lit.name().to_pascal_case(), def.into());
				}
			}
		}
		Description::Reference(_, _) => (),
		Description::Primitive(_, _) => (),
		Description::Set(_) => (),
		Description::Array(_) => (),
		Description::Alias(_, _) => todo!(),
	}

	Ok(())
}

pub struct Context {
	id: serde_json::Value,
	ty: Option<serde_json::Value>,
	container: Option<serde_json::Value>,
}

impl Context {
	fn new(id: serde_json::Value) -> Self {
		Self {
			id,
			ty: None,
			container: None,
		}
	}

	fn into_json(self) -> serde_json::Value {
		if self.ty.is_none() && self.container.is_none() {
			self.id
		} else {
			let mut map = serde_json::Map::new();
			map.insert("@id".into(), self.id);

			if let Some(ty) = self.ty {
				map.insert("@type".into(), ty);
			}

			if let Some(container) = self.container {
				map.insert("@container".into(), container);
			}

			map.into()
		}
	}
}

fn generate_layout_context<F>(
	context: &mut Context,
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
) {
	let layout = model.layouts().get(layout_ref).unwrap();
	use treeldr::layout::Description;

	let non_blank_id = layout.ty().and_then(|ty_ref| {
		let ty = model.types().get(ty_ref).unwrap();
		if ty.id().is_blank() {
			None
		} else {
			Some(ty.id())
		}
	});

	match layout.description() {
		Description::Never(_) => (),
		Description::Struct(_) => {
			if let Some(ty_ref) = layout.ty() {
				let ty = model.types().get(ty_ref).unwrap();
				context.ty = Some(ty.id().display(vocabulary).to_string().into())
			}
		}
		Description::Enum(_) => {
			context.ty = non_blank_id.map(|id| id.display(vocabulary).to_string().into())
		}
		Description::Literal(_) => {
			context.ty = non_blank_id.map(|id| id.display(vocabulary).to_string().into())
		}
		Description::Reference(_, _) => context.ty = Some("@id".into()),
		Description::Primitive(n, _) => context.ty = Some(generate_primitive_type(*n)),
		Description::Set(_) => {
			context.ty = non_blank_id.map(|id| id.display(vocabulary).to_string().into());
			context.container = Some("@set".into());
		}
		Description::Array(_) => {
			context.ty = non_blank_id.map(|id| id.display(vocabulary).to_string().into());
			context.container = Some("@list".into());
		}
		Description::Alias(_, _) => todo!(),
	}
}

fn generate_struct_context<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	fields: &[treeldr::layout::Field<F>],
) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
	let mut json = serde_json::Map::new();

	for field in fields {
		if let Some(property_ref) = field.property() {
			let property = model.properties().get(property_ref).unwrap();

			let field_layout_ref = field.layout();
			let mut field_context =
				Context::new(property.id().display(vocabulary).to_string().into());
			generate_layout_context(&mut field_context, vocabulary, model, field_layout_ref);
			json.insert(field.name().to_camel_case(), field_context.into_json());
		}
	}

	Ok(json)
}

fn generate_primitive_type(n: treeldr::layout::Primitive) -> serde_json::Value {
	use treeldr::layout::Primitive;
	match n {
		Primitive::Boolean => "http://www.w3.org/2001/XMLSchema#boolean".into(),
		Primitive::Integer => "http://www.w3.org/2001/XMLSchema#integer".into(),
		Primitive::PositiveInteger => "http://www.w3.org/2001/XMLSchema#positiveInteger".into(),
		Primitive::Float => "http://www.w3.org/2001/XMLSchema#float".into(),
		Primitive::Double => "http://www.w3.org/2001/XMLSchema#double".into(),
		Primitive::String => "http://www.w3.org/2001/XMLSchema#string".into(),
		Primitive::Time => "http://www.w3.org/2001/XMLSchema#time".into(),
		Primitive::Date => "http://www.w3.org/2001/XMLSchema#date".into(),
		Primitive::DateTime => "http://www.w3.org/2001/XMLSchema#dateTime".into(),
		Primitive::Iri => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
		Primitive::Uri => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
		Primitive::Url => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
	}
}
