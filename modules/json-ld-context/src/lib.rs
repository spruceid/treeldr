use derivative::Derivative;
use json_ld::syntax::{
	context::term_definition, Container, ContainerKind, Entry, Keyword, Nullable,
};
use locspan::Meta;
use std::collections::HashMap;
use std::fmt;
use treeldr::{
	layout::{self, Field},
	prop, ty,
	vocab::Display,
	Ref, Vocabulary,
};

mod command;
pub use command::Command;

/// Generator options.
#[derive(Default)]
pub struct Options {
	rdf_type_to_layout_name: bool,
}

#[derive(Derivative)]
#[derivative(PartialEq(bound = ""), Clone(bound = ""), Copy(bound = ""))]
pub enum TermDefinition<F> {
	Property {
		property_ref: Ref<prop::Definition<F>>,
		layout_ref: Ref<layout::Definition<F>>,
	},
	Type {
		type_ref: Ref<ty::Definition<F>>,
		layout_ref: Ref<layout::Definition<F>>,
	},
}

impl<F> TermDefinition<F> {
	pub fn build(
		&self,
		builder: &ContextBuilder<F>,
	) -> Result<Nullable<json_ld::syntax::context::TermDefinition<()>>, Error> {
		let mut definition = json_ld::syntax::context::term_definition::Expanded::new();

		match *self {
			Self::Property {
				property_ref,
				layout_ref,
			} => {
				let id = if builder.is_type_property(property_ref) {
					term_definition::Id::Keyword(Keyword::Type)
				} else {
					let property = builder.model.properties().get(property_ref).unwrap();
					term_definition::Id::Term(property.id().display(builder.vocabulary).to_string())
				};

				definition.id = Some(Entry::new((), Meta(Nullable::Some(id), ())));
				definition.type_ = builder.generate_property_definition_type(layout_ref);
				definition.container = builder.generate_property_definition_container(layout_ref);
				definition.context = builder.generate_property_definition_context(layout_ref)?;
			}
			Self::Type {
				type_ref,
				layout_ref,
			} => {
				let ty = builder.model.types().get(type_ref).unwrap();
				let syntax_id =
					term_definition::Id::Term(ty.id().display(builder.vocabulary).to_string());

				definition.id = Some(Entry::new((), Meta(Nullable::Some(syntax_id), ())));
				definition.context = builder.generate_type_definition_context(layout_ref)?;
			}
		}

		Ok(definition.simplify())
	}
}

type ContextEntry = Entry<Box<json_ld::syntax::context::Value<()>>, ()>;

pub struct ContextBuilder<'a, F> {
	vocabulary: &'a Vocabulary,
	model: &'a treeldr::Model<F>,
	options: &'a Options,
	parent: Option<&'a ContextBuilder<'a, F>>,
	terms: HashMap<String, TermDefinition<F>>,
	do_propagate: bool,
}

impl<'a, F> ContextBuilder<'a, F> {
	pub fn new(
		vocabulary: &'a Vocabulary,
		model: &'a treeldr::Model<F>,
		options: &'a Options,
		parent: Option<&'a ContextBuilder<'a, F>>,
		do_propagate: bool,
	) -> Self {
		Self {
			vocabulary,
			model,
			options,
			parent,
			terms: HashMap::new(),
			do_propagate,
		}
	}

	pub fn propagate_context(&'a self) -> Option<&'a ContextBuilder<'a, F>> {
		if self.do_propagate {
			Some(self)
		} else {
			self.parent
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

	pub fn insert(&mut self, term: String, definition: TermDefinition<F>) -> Result<(), Error> {
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

	pub fn insert_field(&mut self, field: &'a Field<F>) -> Result<(), Error> {
		match field.property() {
			Some(property_ref) => {
				let term = field.name().to_string();
				let definition = TermDefinition::Property {
					property_ref,
					layout_ref: field.layout(),
				};

				self.insert(term, definition)
			}
			None => Ok(()),
		}
	}

	pub fn is_type_property(&self, property_ref: Ref<prop::Definition<F>>) -> bool {
		let property = self.model.properties().get(property_ref).unwrap();
		match property.id().as_iri() {
			Some(iri) => iri.iri(self.vocabulary).unwrap() == json_ld::rdf::RDF_TYPE,
			None => false,
		}
	}

	pub fn is_type_field(&self, field: &Field<F>) -> bool {
		match field.property() {
			Some(property_ref) => self.is_type_property(property_ref),
			None => false,
		}
	}

	pub fn insert_typed_layout(
		&mut self,
		layout_ref: Ref<layout::Definition<F>>,
	) -> Result<(), Error> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		let term = layout.name().unwrap().to_string();
		let definition = TermDefinition::Type {
			type_ref: layout.ty().unwrap(),
			layout_ref,
		};

		self.insert(term, definition)
	}

	pub fn insert_layout_terms(
		&mut self,
		layout_ref: Ref<layout::Definition<F>>,
		typed: bool,
	) -> Result<(), Error> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description() {
			Description::Set(s) => self.insert_layout_terms(s.item_layout(), typed),
			Description::Required(o) => self.insert_layout_terms(o.item_layout(), typed),
			Description::Option(o) => self.insert_layout_terms(o.item_layout(), typed),
			Description::Struct(s) => {
				if !typed && self.options.rdf_type_to_layout_name {
					// check if there is a required `rdf:type` property field.
					for field in s.fields() {
						if field.is_required(self.model) && self.is_type_field(field) {
							self.insert_field(field)?;
							self.insert_typed_layout(layout_ref)?;

							return Ok(());
						}
					}
				}

				// otherwise, include the fields directly in this context.
				for field in s.fields() {
					self.insert_field(field)?
				}

				Ok(())
			}
			_ => Ok(()),
		}
	}

	/// Generate the `@context` entry of a property definition.
	fn generate_property_definition_context(
		&self,
		layout_ref: Ref<layout::Definition<F>>,
	) -> Result<Option<ContextEntry>, Error> {
		let mut builder = ContextBuilder::new(
			self.vocabulary,
			self.model,
			self.options,
			self.propagate_context(),
			true,
		);

		builder.insert_layout_terms(layout_ref, false)?;

		if builder.is_empty() {
			Ok(None)
		} else {
			let context = builder.build()?;

			Ok(Some(Entry::new((), Meta(Box::new(context), ()))))
		}
	}

	/// Generate the `@context` entry of a type definition.
	fn generate_type_definition_context(
		&self,
		layout_ref: Ref<layout::Definition<F>>,
	) -> Result<Option<ContextEntry>, Error> {
		let mut builder = ContextBuilder::new(
			self.vocabulary,
			self.model,
			self.options,
			self.propagate_context(),
			false,
		);

		builder.insert_layout_terms(layout_ref, true)?;

		if builder.is_empty() {
			Ok(None)
		} else {
			let context = builder.build()?;

			Ok(Some(Entry::new((), Meta(Box::new(context), ()))))
		}
	}

	/// Generate the `@type` entry of a term definition.
	fn generate_property_definition_type(
		&self,
		layout_ref: Ref<layout::Definition<F>>,
	) -> Option<Entry<Nullable<term_definition::Type>, ()>> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description() {
			Description::Required(r) => self.generate_property_definition_type(r.item_layout()),
			Description::Option(o) => self.generate_property_definition_type(o.item_layout()),
			Description::Primitive(n, _) => {
				let syntax_ty = generate_primitive_type(n);
				Some(Entry::new((), Meta(Nullable::Some(syntax_ty), ())))
			}
			_ => None,
		}
	}

	/// Generate the `@container` entry of a term definition.
	fn generate_property_definition_container(
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
	options: Options,
	layouts: Vec<Ref<layout::Definition<F>>>,
) -> Result<json_ld::syntax::context::Value<()>, Error> {
	let mut builder = ContextBuilder::new(vocabulary, model, &options, None, true);

	for layout_ref in layouts {
		builder.insert_layout_terms(layout_ref, false)?;
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
