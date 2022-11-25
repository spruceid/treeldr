use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{
	metadata::Merge,
	vocab::{Term, TreeLdr},
	BlankIdIndex, Id, IriIndex,
};
use treeldr_build::Context;

use crate::{LayoutFieldRangeRestriction, LayoutFieldRestriction};

use super::{Build, Declare, Error, LocalContext, LocalError};

impl<M: Clone + Merge> Declare<M> for Meta<crate::LayoutDefinition<M>, M> {
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let Meta(id, _) = self
			.id
			.clone()
			.build(local_context, context, vocabulary, generator)?;
		context.declare_layout(id, self.metadata().clone());
		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::LayoutDefinition<M>, M> {
	type Target = ();

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let Meta(def, _) = self;
		let Meta(id, id_loc) = def
			.id
			.build(local_context, context, vocabulary, generator)?;

		if let Some(comment) = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary, generator))
			.transpose()?
			.flatten()
		{
			let node = context.get_mut(id).unwrap();
			node.comment_mut().insert(comment)
		}

		let ty_id = match def.ty_id {
			Some(ty_id) => {
				let Meta(ty_id, ty_id_loc) =
					ty_id.build(local_context, context, vocabulary, generator)?;
				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.ty_mut()
					.insert(Meta(ty_id, ty_id_loc));
				Some(ty_id)
			}
			None => None,
		};

		match def.description {
			Meta(crate::LayoutDescription::Normal(fields), fields_loc) => {
				let fields_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					fields,
					|field, context, vocabulary, generator| {
						local_context.scope = ty_id;
						let Meta(item, item_loc) =
							field.build(local_context, context, vocabulary, generator)?;
						local_context.scope = None;
						Ok(Meta(item.into_term(), item_loc))
					},
				)?;

				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.description_mut()
					.insert(Meta(
						treeldr_build::layout::Description::Struct(fields_list),
						fields_loc,
					));
			}
			Meta(crate::LayoutDescription::Alias(expr), expr_loc) => {
				local_context.alias_id = Some(Meta(id, id_loc));
				Meta(expr, expr_loc).build(local_context, context, vocabulary, generator)?;
				local_context.alias_id = None;
			}
		}

		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::OuterLayoutExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(ty, loc) = self;

		match ty {
			crate::OuterLayoutExpr::Inner(e) => {
				Meta(e, loc).build(local_context, context, vocabulary, generator)
			}
			crate::OuterLayoutExpr::Union(label, options) => {
				let Meta(id, _) =
					local_context.anonymous_id(label, vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let variants = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					options,
					|layout_expr, context, vocabulary, generator| {
						let loc = layout_expr.metadata().clone();
						let variant_id = generator.next(vocabulary);

						let (layout_expr, variant_name) = if layout_expr.value().expr.is_namable() {
							let name = layout_expr.value().name.clone();
							(layout_expr, name)
						} else {
							let Meta(layout_expr, loc) = layout_expr;
							let (expr, name) = layout_expr.into_parts();
							(
								Meta(crate::NamedInnerLayoutExpr { expr, name: None }, loc),
								name,
							)
						};

						let Meta(layout, layout_loc) =
							layout_expr.build(local_context, context, vocabulary, generator)?;

						let variant_name = variant_name
							.map(|name| name.build(local_context, context, vocabulary, generator))
							.transpose()?;

						context.declare_layout_variant(variant_id, loc.clone());

						let variant = context.get_mut(variant_id).unwrap();
						variant
							.as_formatted_mut()
							.format_mut()
							.insert(Meta(layout, layout_loc));

						if let Some(name) = variant_name {
							variant.as_component_mut().name_mut().insert(name)
						}

						Ok(Meta(variant_id.into_term(), loc))
					},
				)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut();
				layout.description_mut().insert(Meta(
					treeldr_build::layout::Description::Enum(variants),
					loc.clone(),
				));
				if local_context.implicit_definition {
					layout.ty_mut().insert(Meta(id, loc.clone()))
				}

				Ok(Meta(id, loc))
			}
			crate::OuterLayoutExpr::Intersection(label, layouts) => {
				let Meta(id, _) =
					local_context.anonymous_id(label, vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let layouts_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					layouts,
					|layout_expr, context, vocabulary, generator| {
						let Meta(id, loc) =
							layout_expr.build(local_context, context, vocabulary, generator)?;
						Ok(Meta(id.into_term(), loc))
					},
				)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut();
				layout
					.intersection_of_mut()
					.insert(Meta(layouts_list, loc.clone()));
				if local_context.implicit_definition {
					layout.ty_mut().insert(Meta(id, loc.clone()));
				}

				Ok(Meta(id, loc))
			}
		}
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::NamedInnerLayoutExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(this, loc) = self;
		let is_namable = this.expr.is_namable();
		let Meta(id, expr_loc) = this
			.expr
			.build(local_context, context, vocabulary, generator)?;

		if let Some(name) = this.name {
			let Meta(name, name_loc) = name.build(local_context, context, vocabulary, generator)?;
			if is_namable {
				context
					.get_mut(id)
					.unwrap()
					.as_component_mut()
					.name_mut()
					.insert(Meta(name, name_loc));
			} else {
				return Err(Meta(LocalError::Renaming(id, expr_loc), loc).into());
			}
		}

		Ok(Meta(id, loc))
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::InnerLayoutExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(expr, loc) = self;

		match expr {
			crate::InnerLayoutExpr::Outer(outer) => {
				outer.build(local_context, context, vocabulary, generator)
			}
			crate::InnerLayoutExpr::Id(id) => {
				let alias_id = local_context.alias_id.take();

				let id = id.build(local_context, context, vocabulary, generator)?;

				match alias_id {
					Some(Meta(alias_id, alias_id_loc)) => {
						if alias_id.is_blank() {
							context.declare_layout(alias_id, alias_id_loc);
						}

						let layout = context.get_mut(alias_id).unwrap().as_layout_mut();
						layout.description_mut().insert(Meta(
							treeldr_build::layout::Description::Alias(*id),
							loc.clone(),
						));

						Ok(Meta(alias_id, loc))
					}
					None => Ok(id),
				}
			}
			crate::InnerLayoutExpr::Reference(ty_expr) => {
				let Meta(id, _) =
					local_context.anonymous_id(None, vocabulary, generator, loc.clone());

				let Meta(deref_ty, deref_loc) =
					ty_expr.build(local_context, context, vocabulary, generator)?;

				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let layout = context.get_mut(id).unwrap().as_layout_mut();
				layout.ty_mut().insert(Meta(deref_ty, deref_loc));
				let id_layout = Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(
					treeldr::layout::Primitive::Iri,
				))));
				layout.set_reference(Meta(id_layout, loc.clone()));

				Ok(Meta(id, loc))
			}
			crate::InnerLayoutExpr::Literal(label, lit) => {
				let id = local_context.anonymous_id(label, vocabulary, generator, loc.clone());
				local_context.generate_literal_layout(
					&id,
					context,
					vocabulary,
					generator,
					Meta(lit, loc),
				)?;
				Ok(id)
			}
			crate::InnerLayoutExpr::FieldRestriction(r) => {
				let Meta(id, _) =
					local_context.anonymous_id(None, vocabulary, generator, loc.clone());

				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let name = r
					.alias
					.map(|a| a.build(local_context, context, vocabulary, generator))
					.transpose()?;
				let prop_id = r
					.prop
					.build(local_context, context, vocabulary, generator)?;

				let format = match r.restriction {
					Meta(LayoutFieldRestriction::Cardinality(_), _) => todo!(),
					Meta(LayoutFieldRestriction::Range(r), _) => match r {
						LayoutFieldRangeRestriction::All(expr) => {
							expr.build(local_context, context, vocabulary, generator)?
						}
						LayoutFieldRangeRestriction::Any(_) => todo!(),
					},
				};

				let container_id = generator.next(vocabulary);
				let container = context.declare_layout(container_id, loc.clone());
				container.as_layout_mut().set_one_or_many(format);

				let field_id = generator.next(vocabulary);
				let field = context.declare_layout_field(field_id, loc.clone());

				if let Some(name) = name {
					field.as_component_mut().name_mut().insert(name)
				}

				field
					.as_formatted_mut()
					.format_mut()
					.insert(Meta(container_id, loc.clone()));
				field.as_layout_field_mut().property_mut().insert(prop_id);

				let fields_id = context.create_list(
					vocabulary,
					generator,
					Some(Meta(field_id.into_term(), loc.clone())),
				);

				let layout = context.get_mut(id).unwrap().as_layout_mut();
				layout.set_fields(Meta(fields_id, loc.clone()));

				Ok(Meta(id, loc))
			}
			crate::InnerLayoutExpr::Array(label, item) => {
				let Meta(id, _) =
					local_context.anonymous_id(label, vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let Meta(item_id, _) = item.build(local_context, context, vocabulary, generator)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut();
				if local_context.implicit_definition {
					layout.ty_mut().insert(Meta(id, loc.clone()));
					layout.set_array_semantics(treeldr_build::layout::array::Semantics::rdf_list(
						loc.clone(),
					))
				}

				layout.set_array(Meta(item_id, loc.clone()));

				Ok(Meta(id, loc))
			}
		}
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::FieldDefinition<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(def, loc) = self;

		let id = generator.next(vocabulary);

		let Meta(prop_id, prop_id_loc) =
			def.id
				.build(local_context, context, vocabulary, generator)?;

		let Meta(name, name_loc) = def
			.alias
			.unwrap_or_else(|| match prop_id {
				Id::Iri(id) => {
					let iri = vocabulary.iri(&id).unwrap();

					let id = match iri.fragment() {
						Some(fragment) => fragment.to_string(),
						None => iri
							.path()
							.file_name()
							.expect("invalid property IRI")
							.to_owned(),
					};

					Meta(crate::Alias(id), prop_id_loc.clone())
				}
				_ => panic!("invalid property IRI"),
			})
			.build(local_context, context, vocabulary, generator)?;

		let mut required_loc = None;
		let mut multiple_loc = None;
		let mut single_loc = None;

		let layout = match def.layout {
			Some(Meta(layout, _)) => {
				let scope = local_context.scope.take();
				let layout_id = layout
					.expr
					.build(local_context, context, vocabulary, generator)?;
				local_context.scope = scope;

				for Meta(ann, ann_loc) in layout.annotations {
					match ann {
						crate::Annotation::Multiple => multiple_loc = Some(ann_loc),
						crate::Annotation::Required => required_loc = Some(ann_loc),
						crate::Annotation::Single => single_loc = Some(ann_loc),
					}
				}

				let layout_id = match required_loc {
					Some(required_loc) => {
						match multiple_loc {
							Some(multiple_loc) => {
								// Wrap inside non-empty set.
								let restriction_id = generator.next(vocabulary);
								let restriction = context.declare_layout_restriction(
									restriction_id,
									required_loc.clone(),
								);
								use treeldr_build::layout::{restriction, Restriction};
								restriction.as_layout_restriction_mut().restriction_mut().insert(Meta(
									Restriction::Container(restriction::container::ContainerRestriction::Cardinal(treeldr::layout::restriction::cardinal::Restriction::Min(1))),
									required_loc.clone()
								));
								let restrictions_list = context.create_list(
									vocabulary,
									generator,
									Some(Meta(restriction_id.into_term(), required_loc.clone())),
								);

								let container_id = generator.next(vocabulary);
								context.declare_layout(container_id, multiple_loc.clone());
								let container_layout =
									context.get_mut(container_id).unwrap().as_layout_mut();
								let Meta(item_layout_id, item_layout_loc) = layout_id;

								match single_loc {
									Some(single_loc) => {
										container_layout.set_one_or_many(Meta(
											item_layout_id,
											multiple_loc.merged_with(single_loc),
										));
									}
									None => {
										container_layout
											.set_set(Meta(item_layout_id, multiple_loc));
									}
								}

								container_layout
									.restrictions_mut()
									.insert(Meta(restrictions_list, required_loc));

								Meta(container_id, item_layout_loc)
							}
							None => {
								// Wrap inside non-empty set.
								let container_id = generator.next(vocabulary);
								context.declare_layout(container_id, required_loc.clone());
								let container_layout =
									context.get_mut(container_id).unwrap().as_layout_mut();
								let Meta(item_layout_id, item_layout_loc) = layout_id;
								container_layout.set_required(Meta(item_layout_id, required_loc));
								Meta(container_id, item_layout_loc)
							}
						}
					}
					None => {
						match multiple_loc {
							Some(multiple_loc) => {
								// Wrap inside set.
								let container_id = generator.next(vocabulary);
								context.declare_layout(container_id, multiple_loc.clone());
								let container_layout =
									context.get_mut(container_id).unwrap().as_layout_mut();
								let Meta(item_layout_id, item_layout_loc) = layout_id;

								match single_loc {
									Some(single_loc) => {
										container_layout.set_one_or_many(Meta(
											item_layout_id,
											multiple_loc.merged_with(single_loc),
										));
									}
									None => {
										container_layout
											.set_set(Meta(item_layout_id, multiple_loc));
									}
								}

								Meta(container_id, item_layout_loc)
							}
							None => {
								// Wrap inside option.
								let container_id = generator.next(vocabulary);
								let Meta(item_layout_id, item_layout_loc) = layout_id;
								context.declare_layout(container_id, item_layout_loc.clone());
								let container_layout =
									context.get_mut(container_id).unwrap().as_layout_mut();
								container_layout
									.set_option(Meta(item_layout_id, item_layout_loc.clone()));
								Meta(container_id, item_layout_loc)
							}
						}
					}
				};

				Some(layout_id)
			}
			None => None,
		};

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary, generator))
			.transpose()?
			.flatten();

		context.declare_layout_field(id, loc.clone());
		let node = context.get_mut(id).unwrap();
		if let Some(comment) = doc {
			node.comment_mut().insert(comment)
		}

		node.as_component_mut()
			.name_mut()
			.insert(Meta(name, name_loc));

		if let Some(layout) = layout {
			node.as_formatted_mut().format_mut().insert(layout);
		}

		node.as_layout_field_mut()
			.property_mut()
			.insert(Meta(prop_id, prop_id_loc));

		Ok(Meta(id, loc))
	}
}
