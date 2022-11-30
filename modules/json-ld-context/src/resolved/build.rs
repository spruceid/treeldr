use std::{
	cmp::Ordering,
	collections::{btree_map, BTreeMap},
	ops::{Deref, DerefMut},
};

use contextual::WithContext;
use json_ld::{
	syntax::{context::term_definition, Entry},
	Nullable,
};
use locspan::Meta;
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex};

use super::{
	CompareTermDefinition, LocalContext, LocalContextComparison, LocalContexts, TermDefinition,
	TermDefinitionOrdering,
};

use crate::Error;

impl TermDefinition {
	pub fn build(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		local_contexts: &LocalContexts,
		context_comparison: &LocalContextComparison,
	) -> Result<Nullable<json_ld::syntax::context::TermDefinition<()>>, Error> {
		let mut definition = json_ld::syntax::context::term_definition::Expanded::new();

		definition.id = self.build_id(vocabulary);
		definition.type_ = self.build_type(vocabulary);
		definition.container = self.build_container();

		definition.context = local_contexts
			.build(vocabulary, context_comparison, self.context)?
			.map(|c| Entry::new((), Meta(Box::new(c), ())));

		Ok(definition.simplify())
	}

	pub fn build_container(&self) -> Option<Entry<Nullable<json_ld::syntax::Container<()>>, ()>> {
		let c = match self.container? {
			Nullable::Null => Meta(Nullable::Null, ()),
			Nullable::Some(c) => c.into_syntax(())?.map(Nullable::Some),
		};

		Some(Entry::new((), c))
	}

	pub fn build_id<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &V,
	) -> Option<Entry<Nullable<term_definition::Id>, ()>> {
		let syntax_id = match self.id.clone()? {
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
		let syntax_type = self.type_.clone()?.map(|t| match t {
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
				term_definition::Type::Term(vocabulary.iri(&i).unwrap().to_string())
			}
		});

		Some(Entry::new((), Meta(syntax_type, ())))
	}
}

impl LocalContext {
	pub fn build(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		local_contexts: &LocalContexts,
		context_comparison: &LocalContextComparison,
		bindings: BuiltDefinitions,
	) -> Result<Option<json_ld::syntax::context::Value<()>>, Error> {
		if bindings.is_empty() {
			Ok(None)
		} else {
			let mut definition = json_ld::syntax::context::Definition::new();

			for (term, term_definition) in bindings.iter() {
				if !matches!(term.as_str(), "@type" | "@id") {
					definition.bindings.insert(
						Meta(term.clone().into(), ()),
						Meta(
							term_definition.build(
								vocabulary,
								local_contexts,
								context_comparison,
							)?,
							(),
						),
					);
				}
			}

			Ok(Some(json_ld::syntax::context::Value::One(Meta(
				json_ld::syntax::Context::Definition(definition),
				(),
			))))
		}
	}
}

impl LocalContexts {
	fn overridden_definitions<'a>(
		&'a self,
		context_comparison: &LocalContextComparison<'a>,
		r: Ref<LocalContext>,
		term: &str,
	) -> Vec<&'a TermDefinition> {
		let mut result = Vec::new();

		for p in self.relations.parents(r) {
			for (t, defs) in context_comparison.accessible_bindings(p).unwrap() {
				for def in *defs {
					if *t == term {
						result.push(def)
					}

					if self.contexts.get(def.context).unwrap().type_scoped {
						if let Some(bindings) = self.definitions(def.context) {
							for (t, defs) in bindings {
								if *t == term {
									result.extend(defs.added.iter())
								}
							}
						}
					}
				}
			}
		}

		result
	}

	pub fn build_definitions(
		&self,
		context_comparison: &LocalContextComparison,
		r: Ref<LocalContext>,
	) -> BuiltDefinitions {
		let mut result = BuiltDefinitions::default();

		if let Some(bindings) = self.definitions.get(&r) {
			for (term, defs) in bindings {
				for def in &defs.added {
					result.insert(self, context_comparison, r, term.to_string(), def)
				}
			}
		}

		result
	}

	pub fn build(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		context_comparison: &LocalContextComparison,
		r: Ref<LocalContext>,
	) -> Result<Option<json_ld::syntax::context::Value<()>>, Error> {
		let bindings = self.build_definitions(context_comparison, r);
		let context = self.contexts.get(r).unwrap();
		let r = context.build(vocabulary, self, context_comparison, bindings);
		r
	}
}

#[derive(Default)]
pub struct BuiltDefinitions(BTreeMap<String, TermDefinition>);

impl BuiltDefinitions {
	pub fn insert(
		&mut self,
		local_contexts: &LocalContexts,
		context_comparison: &LocalContextComparison,
		this: Ref<LocalContext>,
		term: String,
		def: &TermDefinition,
	) {
		match self.0.entry(term) {
			btree_map::Entry::Occupied(mut e) => {
				match e.get().compare(context_comparison, def).ordering {
					Some(Ordering::Greater | Ordering::Equal) => {
						// we can skip it.
					}
					Some(Ordering::Less) => {
						e.insert(def.clone());
					}
					None => {
						panic!("ambiguous term `{}`", e.key())
					}
				}
			}
			btree_map::Entry::Vacant(e) => {
				let mut must_override = false;
				let mut parent_defined = false;

				let overridden =
					local_contexts.overridden_definitions(context_comparison, this, e.key());
				for parent_def in overridden {
					parent_defined = true;
					let cmp = parent_def.compare(context_comparison, def);

					match cmp {
						TermDefinitionOrdering {
							ordering: Some(Ordering::Greater | Ordering::Equal),
							..
						} => {
							// skip term
						}
						TermDefinitionOrdering {
							header_ordering, ..
						} => {
							// The term must be defined.
							if parent_def.protected {
								match header_ordering {
									Some(Ordering::Greater | Ordering::Equal) => {
										// move the context terms out.
										if let Some(bindings) =
											local_contexts.definitions.get(&def.context)
										{
											for (term, defs) in bindings {
												for def in &defs.added {
													let move_out = local_contexts
														.definitions(parent_def.context)
														.unwrap()
														.get(term.as_str())
														.is_none();
													if move_out {
														self.insert(
															local_contexts,
															context_comparison,
															this,
															term.clone(),
															def,
														)
													}
												}
											}
										}

										return;
									}
									_ => {
										panic!("protected term redefinition")
									}
								}
							}

							must_override = true;
						}
					}
				}

				if !parent_defined || must_override {
					e.insert(def.clone());
				}
			}
		}
	}
}

impl Deref for BuiltDefinitions {
	type Target = BTreeMap<String, TermDefinition>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for BuiltDefinitions {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
