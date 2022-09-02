use derivative::Derivative;
use json_ld::syntax::{context::term_definition, Container, ContainerKind, Entry, Nullable};
use locspan::Meta;
use std::collections::HashMap;
use std::fmt;
use treeldr::{
	layout::{self, Field},
	prop,
	vocab::Display,
	Ref, Vocabulary,
};

mod command;
pub use command::Command;

#[derive(Derivative)]
#[derivative(PartialEq(bound = ""))]
pub struct TermDefinition<F> {
	property_ref: Ref<prop::Definition<F>>,
	layout_ref: Ref<layout::Definition<F>>,
}

impl<F> TermDefinition<F> {
	pub fn build(
		&self,
		builder: &ContextBuilder<F>,
	) -> Result<Nullable<json_ld::syntax::context::TermDefinition<()>>, Error> {
		let mut definition = json_ld::syntax::context::term_definition::Expanded::new();

		let property = builder.model.properties().get(self.property_ref).unwrap();
		let syntax_id =
			term_definition::Id::Term(property.id().display(builder.vocabulary).to_string());

		definition.id = Some(Entry::new((), Meta(Nullable::Some(syntax_id), ())));
		definition.type_ = builder.generate_term_definition_type(self.layout_ref);
		definition.container = builder.generate_term_definition_container(self.layout_ref);
		definition.context = builder.generate_term_definition_context(self.layout_ref)?;

		Ok(definition.simplify())
	}
}

pub struct ContextBuilder<'a, F> {
	vocabulary: &'a Vocabulary,
	model: &'a treeldr::Model<F>,
	parent: Option<&'a ContextBuilder<'a, F>>,
	terms: HashMap<String, TermDefinition<F>>,
}

impl<'a, F> ContextBuilder<'a, F> {
	pub fn new(
		vocabulary: &'a Vocabulary,
		model: &'a treeldr::Model<F>,
		parent: Option<&'a ContextBuilder<'a, F>>,
	) -> Self {
		Self {
			vocabulary,
			model,
			parent,
			terms: HashMap::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.terms.is_empty()
	}

	pub fn contains(&self, term: &str, definition: &TermDefinition<F>) -> bool {
		match self.terms.get(term) {
			Some(d) => d == definition,
			None => match self.parent {
				Some(parent) => parent.contains(term, definition),
				None => false,
			},
		}
	}

	pub fn insert_field(&mut self, field: &'a Field<F>) -> Result<(), Error> {
		match field.property() {
			Some(property_ref) => {
				let term = field.name().to_string();
				let definition = TermDefinition {
					property_ref,
					layout_ref: field.layout(),
				};

				if let Some(parent) = self.parent {
					if parent.contains(&term, &definition) {
						return Ok(());
					}
				}

				use std::collections::hash_map::Entry;
				match self.terms.entry(term) {
					Entry::Vacant(entry) => {
						entry.insert(definition);
						Ok(())
					}
					Entry::Occupied(entry) => {
						if *entry.get() != definition {
							Err(Error::Ambiguity(entry.key().clone()))
						} else {
							Ok(())
						}
					}
				}
			}
			None => Ok(()),
		}
	}

	pub fn insert_layout(&mut self, layout_ref: Ref<layout::Definition<F>>) -> Result<(), Error> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description() {
			Description::Set(s) => self.insert_layout(s.item_layout()),
			Description::Required(o) => self.insert_layout(o.item_layout()),
			Description::Option(o) => self.insert_layout(o.item_layout()),
			Description::Struct(s) => {
				for field in s.fields() {
					self.insert_field(field)?
				}

				Ok(())
			}
			_ => Ok(()),
		}
	}

	/// Generate the `@context` entry of a term definition.
	fn generate_term_definition_context(
		&self,
		layout_ref: Ref<layout::Definition<F>>,
	) -> Result<Option<Entry<Box<json_ld::syntax::context::Value<()>>, ()>>, Error> {
		let mut builder = ContextBuilder::new(self.vocabulary, self.model, Some(self));

		builder.insert_layout(layout_ref)?;

		if builder.is_empty() {
			Ok(None)
		} else {
			let context = builder.build()?;

			Ok(Some(Entry::new((), Meta(Box::new(context), ()))))
		}
	}

	/// Generate the `@type` entry of a term definition.
	fn generate_term_definition_type(
		&self,
		layout_ref: Ref<layout::Definition<F>>,
	) -> Option<Entry<Nullable<term_definition::Type>, ()>> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description() {
			Description::Required(r) => self.generate_term_definition_type(r.item_layout()),
			Description::Option(o) => self.generate_term_definition_type(o.item_layout()),
			Description::Primitive(n, _) => {
				let syntax_ty = generate_primitive_type(n);
				Some(Entry::new((), Meta(Nullable::Some(syntax_ty), ())))
			}
			_ => None,
		}
	}

	/// Generate the `@container` entry of a term definition.
	fn generate_term_definition_container(
		&self,
		layout_ref: Ref<layout::Definition<F>>,
	) -> Option<Entry<Nullable<Container<()>>, ()>> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description() {
			Description::Set(_) => Some(Entry::new(
				(),
				Meta(Nullable::Some(Container::One(ContainerKind::Set)), ()),
			)),
			Description::Array(_) => Some(Entry::new(
				(),
				Meta(Nullable::Some(Container::One(ContainerKind::List)), ()),
			)),
			_ => None,
		}
	}

	pub fn build(&self) -> Result<json_ld::syntax::context::Value<()>, Error> {
		let mut definition = json_ld::syntax::context::Definition::new();

		for (term, term_definition) in &self.terms {
			definition.bindings.insert(
				Meta(term.clone().into(), ()),
				Meta(term_definition.build(self)?, ()),
			);
		}

		Ok(json_ld::syntax::context::Value::One(Meta(
			json_ld::syntax::Context::Definition(definition),
			(),
		)))
	}
}

pub enum Error {
	Ambiguity(String),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Ambiguity(term) => write!(f, "term `{}` is ambiguous", term),
		}
	}
}

/// Generate a JSON-LD context from a TreeLDR model.
pub fn generate<F>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	layouts: Vec<Ref<layout::Definition<F>>>,
) -> Result<json_ld::syntax::context::Value<()>, Error> {
	let mut builder = ContextBuilder::new(vocabulary, model, None);

	for layout_ref in layouts {
		builder.insert_layout(layout_ref)?;
	}

	builder.build()
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
