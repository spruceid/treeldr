use contextual::WithContext;
use futures::{future::BoxFuture, FutureExt};
use json_ld::{
	syntax::{context::term_definition, Entry, Keyword, Nullable},
	Context, ContextLoader,
};
use locspan::{BorrowStripped, Meta};
use rdf_types::{Vocabulary, VocabularyMut};
use shelves::{Ref, Shelf};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use treeldr::{
	vocab::{Term, TreeLdr},
	BlankIdIndex, Id, IriIndex, TId,
};

pub use json_ld;
mod command;
pub use command::{Command, Files};

fn clone_definition<T: Clone, B: Clone, C: Clone, M: Clone>(
	def: json_ld::context::TermDefinitionRef<T, B, C, M>,
) -> json_ld::context::TermDefinition<T, B, C, M> {
	match def {
		json_ld::context::TermDefinitionRef::Normal(n) => {
			json_ld::context::TermDefinition::Normal(n.clone())
		}
		json_ld::context::TermDefinitionRef::Type(t) => {
			json_ld::context::TermDefinition::Type(t.clone())
		}
	}
}

pub type ProtectedDefinitions<M> = Vec<ProtectedDefinition<M>>;

pub type ProtectedDefinition<M> = (
	json_ld::context::TermDefinition<IriIndex, BlankIdIndex, json_ld::syntax::context::Value<M>, M>,
	ProcessedContext<M>,
);

pub type ProcessedContext<M> =
	Context<IriIndex, BlankIdIndex, json_ld::syntax::context::Value<M>, M>;

async fn find_protected_definitions<'a, V, L, M>(
	vocabulary: &'a mut V,
	model: &'a treeldr::Model<M>,
	loader: &'a mut L,
	context: &'a ProcessedContext<M>,
	term: &'a str,
	visit_type_defs: bool,
) -> Result<ProtectedDefinitions<M>, Meta<json_ld::context_processing::Error<L::ContextError>, M>>
where
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex> + Send + Sync,
	L: ContextLoader<IriIndex, M> + Send + Sync,
	L::Context: Into<json_ld::syntax::context::Value<M>>,
	M: Clone + Send + Sync,
{
	use json_ld::Process;
	let mut result = Vec::new();
	for binding in context.definitions() {
		if binding.key().as_str() == term && binding.definition().protected() {
			result.push((clone_definition(binding.definition()), context.clone()));
		}

		if visit_type_defs {
			if let Some(local_context) = binding.definition().context() {
				if let Some(json_ld::Term::Ref(id)) = binding.definition().value() {
					if let Ok(id) = Id::try_from(id.clone()) {
						if let Some(node) = model.get_resource(id) {
							if node.is_type() {
								let new_context = local_context
									.value
									.process_full(
										vocabulary,
										context,
										loader,
										context.base_iri().cloned(),
										json_ld::context_processing::Options::default()
											.without_propagation(),
										(),
									)
									.await?;

								for binding in new_context.definitions() {
									if binding.key().as_str() == term
										&& binding.definition().protected()
									{
										result.push((
											clone_definition(binding.definition()),
											new_context.clone(),
										));
									}
								}
							}
						}
					}
				}
			}
		}
	}

	Ok(result)
}

/// Generator options.
pub struct Options<V: Vocabulary, M> {
	pub rdf_type_to_layout_name: bool,
	pub context: Context<V::Iri, V::BlankId, json_ld::syntax::context::Value<M>, M>,
}

impl<V: Vocabulary, M> Default for Options<V, M> {
	fn default() -> Self {
		Self {
			rdf_type_to_layout_name: false,
			context: Context::default(),
		}
	}
}

pub struct LocalContext {
	terms: BTreeMap<String, TermDefinition>,
	parent: ParentContext,
	do_propagate: bool,
}

impl LocalContext {
	pub fn new(parent: ParentContext, do_propagate: bool) -> Self {
		Self {
			terms: BTreeMap::new(),
			parent,
			do_propagate,
		}
	}

	pub fn insert(
		&mut self,
		term: String,
		definition: TermDefinition,
	) -> Result<Option<PotentialAmbiguity>, Error> {
		use std::collections::btree_map::Entry;
		match self.terms.entry(term) {
			Entry::Vacant(entry) => {
				entry.insert(definition);
				Ok(None)
			}
			Entry::Occupied(entry) => {
				if entry.get().without_context() != definition.without_context() {
					Err(Error::Ambiguity(entry.key().clone()))
				} else {
					Ok(Some(PotentialAmbiguity(
						entry.key().clone(),
						entry.get().context,
						definition.context,
					)))
				}
			}
		}
	}

	pub fn is_empty(&self) -> bool {
		self.terms.is_empty()
	}

	pub fn get<'a, M>(
		&'a self,
		extern_contexts: &'a Context<IriIndex, BlankIdIndex, json_ld::syntax::context::Value<M>, M>,
		contexts: &'a Shelf<Vec<LocalContext>>,
		term: &str,
	) -> Option<TermDefinitionRef<'a, M>> {
		self.terms
			.get(term)
			.map(TermDefinitionRef::Intern)
			.or_else(|| self.parent.get(extern_contexts, contexts, term))
	}
}

pub struct PotentialAmbiguity(String, Ref<LocalContext>, Ref<LocalContext>);

impl PotentialAmbiguity {
	fn resolve<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		self,
		builder: &ContextBuilder<V, M>,
	) -> Result<(), Error> {
		if builder.context_eq(self.1, self.2) {
			Ok(())
		} else {
			Err(Error::Ambiguity(self.0))
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermDefinitionWithoutContext {
	id: json_ld::Term<IriIndex, BlankIdIndex>,
	type_: Option<json_ld::Type<IriIndex>>,
	container: json_ld::Container,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermDefinition {
	id: json_ld::Term<IriIndex, BlankIdIndex>,
	type_: Option<json_ld::Type<IriIndex>>,
	container: json_ld::Container,
	context: Ref<LocalContext>,
}

impl TermDefinition {
	pub fn without_context(&self) -> TermDefinitionWithoutContext {
		TermDefinitionWithoutContext {
			id: self.id.clone(),
			type_: self.type_.clone(),
			container: self.container,
		}
	}

	pub fn intern_cmp<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		builder: &ContextBuilder<V, M>,
		other: &Self,
	) -> TermDefinitionCmp {
		if self.id == other.id && self.type_ == other.type_ && self.container == other.container {
			if builder.context_eq(self.context, other.context) {
				TermDefinitionCmp::Equal
			} else {
				TermDefinitionCmp::ContextNeq
			}
		} else {
			TermDefinitionCmp::Neq
		}
	}

	pub fn context_eq<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		builder: &ContextBuilder<V, M>,
		other: &Self,
	) -> bool {
		self.intern_cmp(builder, other) == TermDefinitionCmp::Equal
	}

	pub fn build_id<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &V,
	) -> Option<Entry<Nullable<term_definition::Id>, ()>> {
		let syntax_id = match self.id.clone() {
			json_ld::Term::Null => Nullable::Null,
			json_ld::Term::Keyword(k) => Nullable::Some(term_definition::Id::Keyword(k)),
			json_ld::Term::Ref(r) => {
				Nullable::Some(term_definition::Id::Term(r.with(vocabulary).to_string()))
			}
		};

		Some(Entry::new((), Meta(syntax_id, ())))
	}

	pub fn build_type<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &V,
	) -> Option<Entry<Nullable<term_definition::Type>, ()>> {
		let syntax_type = self.type_.as_ref().map(|t| match t {
			json_ld::Type::Id => term_definition::Type::Keyword(term_definition::TypeKeyword::Id),
			json_ld::Type::Json => {
				term_definition::Type::Keyword(term_definition::TypeKeyword::Json)
			}
			json_ld::Type::None => {
				term_definition::Type::Keyword(term_definition::TypeKeyword::None)
			}
			json_ld::Type::Vocab => {
				term_definition::Type::Keyword(term_definition::TypeKeyword::Vocab)
			}
			json_ld::Type::Ref(i) => {
				term_definition::Type::Term(vocabulary.iri(i).unwrap().to_string())
			}
		});

		syntax_type.map(|v| Entry::new((), Meta(Nullable::Some(v), ())))
	}

	pub fn build<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		vocabulary: &V,
		builder: &ContextBuilder<V, M>,
	) -> Result<Nullable<json_ld::syntax::context::TermDefinition<()>>, Error> {
		let mut definition = json_ld::syntax::context::term_definition::Expanded::new();

		definition.id = self.build_id(vocabulary);
		definition.type_ = self.build_type(vocabulary);
		definition.container = self
			.container
			.into_syntax(())
			.map(|v| Entry::new((), v.map(Nullable::Some)));

		definition.context = if builder.is_context_empty(self.context) {
			None
		} else {
			Some(Entry::new(
				(),
				Meta(Box::new(builder.build(vocabulary, self.context)?), ()),
			))
		};

		Ok(definition.simplify())
	}

	pub fn eq_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		vocabulary: &V,
		builder: &ContextBuilder<V, M>,
		other: &Nullable<json_ld::syntax::context::TermDefinition<M>>,
	) -> bool {
		match self.build(vocabulary, builder) {
			Ok(a) => a.stripped() == other.stripped(),
			Err(_) => false,
		}
	}
}

pub enum TermDefinitionRef<'a, M> {
	Intern(&'a TermDefinition),
	Extern(
		json_ld::context::TermDefinitionRef<
			'a,
			IriIndex,
			BlankIdIndex,
			json_ld::syntax::context::Value<M>,
			M,
		>,
	),
}

impl<'a, M> TermDefinitionRef<'a, M> {
	fn intern_cmp<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &V,
		builder: &ContextBuilder<V, M>,
		def: &TermDefinition,
	) -> TermDefinitionCmp {
		match self {
			Self::Intern(i) => i.intern_cmp(builder, def),
			Self::Extern(e) => match e {
				json_ld::context::TermDefinitionRef::Type(_) => TermDefinitionCmp::Equal,
				json_ld::context::TermDefinitionRef::Normal(n) => {
					if !n.prefix
						&& !n.reverse_property && n.base_url.is_none()
						&& n.direction.is_none() && n.index.is_none()
						&& n.language.is_none() && n.nest.is_none()
						&& n.container == def.container
						&& n.value.as_ref() == Some(&def.id)
						&& n.typ == def.type_
					{
						match &n.context {
							Some(context) => {
								if builder.context_extern_eq(
									vocabulary,
									context.value(),
									def.context,
								) {
									TermDefinitionCmp::Equal
								} else {
									TermDefinitionCmp::ContextNeq
								}
							}
							None => {
								if builder.is_context_empty(def.context) {
									TermDefinitionCmp::Equal
								} else {
									TermDefinitionCmp::ContextNeq
								}
							}
						}
					} else {
						TermDefinitionCmp::Neq
					}
				}
			},
		}
	}
}

pub enum ContextRef<'a, M> {
	Intern(&'a LocalContext),
	Extern(Option<&'a Meta<json_ld::syntax::context::Value<M>, M>>),
}

#[derive(Debug, Clone, Copy)]
pub enum ParentContext {
	Extern,
	Intern(Ref<LocalContext>),
}

impl ParentContext {
	pub fn is_extern(&self) -> bool {
		matches!(self, Self::Extern)
	}

	pub fn get<'a, M>(
		&self,
		extern_context: &'a Context<IriIndex, BlankIdIndex, json_ld::syntax::context::Value<M>, M>,
		contexts: &'a Shelf<Vec<LocalContext>>,
		term: &str,
	) -> Option<TermDefinitionRef<'a, M>> {
		match self {
			Self::Intern(i) => contexts
				.get(*i)
				.unwrap()
				.get(extern_context, contexts, term),
			Self::Extern => extern_context.get(term).map(TermDefinitionRef::Extern),
		}
	}

	pub fn definitions<'a, M>(
		&self,
		extern_context: &'a Context<IriIndex, BlankIdIndex, json_ld::syntax::context::Value<M>, M>,
		contexts: &'a Shelf<Vec<LocalContext>>,
		term: &str,
	) -> Option<TermDefinitionRef<'a, M>> {
		match self {
			Self::Intern(i) => contexts
				.get(*i)
				.unwrap()
				.get(extern_context, contexts, term),
			Self::Extern => extern_context.get(term).map(TermDefinitionRef::Extern),
		}
	}
}

pub struct ContextBuilder<'a, V: Vocabulary, M> {
	model: &'a treeldr::Model<M>,
	options: &'a Options<V, M>,
	contexts: Shelf<Vec<LocalContext>>,
	reference_layouts: HashMap<TId<treeldr::Layout>, bool>,
}

impl<'a, V: Vocabulary, M> ContextBuilder<'a, V, M> {
	pub fn new(model: &'a treeldr::Model<M>, options: &'a Options<V, M>) -> Self {
		Self {
			model,
			options,
			contexts: Shelf::default(),
			reference_layouts: HashMap::new(),
		}
	}

	pub fn contexts_mut(&mut self) -> &mut Shelf<Vec<LocalContext>> {
		&mut self.contexts
	}

	pub fn propagate_context(&'a self, context_ref: Ref<LocalContext>) -> ParentContext {
		let context = self.contexts.get(context_ref).unwrap();
		if context.do_propagate {
			ParentContext::Intern(context_ref)
		} else {
			context.parent
		}
	}

	// pub fn contains(&self, term: &str, definition: &TermDefinition<M>) -> bool {
	// 	match self.terms.get(term) {
	// 		Some(d) => d == definition,
	// 		None => match self.parent {
	// 			Some(parent) => parent.contains(term, definition),
	// 			None => false,
	// 		},
	// 	}
	// }
}

impl<'a, V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M> ContextBuilder<'a, V, M> {
	pub fn is_type_property(&self, vocabulary: &V, property_ref: TId<treeldr::Property>) -> bool {
		let property = self.model.get(property_ref).unwrap();
		match property.id().as_iri() {
			Some(iri) => vocabulary.iri(iri).unwrap() == json_ld::rdf::RDF_TYPE,
			None => false,
		}
	}

	pub fn is_id_property(&self, property_ref: TId<treeldr::Property>) -> bool {
		let property = self.model.get(property_ref).unwrap();
		match property.id().as_iri() {
			Some(iri) => *iri == IriIndex::Iri(Term::TreeLdr(TreeLdr::Self_)),
			None => false,
		}
	}

	pub fn is_type_field(&self, vocabulary: &V, field_id: TId<treeldr::layout::Field>) -> bool {
		let field = self.model.get(field_id).unwrap();
		match field.as_layout_field().property() {
			Some(property_ref) => self.is_type_property(vocabulary, **property_ref),
			None => false,
		}
	}

	pub fn is_id_field(&self, field_id: TId<treeldr::layout::Field>) -> bool {
		let field = self.model.get(field_id).unwrap();
		match field.as_layout_field().property() {
			Some(property_ref) => self.is_id_property(**property_ref),
			None => false,
		}
	}

	pub fn context_eq(&self, a: Ref<LocalContext>, b: Ref<LocalContext>) -> bool {
		if a == b {
			true
		} else {
			let a = self.contexts.get(a).unwrap();
			let b = self.contexts.get(b).unwrap();

			a.terms.len() == b.terms.len()
				&& a.terms.iter().all(|(t, a)| {
					b.terms
						.get(t)
						.map(|b| a.context_eq(self, b))
						.unwrap_or(false)
				})
		}
	}

	pub fn context_extern_eq(
		&self,
		vocabulary: &V,
		a: &json_ld::syntax::context::Value<M>,
		b: Ref<LocalContext>,
	) -> bool {
		match a {
			json_ld::syntax::context::Value::One(a) => match a.value() {
				json_ld::syntax::Context::Definition(a) => {
					let b = self.contexts.get(b).unwrap();
					a.version.is_none()
						&& a.base.is_none() && a.vocab.is_none()
						&& a.import.is_none() && a.language.is_none()
						&& a.direction.is_none() && a.propagate.is_none()
						&& a.protected.is_none() && a.type_.is_none()
						&& a.bindings.len() == b.terms.len()
						&& a.bindings.iter().all(|(t, a)| {
							b.terms
								.get(t.as_str())
								.map(|b| b.eq_syntax(vocabulary, self, a.definition.value()))
								.unwrap_or(false)
						})
				}
				_ => false,
			},
			json_ld::syntax::context::Value::Many(_) => false,
		}
	}

	pub fn extern_term_definition_cmp(
		&self,
		vocabulary: &V,
		e: &json_ld::context::TermDefinition<
			IriIndex,
			BlankIdIndex,
			json_ld::syntax::context::Value<M>,
			M,
		>,
		def: &TermDefinition,
	) -> TermDefinitionCmp {
		match e {
			json_ld::context::TermDefinition::Type(_) => TermDefinitionCmp::Equal,
			json_ld::context::TermDefinition::Normal(n) => {
				if !n.prefix
					&& !n.reverse_property
					&& n.base_url.is_none()
					&& n.direction.is_none()
					&& n.index.is_none() && n.language.is_none()
					&& n.nest.is_none() && n.container == def.container
					&& n.value.as_ref() == Some(&def.id)
					&& n.typ == def.type_
				{
					match &n.context {
						Some(context) => {
							if self.context_extern_eq(vocabulary, context.value(), def.context) {
								TermDefinitionCmp::Equal
							} else {
								TermDefinitionCmp::ContextNeq
							}
						}
						None => {
							if self.is_context_empty(def.context) {
								TermDefinitionCmp::Equal
							} else {
								TermDefinitionCmp::ContextNeq
							}
						}
					}
				} else {
					TermDefinitionCmp::Neq
				}
			}
		}
	}

	pub fn insert_field(
		&mut self,
		vocabulary: &V,
		context_ref: Ref<LocalContext>,
		field_id: TId<treeldr::layout::Field>,
	) -> Result<(), Error> {
		let field = self.model.get(field_id).unwrap();
		match field.as_layout_field().property() {
			Some(property_ref) => {
				let term = field.name().expect("missing field name").to_string();
				let layout_ref = **field
					.as_formatted()
					.format()
					.as_ref()
					.expect("missing field layout");

				let is_type = self.is_type_property(vocabulary, **property_ref);
				let is_id = self.is_id_property(**property_ref);

				let id = if is_type {
					json_ld::Term::Keyword(Keyword::Type)
				} else if is_id {
					json_ld::Term::Keyword(Keyword::Id)
				} else {
					let property = self.model.get(**property_ref).unwrap();
					json_ld::Term::Ref(property.id().into())
				};

				let definition = TermDefinition {
					id,
					type_: self.generate_property_definition_type(layout_ref, !is_id && !is_type),
					container: self.generate_property_definition_container(layout_ref),
					context: self.generate_property_definition_context(
						vocabulary,
						context_ref,
						layout_ref,
					)?,
				};

				let context = self.contexts.get_mut(context_ref).unwrap();
				if let Some(pa) = context.insert(term, definition)? {
					pa.resolve(self)?;
				}

				Ok(())
			}
			None => Ok(()),
		}
	}

	pub fn insert_typed_layout(
		&mut self,
		vocabulary: &V,
		context_ref: Ref<LocalContext>,
		layout_ref: TId<treeldr::Layout>,
	) -> Result<(), Error> {
		let layout = self.model.get(layout_ref).unwrap();
		let ty = self.model.get(layout.as_layout().ty().unwrap()).unwrap();

		let term = layout.as_component().name().unwrap().to_string();

		let definition = TermDefinition {
			id: json_ld::Term::Ref(ty.id().into()),
			type_: None,
			container: json_ld::Container::None,
			context: self.generate_type_definition_context(vocabulary, context_ref, layout_ref)?,
		};

		let context = self.contexts.get_mut(context_ref).unwrap();
		if let Some(pa) = context.insert(term, definition)? {
			pa.resolve(self)?
		}

		Ok(())
	}

	/// Generate the `@type` entry of a term definition.
	fn generate_property_definition_type(
		&mut self,
		layout_ref: TId<treeldr::Layout>,
		generate_id_type: bool,
	) -> Option<json_ld::Type<IriIndex>> {
		let layout = self.model.get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.as_layout().description().value() {
			Description::Required(r) => {
				self.generate_property_definition_type(**r.item_layout(), generate_id_type)
			}
			Description::Option(o) => {
				self.generate_property_definition_type(**o.item_layout(), generate_id_type)
			}
			Description::Primitive(n) => Some(generate_primitive_type(n)),
			_ => {
				if generate_id_type
					&& self
						.model
						.can_be_reference_layout(&mut self.reference_layouts, layout_ref)
				{
					Some(json_ld::Type::Id)
				} else {
					None
				}
			}
		}
	}

	/// Generate the `@container` entry of a term definition.
	fn generate_property_definition_container(
		&self,
		layout_ref: TId<treeldr::Layout>,
	) -> json_ld::Container {
		let layout = self.model.get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.as_layout().description().value() {
			Description::Set(_) => json_ld::Container::Set,
			Description::Array(_) => json_ld::Container::List,
			_ => json_ld::Container::None,
		}
	}

	pub fn insert_layout_terms(
		&mut self,
		vocabulary: &V,
		context_ref: Ref<LocalContext>,
		layout_ref: TId<treeldr::Layout>,
		type_scoped_context: bool,
	) -> Result<(), Error> {
		let layout = self.model.get(layout_ref).unwrap();

		use treeldr::layout::Description;
		match layout.as_layout().description().value() {
			Description::Set(s) => self.insert_layout_terms(
				vocabulary,
				context_ref,
				**s.item_layout(),
				type_scoped_context,
			),
			Description::OneOrMany(s) => self.insert_layout_terms(
				vocabulary,
				context_ref,
				**s.item_layout(),
				type_scoped_context,
			),
			Description::Required(o) => self.insert_layout_terms(
				vocabulary,
				context_ref,
				**o.item_layout(),
				type_scoped_context,
			),
			Description::Option(o) => self.insert_layout_terms(
				vocabulary,
				context_ref,
				**o.item_layout(),
				type_scoped_context,
			),
			Description::Enum(e) => {
				for v_id in e.variants() {
					let v = self.model.get(**v_id).unwrap();
					if let Some(layout_ref) = v.as_formatted().format().as_ref() {
						self.insert_layout_terms(
							vocabulary,
							context_ref,
							**layout_ref,
							type_scoped_context,
						)?
					}
				}

				Ok(())
			}
			Description::Struct(s) => {
				if !type_scoped_context && self.options.rdf_type_to_layout_name {
					// check if there is a required `rdf:type` property field.
					for field_id in s.fields() {
						let field = self.model.get(**field_id).unwrap();
						if field.is_required(self.model)
							&& self.is_type_field(vocabulary, **field_id)
						{
							self.insert_field(vocabulary, context_ref, **field_id)?;
							self.insert_typed_layout(vocabulary, context_ref, layout_ref)?;

							return Ok(());
						}
					}
				}

				// otherwise, include the fields directly in this context.
				for field_id in s.fields() {
					self.insert_field(vocabulary, context_ref, **field_id)?
				}

				Ok(())
			}
			_ => Ok(()),
		}
	}

	fn is_context_empty(&self, context_ref: Ref<LocalContext>) -> bool {
		let context = self.contexts.get(context_ref).unwrap();
		context.is_empty()
	}

	/// Generate the `@context` entry of a property definition.
	fn generate_property_definition_context(
		&mut self,
		vocabulary: &V,
		current_context_ref: Ref<LocalContext>,
		layout_ref: TId<treeldr::Layout>,
	) -> Result<Ref<LocalContext>, Error> {
		let context_ref = self.contexts.insert(LocalContext::new(
			self.propagate_context(current_context_ref),
			true,
		));
		self.insert_layout_terms(vocabulary, context_ref, layout_ref, false)?;
		Ok(context_ref)
	}

	/// Generate the `@context` entry of a type definition.
	fn generate_type_definition_context(
		&mut self,
		vocabulary: &V,
		current_context_ref: Ref<LocalContext>,
		layout_ref: TId<treeldr::Layout>,
	) -> Result<Ref<LocalContext>, Error> {
		let context_ref = self.contexts.insert(LocalContext::new(
			self.propagate_context(current_context_ref),
			false,
		));
		self.insert_layout_terms(vocabulary, context_ref, layout_ref, true)?;
		Ok(context_ref)
	}

	pub async fn simplify<L>(
		&mut self,
		vocabulary: &mut V,
		loader: &mut L,
	) -> Result<(), SimplifyError<L::ContextError, M>>
	where
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex> + Send + Sync,
		L: ContextLoader<IriIndex, M> + Send + Sync,
		L::Context: Into<json_ld::syntax::context::Value<M>>,
		L::ContextError: Send,
		M: Clone + Send + Sync,
	{
		let genealogy = Genealogy::new(&self.contexts);

		let mut continue_simplifying = true;

		while continue_simplifying {
			continue_simplifying = false;
			for &root in &genealogy.roots {
				continue_simplifying |= self
					.simplify_context(vocabulary, loader, &genealogy, root)
					.await?;
			}
		}

		Ok(())
	}

	fn simplify_context<'f, L>(
		&'f mut self,
		vocabulary: &'f mut V,
		loader: &'f mut L,
		genealogy: &'f Genealogy,
		context_ref: Ref<LocalContext>,
	) -> BoxFuture<'f, Result<bool, SimplifyError<L::ContextError, M>>>
	where
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex> + Send + Sync,
		L: ContextLoader<IriIndex, M> + Send + Sync,
		L::Context: Into<json_ld::syntax::context::Value<M>>,
		L::ContextError: Send,
		M: Clone + Send + Sync,
	{
		async move {
			let mut continue_simplification = false;
			for &child in genealogy.children.get(context_ref).unwrap() {
				continue_simplification |= self
					.simplify_context(vocabulary, loader, genealogy, child)
					.await?;
			}

			let terms = {
				let context = self.contexts.get_mut(context_ref).unwrap();
				std::mem::take(&mut context.terms)
			};

			let context = self.contexts.get(context_ref).unwrap();
			let mut preserved_terms = BTreeMap::new();
			let mut moved_terms = Vec::new();

			for (term, definition) in terms {
				if let Some(parent_def) =
					context
						.parent
						.get(&self.options.context, &self.contexts, &term)
				{
					if parent_def.intern_cmp(vocabulary, self, &definition)
						== TermDefinitionCmp::Equal
					{
						// we can ignore this term.
						continue;
					}
				}

				let protected_definitions = find_protected_definitions(
					vocabulary,
					self.model,
					loader,
					&self.options.context,
					&term,
					true,
				)
				.await
				.map_err(SimplifyError::ContextProcessing)?;

				if protected_definitions.is_empty() {
					preserved_terms.insert(term, definition);
				} else {
					for (protected_definition, active_context) in protected_definitions {
						match self.extern_term_definition_cmp(
							vocabulary,
							&protected_definition,
							&definition,
						) {
							TermDefinitionCmp::Equal => (),
							TermDefinitionCmp::ContextNeq => {
								// we must not redefine the term.
								// let parent_definition_context = parent_definition.context(&self.contexts);
								let context = self.contexts.get(definition.context).unwrap();
								match protected_definition.context() {
									Some(local_context) => {
										use json_ld::Process;
										let active_context = local_context
											.process_with(
												vocabulary,
												&active_context,
												loader,
												active_context.base_iri().cloned(),
												json_ld::context_processing::Options::default(),
											)
											.await
											.map_err(SimplifyError::ContextProcessing)?;

										for (t, def) in &context.terms {
											match active_context.get(t.as_str()) {
												Some(parent_def) => {
													if TermDefinitionRef::Extern(parent_def)
														.intern_cmp(vocabulary, self, def)
														!= TermDefinitionCmp::Equal
													{
														return Err(SimplifyError::ProtectedTermRedefinition(term.clone()));
													}
												}
												None => moved_terms.push((t.clone(), def.clone())),
											}
										}
									}
									None => {
										for (t, def) in &context.terms {
											moved_terms.push((t.clone(), def.clone()))
										}
									}
								}
							}
							TermDefinitionCmp::Neq => {
								// we cannot define this term!
								return Err(SimplifyError::ProtectedTermRedefinition(term.clone()));
							}
						}
					}
				}
			}

			let context = self.contexts.get_mut(context_ref).unwrap();
			context.terms = preserved_terms;

			continue_simplification |= !moved_terms.is_empty();
			for (term, definition) in moved_terms {
				let context = self.contexts.get_mut(context_ref).unwrap();
				if let Some(pa) = context
					.insert(term, definition)
					.map_err(SimplifyError::Error)?
				{
					pa.resolve(self).map_err(SimplifyError::Error)?
				}
			}

			Ok(continue_simplification)
		}
		.boxed()
	}

	pub fn build(
		&self,
		vocabulary: &V,
		context_ref: Ref<LocalContext>,
	) -> Result<json_ld::syntax::context::Value<()>, Error> {
		let mut definition = json_ld::syntax::context::Definition::new();

		let context = self.contexts.get(context_ref).unwrap();
		for (term, term_definition) in &context.terms {
			if !matches!(term.as_str(), "@type" | "@id") {
				definition.bindings.insert(
					Meta(term.clone().into(), ()),
					Meta(term_definition.build(vocabulary, self)?, ()),
				);
			}
		}

		Ok(json_ld::syntax::context::Value::One(Meta(
			json_ld::syntax::Context::Definition(definition),
			(),
		)))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermDefinitionCmp {
	Equal,
	ContextNeq,
	Neq,
}

pub struct Genealogy {
	roots: Vec<Ref<LocalContext>>,
	children: shelves::Map<LocalContext, Vec<Vec<Ref<LocalContext>>>>,
}

impl Genealogy {
	fn new(contexts: &Shelf<Vec<LocalContext>>) -> Self {
		let mut children: shelves::Map<LocalContext, Vec<Vec<Ref<LocalContext>>>> =
			shelves::Map::new(contexts.iter().map(|_| Vec::new()).collect());
		let mut roots = Vec::new();

		for (context_ref, context) in contexts {
			match context.parent {
				ParentContext::Extern => roots.push(context_ref),
				ParentContext::Intern(parent_ref) => {
					children.get_mut(parent_ref).unwrap().push(context_ref)
				}
			}
		}

		Self { roots, children }
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

#[derive(Debug)]
pub enum SimplifyError<E, M> {
	Error(Error),
	ProtectedTermRedefinition(String),
	ContextProcessing(Meta<json_ld::context_processing::Error<E>, M>),
}

impl<E, M> fmt::Display for SimplifyError<E, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Error(e) => e.fmt(f),
			Self::ProtectedTermRedefinition(term) => {
				write!(f, "protected `{term}` term redefinition")
			}
			Self::ContextProcessing(_) => "unable to process context".fmt(f),
		}
	}
}

#[derive(Debug)]
pub enum GenerateError<E, M> {
	Error(Error),
	Simplify(SimplifyError<E, M>),
}

impl<E, M> fmt::Display for GenerateError<E, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Error(e) => e.fmt(f),
			Self::Simplify(e) => e.fmt(f),
		}
	}
}

/// Generate a JSON-LD context from a TreeLDR model.
pub async fn generate<V, L, M>(
	vocabulary: &mut V,
	loader: &mut L,
	model: &treeldr::Model<M>,
	options: Options<V, M>,
	layouts: &[TId<treeldr::Layout>],
) -> Result<json_ld::syntax::context::Value<()>, GenerateError<L::ContextError, M>>
where
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex> + Send + Sync,
	L: ContextLoader<IriIndex, M> + Send + Sync,
	L::Context: Into<json_ld::syntax::context::Value<M>>,
	L::ContextError: Send,
	M: Clone + Send + Sync,
{
	let mut builder = ContextBuilder::new(model, &options);
	let context_ref = builder
		.contexts_mut()
		.insert(LocalContext::new(ParentContext::Extern, true));

	for layout_ref in layouts {
		builder
			.insert_layout_terms(vocabulary, context_ref, *layout_ref, false)
			.map_err(GenerateError::Error)?;
	}

	builder
		.simplify(vocabulary, loader)
		.await
		.map_err(GenerateError::Simplify)?;
	builder
		.build(vocabulary, context_ref)
		.map_err(GenerateError::Error)
}

fn generate_primitive_type<M>(
	n: &treeldr::layout::primitive::Restricted<M>,
) -> json_ld::Type<IriIndex> {
	json_ld::Type::Ref(*n.primitive().id().as_iri().unwrap())
}
