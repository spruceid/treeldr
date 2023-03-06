use std::collections::{BTreeSet, HashMap, HashSet};

use json_ld::{syntax::Keyword, ContextLoader, Nullable};
use locspan::Meta;
use rdf_types::{IriVocabulary, VocabularyMut};
use shelves::Ref;
use treeldr::{
	layout::Description,
	vocab::{self, Term, TreeLdr},
	BlankIdIndex, IriIndex, MutableModel, PropertyValueRef, TId,
};
use unresolved::Unresolved;

pub mod command;
mod import;
mod resolved;
mod unresolved;

pub use command::Command;

#[derive(Debug)]
pub enum Error {
	//
}

/// Generator options.
pub struct Options<M> {
	pub rdf_type_to_layout_name: bool,
	pub flatten: bool,
	pub prefixes: HashMap<String, IriIndex>,
	pub context: json_ld::Context<IriIndex, BlankIdIndex, json_ld::syntax::context::Value<M>, M>,
}

pub struct Builder<'a, V, M> {
	vocabulary: &'a V,
	model: &'a MutableModel<M>,
	options: Options<M>,
	reference_layouts: HashMap<TId<treeldr::Layout>, bool>,
}

impl<'a, V, M> Builder<'a, V, M> {
	pub fn new(vocabulary: &'a V, model: &'a MutableModel<M>, options: Options<M>) -> Self {
		Self {
			model,
			vocabulary,
			options,
			reference_layouts: HashMap::new(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IncludedLayout {
	pub id: TId<treeldr::Layout>,
	pub type_scoped: bool,
}

impl IncludedLayout {
	pub fn new(id: TId<treeldr::Layout>, type_scoped: bool) -> Self {
		Self { id, type_scoped }
	}

	fn flatten<M>(
		&self,
		model: &MutableModel<M>,
		options: &Options<M>,
		result: &mut HashSet<IncludedLayout>,
	) {
		if result.insert(*self) {
			let layout = model.get(self.id).unwrap();

			match layout.as_layout().description() {
				Description::Never
				| Description::Primitive(_)
				| Description::Derived(_)
				| Description::Reference(_) => (),
				Description::Alias(id) => IncludedLayout::new(*id.value(), self.type_scoped)
					.flatten(model, options, result),
				Description::Required(r) => {
					IncludedLayout::new(**r.item_layout(), self.type_scoped)
						.flatten(model, options, result)
				}
				Description::Option(o) => IncludedLayout::new(**o.item_layout(), self.type_scoped)
					.flatten(model, options, result),
				Description::OneOrMany(o) => {
					IncludedLayout::new(**o.item_layout(), self.type_scoped)
						.flatten(model, options, result)
				}
				Description::Set(s) => IncludedLayout::new(**s.item_layout(), self.type_scoped)
					.flatten(model, options, result),
				Description::Array(a) => IncludedLayout::new(**a.item_layout(), self.type_scoped)
					.flatten(model, options, result),
				Description::Enum(e) => {
					for vid in e.variants() {
						let v = model.get(**vid).unwrap();
						if let Some(layout_id) = v.as_formatted().format().as_ref() {
							IncludedLayout::new(*layout_id, self.type_scoped)
								.flatten(model, options, result)
						}
					}
				}
				Description::Struct(s) => {
					if options.flatten {
						for fid in s.fields() {
							let field = model.get(**fid).unwrap();
							if let Some(layout_id) = field.as_formatted().format().as_ref() {
								IncludedLayout::new(*layout_id, self.type_scoped)
									.flatten(model, options, result)
							}
						}
					}
				}
			}
		}
	}

	pub fn compute_definitions<V: IriVocabulary<Iri = IriIndex>, M>(
		&self,
		builder: &mut Builder<V, M>,
		local_contexts: &mut unresolved::LocalContexts,
		layout_contexts: &mut LayoutLocalContexts,
		bindings: &mut unresolved::Bindings,
		parent: Ref<unresolved::LocalContext>,
	) {
		let layout = builder.model.get(self.id).unwrap();

		if let Description::Struct(s) = layout.as_layout().description() {
			if !self.type_scoped && builder.options.rdf_type_to_layout_name {
				// check if there is a `rdf:type` property field.
				for fid in s.fields() {
					if builder.is_type_field(**fid) {
						bindings.insert_field(
							builder,
							local_contexts,
							layout_contexts,
							parent,
							**fid,
						);
						bindings.insert_typed_layout(
							builder,
							local_contexts,
							layout_contexts,
							parent,
							self.id,
						);

						let field = builder.model.get(**fid).unwrap();
						if !builder.options.flatten && field.is_required(builder.model) {
							// if it is required then we don't need to
							// include the other layout fields here.
							return;
						} else {
							break;
						}
					}
				}
			}

			for fid in s.fields() {
				if !self.type_scoped || !builder.is_type_field(**fid) {
					bindings.insert_field(builder, local_contexts, layout_contexts, parent, **fid)
				}
			}
		}
	}
}

impl unresolved::Bindings {
	fn insert_field<V: IriVocabulary<Iri = IriIndex>, M>(
		&mut self,
		builder: &mut Builder<V, M>,
		local_contexts: &mut unresolved::LocalContexts,
		layout_contexts: &mut LayoutLocalContexts,
		parent: Ref<unresolved::LocalContext>,
		fid: TId<treeldr::layout::Field>,
	) {
		let f = builder.model.get(fid).unwrap();

		let term = f.as_component().name().unwrap().to_string();

		if term == "@type" {
			match f.as_layout_field().property() {
				Some(property_ref) if builder.is_type_property(*property_ref) => {
					let layout_id = f.as_formatted().format().unwrap();

					match builder.generate_property_definition_container(layout_id) {
						Some(Nullable::Some(json_ld::Container::Set)) => {
							let definition = unresolved::TermDefinition {
								container: builder
									.generate_property_definition_container(layout_id),
								context: local_contexts.empty_context(),
								..Default::default()
							};

							self.insert(term, Nullable::Some(definition));
						}
						Some(_) => panic!("invalid `@type` container"),
						None => (),
					}
				}
				_ => {
					panic!("`@type` is not a type field")
				}
			}
		} else if let Some(property_ref) = f.as_layout_field().property() {
			let property = builder.model.get(*property_ref).unwrap();
			let is_type = builder.is_type_property(*property_ref);
			let is_id = builder.is_id_property(*property_ref);

			let id = if is_type {
				json_ld::Term::Keyword(Keyword::Type)
			} else if is_id {
				json_ld::Term::Keyword(Keyword::Id)
			} else {
				json_ld::Term::Id(property.id().into())
			};

			let layout_id = f.as_formatted().format().unwrap();

			let datatype = property
				.as_property()
				.range()
				.iter()
				.find(|v| {
					builder
						.model
						.get(*v.value.into_value())
						.unwrap()
						.is_datatype(builder.model)
				})
				.or_else(|| property.as_property().range().first())
				.map(PropertyValueRef::into_value)
				.map(Meta::into_value)
				.cloned()
				.unwrap();

			let definition = unresolved::TermDefinition {
				id: Some(Unresolved::Resolved(id)),
				type_: builder
					.generate_property_definition_type(layout_id, datatype, !is_id && !is_type)
					.map(Unresolved::Resolved)
					.map(Nullable::Some),
				container: builder.generate_property_definition_container(layout_id),
				context: layout_contexts.insert(
					builder,
					local_contexts,
					parent,
					[layout_id],
					false,
				),
				..Default::default()
			};

			self.insert(term, Nullable::Some(definition));
		}
	}

	fn insert_typed_layout<V: IriVocabulary<Iri = IriIndex>, M>(
		&mut self,
		builder: &mut Builder<V, M>,
		local_contexts: &mut unresolved::LocalContexts,
		layout_contexts: &mut LayoutLocalContexts,
		parent: Ref<unresolved::LocalContext>,
		layout_id: TId<treeldr::Layout>,
	) {
		let layout = builder.model.get(layout_id).unwrap();

		let term = layout.as_component().name().unwrap().to_string();

		let definition = unresolved::TermDefinition {
			id: layout
				.as_layout()
				.ty()
				.map(|id| Unresolved::Resolved(json_ld::Term::Id(id.id().into()))),
			context: layout_contexts.insert(builder, local_contexts, parent, [layout_id], true),
			..Default::default()
		};

		self.insert(term, Nullable::Some(definition));
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutLocalContext {
	parent: Ref<unresolved::LocalContext>,
	layouts: BTreeSet<IncludedLayout>,
	type_scoped: bool,
}

impl LayoutLocalContext {
	pub fn propagate(&self, this: Ref<unresolved::LocalContext>) -> Ref<unresolved::LocalContext> {
		if self.type_scoped {
			self.parent
		} else {
			this
		}
	}

	pub fn compute_definitions<V: IriVocabulary<Iri = IriIndex>, M>(
		&self,
		this: Ref<unresolved::LocalContext>,
		builder: &mut Builder<V, M>,
		local_contexts: &mut unresolved::LocalContexts,
		layout_contexts: &mut LayoutLocalContexts,
	) -> unresolved::Bindings {
		let mut bindings = unresolved::Bindings::new();

		let propagated = self.propagate(this);

		for layout in &self.layouts {
			layout.compute_definitions(
				builder,
				local_contexts,
				layout_contexts,
				&mut bindings,
				propagated,
			)
		}

		bindings
	}
}

#[derive(Default)]
pub struct LayoutLocalContexts {
	forward: HashMap<Ref<unresolved::LocalContext>, LayoutLocalContext>,
	backward: HashMap<LayoutLocalContext, Ref<unresolved::LocalContext>>,
}

impl LayoutLocalContexts {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn contains_layout(
		&self,
		r: Ref<unresolved::LocalContext>,
		layout: &IncludedLayout,
	) -> bool {
		self.forward
			.get(&r)
			.map(|l| l.layouts.contains(layout) || self.contains_layout(l.parent, layout))
			.unwrap_or(false)
	}

	pub fn insert<V: IriVocabulary<Iri = IriIndex>, M>(
		&mut self,
		builder: &mut Builder<V, M>,
		local_contexts: &mut unresolved::LocalContexts,
		parent: Ref<unresolved::LocalContext>,
		layouts: impl IntoIterator<Item = TId<treeldr::Layout>>,
		type_scoped: bool,
	) -> Ref<unresolved::LocalContext> {
		let layouts = flatten_layouts(
			builder.model,
			&builder.options,
			layouts
				.into_iter()
				.map(|id| IncludedLayout::new(id, type_scoped)),
		);

		let context = LayoutLocalContext {
			parent,
			layouts: layouts
				.into_iter()
				.filter(|l| !self.contains_layout(parent, l))
				.collect(),
			type_scoped,
		};

		let mut define = false;

		let r = *self
			.backward
			.entry(context.clone())
			.or_insert_with_key(|context| {
				let r = local_contexts.insert(unresolved::LocalContext::default());
				self.forward.insert(r, context.clone());
				define = true;
				r
			});

		if define {
			let bindings = context.compute_definitions(r, builder, local_contexts, self);
			local_contexts.set_bindings(r, bindings);
		}

		r
	}
}

fn flatten_layouts<M>(
	model: &MutableModel<M>,
	options: &Options<M>,
	layouts: impl IntoIterator<Item = IncludedLayout>,
) -> HashSet<IncludedLayout> {
	let mut result = HashSet::new();

	for layout in layouts {
		layout.flatten(model, options, &mut result)
	}

	result
}

impl<'a, V, M> Builder<'a, V, M> {
	pub fn is_id_property(&self, property_ref: TId<treeldr::Property>) -> bool {
		let property = self.model.get(property_ref).unwrap();
		match property.id().as_iri() {
			Some(iri) => *iri == IriIndex::Iri(Term::TreeLdr(TreeLdr::Self_)),
			None => false,
		}
	}

	/// Generate the `@type` entry of a term definition.
	fn generate_property_definition_type(
		&mut self,
		layout_ref: TId<treeldr::Layout>,
		type_ref: TId<treeldr::Type>,
		generate_id_type: bool,
	) -> Option<json_ld::Type<IriIndex>> {
		let layout = self.model.get(layout_ref).unwrap();
		match layout.as_layout().description() {
			Description::Alias(a) => {
				self.generate_property_definition_type(*a.value(), type_ref, generate_id_type)
			}
			Description::Required(r) => self.generate_property_definition_type(
				**r.item_layout(),
				type_ref,
				generate_id_type,
			),
			Description::Option(o) => self.generate_property_definition_type(
				**o.item_layout(),
				type_ref,
				generate_id_type,
			),
			Description::Array(a) => self.generate_property_definition_type(
				**a.item_layout(),
				type_ref,
				generate_id_type,
			),
			Description::Set(s) => self.generate_property_definition_type(
				**s.item_layout(),
				type_ref,
				generate_id_type,
			),
			Description::OneOrMany(o) => self.generate_property_definition_type(
				**o.item_layout(),
				type_ref,
				generate_id_type,
			),
			Description::Primitive(p) => match p {
				vocab::Primitive::Iri | vocab::Primitive::Uri | vocab::Primitive::Url => {
					Some(json_ld::Type::Id)
				}
				_ => match type_ref.id() {
					treeldr::Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::String))) => None,
					treeldr::Id::Iri(iri) => Some(json_ld::Type::Iri(iri)),
					_ => None,
				},
			},
			Description::Derived(p) => match p.primitive() {
				vocab::Primitive::Iri | vocab::Primitive::Uri | vocab::Primitive::Url => {
					Some(json_ld::Type::Id)
				}
				_ => match type_ref.id() {
					treeldr::Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::String))) => None,
					treeldr::Id::Iri(iri) => Some(json_ld::Type::Iri(iri)),
					_ => None,
				},
			},
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
	) -> Option<Nullable<json_ld::Container>> {
		let layout = self.model.get(layout_ref).unwrap();
		match layout.as_layout().description() {
			Description::Set(_) => Some(Nullable::Some(json_ld::Container::Set)),
			Description::Array(_) => Some(Nullable::Some(json_ld::Container::List)),
			_ => None,
		}
	}
}

impl<'a, V: IriVocabulary<Iri = IriIndex>, M> Builder<'a, V, M> {
	pub fn is_type_property(&self, property_ref: TId<treeldr::Property>) -> bool {
		let property = self.model.get(property_ref).unwrap();
		match property.id().as_iri() {
			Some(iri) => self.vocabulary.iri(iri).unwrap() == json_ld::rdf::RDF_TYPE,
			None => false,
		}
	}

	pub fn is_type_field(&self, field_id: TId<treeldr::layout::Field>) -> bool {
		let field = self.model.get(field_id).unwrap();
		match field.as_layout_field().property() {
			Some(property_ref) => self.is_type_property(*property_ref),
			None => false,
		}
	}

	// pub fn is_id_field(&self, field_id: TId<treeldr::layout::Field>) -> bool {
	// 	let field = self.model.get(field_id).unwrap();
	// 	match field.as_layout_field().property() {
	// 		Some(property_ref) => self.is_id_property(**property_ref),
	// 		None => false,
	// 	}
	// }
}

/// Generate a JSON-LD context from a TreeLDR model.
pub async fn generate<V, L, M>(
	vocabulary: &mut V,
	_loader: &mut L,
	model: &treeldr::MutableModel<M>,
	options: Options<M>,
	layouts: &[TId<treeldr::Layout>],
) -> Result<json_ld::syntax::context::Value<()>, Error>
where
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex> + Send + Sync,
	L: ContextLoader<IriIndex, M> + Send + Sync,
	L::Context: Into<json_ld::syntax::context::Value<M>>,
	L::ContextError: Send,
	M: Clone + Send + Sync,
{
	let mut builder = Builder::new(vocabulary, model, options);
	let mut local_contexts = unresolved::LocalContexts::new();

	let base_context = local_contexts.import(&builder.options.context);

	let mut layout_contexts = LayoutLocalContexts::new();
	let context_ref = layout_contexts.insert(
		&mut builder,
		&mut local_contexts,
		base_context,
		layouts.iter().copied(),
		false,
	);

	local_contexts.add_iri_prefixes(&builder.options.prefixes, context_ref);
	local_contexts.set_base_context(base_context, Some(context_ref));

	let prefixes = builder.options.prefixes.clone().into();

	let local_contexts = local_contexts.resolve(vocabulary);

	let accessible_bindings = local_contexts.compute_accessible_bindings();
	let context_comparison =
		resolved::LocalContextComparison::new(&local_contexts, accessible_bindings);

	log::debug!("building...");
	let result = local_contexts
		.build(
			vocabulary,
			&prefixes,
			&context_comparison,
			context_ref.cast(),
		)?
		.unwrap_or_else(|| {
			json_ld::syntax::context::Value::One(Meta(
				json_ld::syntax::Context::Definition(json_ld::syntax::context::Definition::new()),
				(),
			))
		});

	log::debug!("done.");
	Ok(result)
}
