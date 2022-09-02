use json_ld::syntax::{Nullable, Container, ContainerKind, Entry, context::term_definition};
use treeldr::{layout::{self, Field}, prop, vocab::Display, Ref, Vocabulary};
use locspan::Meta;
use std::collections::HashMap;
use std::fmt;

mod command;
pub use command::Command;

pub struct TermDefinition<F> {
	property_ref: Ref<prop::Definition<F>>,
	layout_ref: Ref<layout::Definition<F>>
}

impl<F> TermDefinition<F> {
	pub fn build(
		self,
		vocabulary: &Vocabulary,
		model: &treeldr::Model<F>
	) -> Nullable<json_ld::syntax::context::TermDefinition<()>> {
		let mut definition = json_ld::syntax::context::term_definition::Expanded::new();

		let property = model.properties().get(self.property_ref).unwrap();
		let syntax_id = term_definition::Id::Term(property.id().display(vocabulary).to_string());

		definition.id = Some(Entry::new((), Meta(Nullable::Some(syntax_id), ())));
		definition.type_ = generate_term_definition_type(vocabulary, model, self.layout_ref);
		definition.container = generate_term_definition_container(model, self.layout_ref);

		definition.simplify()
	}
}

pub struct ContextBuilder<'a, F> {
	vocabulary: &'a Vocabulary,
	model: &'a treeldr::Model<F>,
	terms: HashMap<String, TermDefinition<F>>
}

impl<'a, F> ContextBuilder<'a, F> {
	pub fn new(
		vocabulary: &'a Vocabulary,
		model: &'a treeldr::Model<F>,
	) -> Self {
		Self { vocabulary, model, terms: HashMap::new() }
	}

	pub fn insert_field(&mut self, field: &'a Field<F>) -> Result<(), Error> {
		match field.property() {
			Some(property_ref) => {
				let name = field.name().to_string();

				use std::collections::hash_map::Entry;
				match self.terms.entry(name) {
					Entry::Vacant(entry) => {
						entry.insert(TermDefinition {
							property_ref,
							layout_ref: field.layout()
						});

						Ok(())
					}
					Entry::Occupied(entry) => {
						let term_definition = entry.get();

						if term_definition.property_ref != property_ref || term_definition.layout_ref != field.layout() {
							Err(Error::Ambiguity(entry.key().clone()))
						} else {
							Ok(())
						}
					}
				}
			}
			None => {
				Ok(())
			}
		}
	}

	pub fn insert_layout(&mut self, layout_ref: Ref<layout::Definition<F>>) -> Result<(), Error> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description() {
			Description::Struct(s) => {
				for field in s.fields() {
					self.insert_field(field)?
				}
			}
			_ => ()
		}

		Ok(())
	}

	pub fn build(self) -> json_ld::syntax::context::Value<()> {
		let mut definition = json_ld::syntax::context::Definition::new();

		for (term, term_definition) in self.terms {
			definition.bindings.insert(
				Meta(term.into(), ()),
				Meta(term_definition.build(self.vocabulary, self.model), ())
			);
		}

		json_ld::syntax::context::Value::One(Meta(
			json_ld::syntax::Context::Definition(definition),
			()
		))
	}
}

pub enum Error {
	Ambiguity(String)
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Ambiguity(term) => write!(f, "term `{}` is ambiguous", term)
		}
	}
}

/// Generate a JSON-LD context from a TreeLDR model.
pub fn generate<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layouts: Vec<Ref<layout::Definition<F>>>,
) -> Result<json_ld::syntax::context::Value<()>, Error> {
	let mut builder = ContextBuilder::new(vocabulary, model);

	for layout_ref in layouts {
		builder.insert_layout(layout_ref)?;
	}

	Ok(builder.build())
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
