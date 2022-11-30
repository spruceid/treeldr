use json_ld::{syntax::context::definition::Version, LenientLanguageTag, Nullable};
use locspan::Meta;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex};

use crate::unresolved::{self, Bindings, Unresolved};

use unresolved::{LocalContext, TermDefinition};

impl unresolved::LocalContexts {
	pub fn import_local_contexts<M>(
		&mut self,
		ld: &json_ld::syntax::context::Value<M>,
		type_scoped: bool,
	) -> Ref<LocalContext> {
		match ld {
			json_ld::syntax::context::Value::One(context) => {
				self.import_local_context(context, type_scoped)
			}
			json_ld::syntax::context::Value::Many(contexts) => match contexts.len() {
				0 => self.empty_context(),
				1 => self.import_local_context(&contexts[0], type_scoped),
				_ => todo!("multiple local contexts"),
			},
		}
	}

	pub fn import_local_context<M>(
		&mut self,
		context: &json_ld::syntax::Context<json_ld::syntax::context::Definition<M>>,
		type_scoped: bool,
	) -> Ref<LocalContext> {
		match context {
			json_ld::syntax::Context::Definition(def) => {
				self.import_local_context_definition(def, type_scoped)
			}
			json_ld::syntax::Context::IriRef(_) => {
				todo!("import local context")
			}
			json_ld::syntax::Context::Null => {
				todo!("null local context")
			}
		}
	}

	pub fn import_local_context_definition<M>(
		&mut self,
		context: &json_ld::syntax::context::Definition<M>,
		type_scoped: bool,
	) -> Ref<LocalContext> {
		if let Some(version) = &context.version {
			if !matches!(version.value(), Version::V1_1) {
				panic!("unsupported version")
			}
		}

		if context.import.is_some() {
			panic!("unsupported imports")
		}

		let base = context.base.as_ref().map(|e| {
			e.value()
				.as_ref()
				.map(|v| Unresolved::Unresolved(v.clone()))
		});
		let vocab = context.vocab.as_ref().map(|e| {
			e.value()
				.as_ref()
				.map(|v| Unresolved::Unresolved(v.as_str().to_string()))
		});

		let protected = context
			.protected
			.as_ref()
			.map(|e| *e.value())
			.unwrap_or(false);

		let local = LocalContext {
			base,
			vocab,
			language: context.language.as_ref().map(|e| e.value.clone()),
			direction: context.direction.as_ref().map(|e| *e.value),
			propagate: true,
			type_scoped,
		};

		let r = self.insert(local);

		let mut bindings = Bindings::new();

		if let Some(type_) = &context.type_ {
			bindings.insert(
				"@type".to_string(),
				Nullable::Some(TermDefinition {
					container: Some(Nullable::Some(json_ld::Container::Set)),
					protected: protected
						|| type_.protected.as_ref().map(|e| *e.value).unwrap_or(false),
					context: self.empty_context(),
					..Default::default()
				}),
			);
		}

		for (key, def) in context.bindings.iter() {
			bindings.insert(
				key.to_string(),
				def.definition.value().as_ref().map(|d| match d {
					json_ld::syntax::context::TermDefinition::Simple(id) => TermDefinition {
						id: Some(Unresolved::Unresolved(id.as_str().to_string())),
						protected,
						context: self.empty_context(),
						..Default::default()
					},
					json_ld::syntax::context::TermDefinition::Expanded(def) => {
						let mut id = def.id.as_ref().map(|e| match e.value() {
							Nullable::Null => Unresolved::Resolved(json_ld::Term::Null),
							Nullable::Some(v) => Unresolved::Unresolved(v.as_str().to_string()),
						});
						let mut reverse = false;

						let reverse_id = def
							.reverse
							.as_ref()
							.map(|v| Unresolved::Unresolved(v.as_str().to_string()));

						if reverse_id.is_some() {
							if id.is_some() {
								panic!("reverse property cannot have id")
							} else {
								id = reverse_id;
								reverse = true
							}
						}

						fn import_container<M>(
							c: &json_ld::syntax::Container<M>,
						) -> json_ld::Container {
							match c {
								json_ld::syntax::Container::One(c) => (*c).into(),
								json_ld::syntax::Container::Many(m) => {
									json_ld::Container::from(m.iter().map(Meta::value))
										.expect("invalid container")
								}
							}
						}

						let type_scoped = type_term(key.as_str());

						TermDefinition {
							id,
							type_: def.type_.as_ref().map(|e| {
								e.value()
									.as_ref()
									.map(|v| Unresolved::Unresolved(v.clone()))
							}),
							container: def
								.container
								.as_ref()
								.map(|e| e.value().as_ref().map(|v| import_container(v))),
							index: def.index.as_ref().map(|e| e.value().clone()),
							language: def
								.language
								.as_ref()
								.map(|e| e.value().as_ref().map(|v| v.to_owned())),
							direction: def.direction.as_ref().map(|e| *e.value()),
							protected: def
								.protected
								.as_ref()
								.map(|e| *e.value())
								.unwrap_or(protected),
							prefix: def.prefix.as_ref().map(|e| *e.value()).unwrap_or(false),
							reverse,
							nest: def.nest.as_ref().map(|e| e.value().clone()),
							context: def
								.context
								.as_ref()
								.map(|e| self.import_local_contexts(e.value(), type_scoped))
								.unwrap_or_else(|| self.empty_context()),
						}
					}
				}),
			);
		}

		self.set_bindings(r, bindings);

		r
	}

	pub fn import<M>(
		&mut self,
		context: &json_ld::Context<IriIndex, BlankIdIndex, json_ld::syntax::context::Value<M>, M>,
	) -> Ref<LocalContext> {
		let local = LocalContext {
			base: context
				.base_iri()
				.cloned()
				.map(Unresolved::Resolved)
				.map(Nullable::Some),
			vocab: context
				.vocabulary()
				.cloned()
				.map(Unresolved::Resolved)
				.map(Nullable::Some),
			language: context
				.default_language()
				.map(LenientLanguageTag::to_owned)
				.map(Nullable::Some),
			direction: context.default_base_direction().map(Nullable::Some),
			propagate: true,
			type_scoped: false,
		};

		let r = self.insert(local);

		let mut bindings = Bindings::new();

		for b in context.definitions() {
			match b {
				json_ld::context::BindingRef::Normal(key, def) => {
					let type_scoped = type_term(key.as_str());

					bindings.insert(
						key.to_string(),
						Nullable::Some(TermDefinition {
							id: def.value.clone().map(Unresolved::Resolved),
							type_: def
								.typ
								.clone()
								.map(Unresolved::Resolved)
								.map(Nullable::Some),
							container: if def.container.is_empty() {
								None
							} else {
								Some(Nullable::Some(def.container))
							},
							protected: def.protected,
							index: def.index.as_ref().map(|e| e.value.clone()),
							language: def.language.clone(),
							direction: def.direction,
							prefix: def.prefix,
							reverse: def.reverse_property,
							nest: def.nest.as_ref().map(|e| e.value.clone()),
							context: def
								.context
								.as_ref()
								.map(|e| self.import_local_contexts(e.value(), type_scoped))
								.unwrap_or_else(|| self.empty_context()),
						}),
					);
				}
				json_ld::context::BindingRef::Type(def) => {
					bindings.insert(
						"@type".into(),
						Nullable::Some(TermDefinition {
							container: Some(Nullable::Some(def.container.into())),
							protected: def.protected,
							context: self.empty_context(),
							..Default::default()
						}),
					);
				}
			}
		}

		self.set_bindings(r, bindings);

		r
	}
}

fn type_term(term: &str) -> bool {
	term.chars().next().map(char::is_uppercase).unwrap_or(false)
}
