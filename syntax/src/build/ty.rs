use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{
	metadata::Merge,
	vocab::{Object, Rdf, Term},
	BlankIdIndex, Id, IriIndex,
};
use treeldr_build::Context;

use super::{Build, Declare, Error, LocalContext, LocalError};

impl<M: Clone + Merge> Declare<M> for Meta<crate::TypeDefinition<M>, M> {
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
		context.declare_type(id, self.metadata().clone());

		if let Meta(crate::TypeDescription::Normal(properties), _) = &self.description {
			for prop in properties {
				local_context.scope = Some(id);
				prop.declare(local_context, context, vocabulary, generator)?;
				local_context.scope = None
			}
		}

		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::TypeDefinition<M>, M> {
	type Target = ();

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let implicit_layout = Meta(self.implicit_layout_definition(), self.metadata().clone());

		let Meta(def, _) = self;
		let Meta(id, id_loc) = def
			.id
			.build(local_context, context, vocabulary, generator)?;

		match def.description {
			Meta(crate::TypeDescription::Normal(properties), _) => {
				for property in properties {
					local_context.scope = Some(id);
					let Meta(prop_id, prop_loc) =
						property.build(local_context, context, vocabulary, generator)?;
					local_context.scope = None;

					let prop = context.get_mut(prop_id).unwrap().as_property_mut();
					prop.domain_mut().insert_base(Meta(id, prop_loc.clone()));
				}
			}
			Meta(crate::TypeDescription::Alias(expr), expr_loc) => {
				local_context.alias_id = Some(Meta(id, id_loc));
				Meta(expr, expr_loc).build(local_context, context, vocabulary, generator)?;
				local_context.alias_id = None
			}
		}

		if let Some(comment) = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary, generator))
			.transpose()?
			.flatten()
		{
			let node = context.get_mut(id).unwrap();
			node.comment_mut().insert_base(comment.cast())
		}

		local_context.implicit_definition = true;
		implicit_layout.declare(local_context, context, vocabulary, generator)?;
		implicit_layout.build(local_context, context, vocabulary, generator)?;
		local_context.implicit_definition = false;

		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::OuterTypeExpr<M>, M> {
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
			crate::OuterTypeExpr::Inner(e) => {
				Meta(e, loc).build(local_context, context, vocabulary, generator)
			}
			crate::OuterTypeExpr::Union(label, options) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let options_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					options,
					|ty_expr, context, vocabulary, generator| {
						let Meta(id, loc) =
							ty_expr.build(local_context, context, vocabulary, generator)?;
						Ok(Meta(id.into_term(), loc))
					},
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut();
				ty.union_of_mut()
					.insert_base(Meta(options_list, loc.clone()));

				Ok(Meta(id, loc))
			}
			crate::OuterTypeExpr::Intersection(label, types) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let types_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					types,
					|ty_expr, context, vocabulary, generator| {
						let Meta(id, loc) =
							ty_expr.build(local_context, context, vocabulary, generator)?;
						Ok(Meta(id.into_term(), loc))
					},
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut();
				ty.intersection_of_mut()
					.insert_base(Meta(types_list, loc.clone()));

				Ok(Meta(id, loc))
			}
		}
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::NamedInnerTypeExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		self.into_value()
			.expr
			.build(local_context, context, vocabulary, generator)
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::InnerTypeExpr<M>, M> {
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
			crate::InnerTypeExpr::Outer(outer) => {
				outer.build(local_context, context, vocabulary, generator)
			}
			crate::InnerTypeExpr::Id(id) => {
				if let Some(Meta(id, id_loc)) = local_context.alias_id.take() {
					return Err(Meta(LocalError::TypeAlias(id, id_loc), loc).into());
				}

				id.build(local_context, context, vocabulary, generator)
			}
			crate::InnerTypeExpr::Reference(r) => {
				r.build(local_context, context, vocabulary, generator)
			}
			crate::InnerTypeExpr::Literal(label, lit) => {
				let id =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				local_context.generate_literal_type(
					&id,
					context,
					vocabulary,
					generator,
					Meta(lit, loc),
				)?;
				Ok(id)
			}
			crate::InnerTypeExpr::PropertyRestriction(r) => {
				let Meta(id, loc) = local_context.anonymous_id(None, vocabulary, generator, loc);
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let prop_id = r
					.prop
					.build(local_context, context, vocabulary, generator)?;

				use treeldr_build::ty::{restriction, Restriction};
				let Meta(restriction, restriction_loc) = r.restriction;
				let restriction = match restriction {
					crate::TypePropertyRestriction::Range(r) => {
						let r = match r {
							crate::TypePropertyRangeRestriction::Any(id) => {
								let Meta(id, _) =
									id.build(local_context, context, vocabulary, generator)?;
								restriction::Range::Any(id)
							}
							crate::TypePropertyRangeRestriction::All(id) => {
								let Meta(id, _) =
									id.build(local_context, context, vocabulary, generator)?;
								restriction::Range::All(id)
							}
						};

						Restriction::Range(r)
					}
					crate::TypePropertyRestriction::Cardinality(c) => {
						let c = match c {
							crate::TypePropertyCardinalityRestriction::AtLeast(min) => {
								restriction::Cardinality::AtLeast(min)
							}
							crate::TypePropertyCardinalityRestriction::AtMost(max) => {
								restriction::Cardinality::AtMost(max)
							}
							crate::TypePropertyCardinalityRestriction::Exactly(n) => {
								restriction::Cardinality::Exactly(n)
							}
						};

						Restriction::Cardinality(c)
					}
				};

				let node = context.get_mut(id).unwrap();
				node.type_mut().insert_base(Meta(
					treeldr_build::ty::SubClass::Restriction.into(),
					loc.clone(),
				));
				node.as_restriction_mut()
					.property_mut()
					.insert_base(prop_id);
				node.as_restriction_mut()
					.restriction_mut()
					.insert(Meta(restriction, restriction_loc));

				Ok(Meta(id, loc))
			}
			crate::InnerTypeExpr::List(label, item) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let Meta(item_id, _) = item.build(local_context, context, vocabulary, generator)?;

				// Restriction on the `rdf:first` property.
				use treeldr_build::ty::{restriction, Restriction};
				let Meta(first_restriction_id, _) =
					local_context.anonymous_id(None, vocabulary, generator, loc.clone());
				let first_restriction =
					context.declare_restriction(first_restriction_id, loc.clone());
				first_restriction
					.as_restriction_mut()
					.property_mut()
					.insert_base(Meta(
						Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
						loc.clone(),
					));
				first_restriction
					.as_restriction_mut()
					.restriction_mut()
					.insert(Meta(
						Restriction::Range(restriction::Range::All(item_id)),
						loc.clone(),
					));

				// Restriction on the `rdf:rest` property.
				let Meta(rest_restriction_id, _) =
					local_context.anonymous_id(None, vocabulary, generator, loc.clone());
				let rest_restriction =
					context.declare_restriction(rest_restriction_id, loc.clone());
				rest_restriction
					.as_restriction_mut()
					.property_mut()
					.insert_base(Meta(
						Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
						loc.clone(),
					));
				rest_restriction
					.as_restriction_mut()
					.restriction_mut()
					.insert(Meta(
						Restriction::Range(restriction::Range::All(id)),
						loc.clone(),
					));

				// Intersection list.
				let types_id = context.create_list(
					vocabulary,
					generator,
					[
						Meta(
							Object::Id(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List)))),
							loc.clone(),
						),
						Meta(first_restriction_id.into_term(), loc.clone()),
						Meta(rest_restriction_id.into_term(), loc.clone()),
					],
				);

				let ty = context.get_mut(id).unwrap().as_type_mut();
				ty.intersection_of_mut()
					.insert_base(Meta(types_id, loc.clone()));

				Ok(Meta(id, loc))
			}
		}
	}
}
