use contextual::WithContext;
use derivative::Derivative;
use json_ld::syntax::{
	context::term_definition, Container, ContainerKind, Entry, Keyword, Nullable,
};
use locspan::Meta;
use rdf_types::Vocabulary;
use std::collections::HashMap;
use std::fmt;
use treeldr::{
	layout::{self, Field},
	prop, ty,
	vocab::{Term, TreeLdr},
	BlankIdIndex, IriIndex, Ref,
};

mod command;
pub use command::Command;

/// Generator options.
#[derive(Default)]
pub struct Options {
	pub rdf_type_to_layout_name: bool,
}

#[derive(Derivative)]
#[derivative(PartialEq(bound = ""), Clone(bound = ""), Copy(bound = ""))]
pub enum TermDefinition<M> {
	Property {
		property_ref: Ref<prop::Definition<M>>,
		layout_ref: Ref<layout::Definition<M>>,
	},
	Type {
		type_ref: Ref<ty::Definition<M>>,
		layout_ref: Ref<layout::Definition<M>>,
	},
}

impl<M> TermDefinition<M> {
	pub fn build<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		builder: &ContextBuilder<V, M>,
	) -> Result<Nullable<json_ld::syntax::context::TermDefinition<()>>, Error> {
		let mut definition = json_ld::syntax::context::term_definition::Expanded::new();

		match *self {
			Self::Property {
				property_ref,
				layout_ref,
			} => {
				let id = if builder.is_type_property(property_ref) {
					term_definition::Id::Keyword(Keyword::Type)
				} else if builder.is_id_property(property_ref) {
					term_definition::Id::Keyword(Keyword::Id)
				} else {
					let property = builder.model.properties().get(property_ref).unwrap();
					term_definition::Id::Term(property.id().with(builder.vocabulary).to_string())
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
					term_definition::Id::Term(ty.id().with(builder.vocabulary).to_string());

				definition.id = Some(Entry::new((), Meta(Nullable::Some(syntax_id), ())));
				definition.context = builder.generate_type_definition_context(layout_ref)?;
			}
		}

		Ok(definition.simplify())
	}
}

type ContextEntry = Entry<Box<json_ld::syntax::context::Value<()>>, ()>;

pub struct ContextBuilder<'a, V, M> {
	vocabulary: &'a V,
	model: &'a treeldr::Model<M>,
	options: &'a Options,
	parent: Option<&'a Self>,
	terms: HashMap<String, TermDefinition<M>>,
	do_propagate: bool,
}

impl<'a, V, M> ContextBuilder<'a, V, M> {
	pub fn new(
		vocabulary: &'a V,
		model: &'a treeldr::Model<M>,
		options: &'a Options,
		parent: Option<&'a Self>,
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

	pub fn propagate_context(&'a self) -> Option<&'a Self> {
		if self.do_propagate {
			Some(self)
		} else {
			self.parent
		}
	}

	pub fn is_empty(&self) -> bool {
		self.terms.is_empty()
	}

	pub fn contains(&self, term: &str, definition: &TermDefinition<M>) -> bool {
		match self.terms.get(term) {
			Some(d) => d == definition,
			None => match self.parent {
				Some(parent) => parent.contains(term, definition),
				None => false,
			},
		}
	}

	pub fn insert(&mut self, term: String, definition: TermDefinition<M>) -> Result<(), Error> {
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

	pub fn insert_field(&mut self, field: &'a Field<M>) -> Result<(), Error> {
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

	pub fn insert_typed_layout(
		&mut self,
		layout_ref: Ref<layout::Definition<M>>,
	) -> Result<(), Error> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		let term = layout.name().unwrap().to_string();
		let definition = TermDefinition::Type {
			type_ref: layout.ty().unwrap(),
			layout_ref,
		};

		self.insert(term, definition)
	}

	/// Generate the `@type` entry of a term definition.
	fn generate_property_definition_type(
		&self,
		layout_ref: Ref<layout::Definition<M>>,
	) -> Option<Entry<Nullable<term_definition::Type>, ()>> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description().value() {
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
		layout_ref: Ref<layout::Definition<M>>,
	) -> Option<Entry<Nullable<Container<()>>, ()>> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description().value() {
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
}

impl<'a, V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M> ContextBuilder<'a, V, M> {
	pub fn is_type_property(&self, property_ref: Ref<prop::Definition<M>>) -> bool {
		let property = self.model.properties().get(property_ref).unwrap();
		match property.id().as_iri() {
			Some(iri) => self.vocabulary.iri(iri).unwrap() == json_ld::rdf::RDF_TYPE,
			None => false,
		}
	}

	pub fn is_id_property(&self, property_ref: Ref<prop::Definition<M>>) -> bool {
		let property = self.model.properties().get(property_ref).unwrap();
		match property.id().as_iri() {
			Some(iri) => *iri == IriIndex::Iri(Term::TreeLdr(TreeLdr::Self_)),
			None => false,
		}
	}

	pub fn is_type_field(&self, field: &Field<M>) -> bool {
		match field.property() {
			Some(property_ref) => self.is_type_property(property_ref),
			None => false,
		}
	}

	pub fn is_id_field(&self, field: &Field<M>) -> bool {
		match field.property() {
			Some(property_ref) => self.is_id_property(property_ref),
			None => false,
		}
	}

	pub fn insert_layout_terms(
		&mut self,
		layout_ref: Ref<layout::Definition<M>>,
		typed: bool,
	) -> Result<(), Error> {
		let layout = self.model.layouts().get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.description().value() {
			Description::Set(s) => self.insert_layout_terms(s.item_layout(), typed),
			Description::OneOrMany(s) => self.insert_layout_terms(s.item_layout(), typed),
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
		layout_ref: Ref<layout::Definition<M>>,
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
		layout_ref: Ref<layout::Definition<M>>,
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

	pub fn build(&self) -> Result<json_ld::syntax::context::Value<()>, Error> {
		let mut definition = json_ld::syntax::context::Definition::new();

		for (term, term_definition) in &self.terms {
			if !matches!(term.as_str(), "@type" | "@id") {
				definition.bindings.insert(
					Meta(term.clone().into(), ()),
					Meta(term_definition.build(self)?, ()),
				);
			}
		}

		Ok(json_ld::syntax::context::Value::One(Meta(
			json_ld::syntax::Context::Definition(definition),
			(),
		)))
	}
}

#[derive(Debug)]
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
pub fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
	vocabulary: &V,
	model: &treeldr::Model<M>,
	options: Options,
	layouts: &[Ref<layout::Definition<M>>],
) -> Result<json_ld::syntax::context::Value<()>, Error> {
	let mut builder = ContextBuilder::new(vocabulary, model, &options, None, true);

	for layout_ref in layouts {
		builder.insert_layout_terms(*layout_ref, false)?;
	}

	builder.build()
}

fn generate_primitive_type<M>(
	n: &treeldr::layout::primitive::Restricted<M>,
) -> term_definition::Type {
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
