use json_ld::syntax::{Nullable, Container, ContainerKind, Entry, context::term_definition};
use treeldr::{layout::{self, Field}, vocab::Display, Ref, Vocabulary};
use locspan::Meta;

mod command;
pub use command::Command;

pub enum Error {
	Serialization(serde_json::Error),
}

/// Generate a JSON-LD context from a TreeLDR model.
pub fn generate<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layouts: Vec<Ref<layout::Definition<F>>>,
) -> Result<json_ld::syntax::context::Value<()>, Error> {
	let mut context = json_ld::syntax::context::Definition::new();

	for layout_ref in layouts {
		generate_layout_term_definitions(vocabulary, model, layout_ref, &mut context)?;
	}

	Ok(json_ld::syntax::context::Value::One(Meta(
		json_ld::syntax::Context::Definition(context),
		()
	)))
}

/// Generate all the expanded term definitions for the given layout.
fn generate_layout_term_definitions<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>,
	context: &mut json_ld::syntax::context::Definition<()>
) -> Result<(), Error> {
	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Never(_) => (),
		Description::Struct(s) => {
			for field in s.fields() {
				if let Some(definition) = generate_field_term_definition(vocabulary, model, field)? {
					context.bindings.insert(
						Meta(field.name().to_camel_case().into(), ()),
						Meta(definition, ())
					);
				}
			}
		}
		Description::Enum(_) => (),
		Description::Reference(_) => (),
		Description::Primitive(_, _) => (),
		Description::Required(_) => (),
		Description::Option(_) => (),
		Description::Set(_) => (),
		Description::Array(_) => (),
		Description::Alias(_, _) => todo!(),
	}

	Ok(())
}

fn generate_field_term_definition<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	field: &Field<F>,
) -> Result<Option<Nullable<json_ld::syntax::context::TermDefinition<()>>>, Error> {
	match field.property() {
		Some(property_ref) => {
			let property = model.properties().get(property_ref).unwrap();
			
			let field_layout_ref = field.layout();

			let mut definition = json_ld::syntax::context::term_definition::Expanded::new();
			let syntax_id = term_definition::Id::Term(property.id().display(vocabulary).to_string());
			definition.id = Some(Entry::new((), Meta(Nullable::Some(syntax_id), ())));
			definition.type_ = generate_term_definition_type(vocabulary, model, field_layout_ref);
			definition.container = generate_term_definition_container(model, field_layout_ref);

			Ok(Some(definition.simplify()))
		}
		None => {
			Ok(None)
		}
	}
}

/// Generate the `@type` entry of a term definition.
fn generate_term_definition_type<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>
) -> Option<Entry<Nullable<term_definition::Type>, ()>> {
	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Required(r) => {
			generate_term_definition_type(vocabulary, model, r.item_layout())
		}
		Description::Option(o) => {
			generate_term_definition_type(vocabulary, model, o.item_layout())
		},
		Description::Primitive(n, _) => {
			let syntax_ty = generate_primitive_type(n);
			Some(Entry::new((), Meta(Nullable::Some(syntax_ty), ())))
		},
		_ => None
	}
}

/// Generate the `@container` entry of a term definition.
fn generate_term_definition_container<F>(
	model: &treeldr::Model<F>,
	layout_ref: Ref<layout::Definition<F>>
) -> Option<Entry<Nullable<Container<()>>, ()>> {
	let layout = model.layouts().get(layout_ref).unwrap();

	use treeldr::layout::Description;
	match layout.description() {
		Description::Set(_) => {
			Some(Entry::new((), Meta(Nullable::Some(Container::One(ContainerKind::Set)), ())))
		}
		Description::Array(_) => {
			Some(Entry::new((), Meta(Nullable::Some(Container::One(ContainerKind::List)), ())))
		}
		_ => None
	}
}

fn generate_primitive_type(n: &treeldr::layout::RestrictedPrimitive) -> term_definition::Type {
	use treeldr::layout::Primitive;

	let iri: String = match n.primitive() {
		Primitive::Boolean => "http://www.w3.org/2001/XMLSchema#boolean".into(),
		Primitive::Integer => "http://www.w3.org/2001/XMLSchema#integer".into(),
		Primitive::UnsignedInteger => "http://www.w3.org/2001/XMLSchema#nonNegativeInteger".into(),
		Primitive::Float => "http://www.w3.org/2001/XMLSchema#float".into(),
		Primitive::Double => "http://www.w3.org/2001/XMLSchema#double".into(),
		Primitive::String => "http://www.w3.org/2001/XMLSchema#string".into(),
		Primitive::Time => "http://www.w3.org/2001/XMLSchema#time".into(),
		Primitive::Date => "http://www.w3.org/2001/XMLSchema#date".into(),
		Primitive::DateTime => "http://www.w3.org/2001/XMLSchema#dateTime".into(),
		Primitive::Iri => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
		Primitive::Uri => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
		Primitive::Url => "http://www.w3.org/2001/XMLSchema#anyURI".into(),
	};

	term_definition::Type::Term(iri)
}
