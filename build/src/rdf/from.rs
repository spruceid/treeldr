use std::cmp::Ordering;

use crate::{error, Context, Document, Error};
use grdf::{Dataset, Quad};
use locspan::Meta;
use rdf_types::Generator;
use treeldr::{
	metadata::Merge,
	prop::UnknownProperty,
	vocab::{self, GraphLabel, StrippedObject, Term, TldrVocabulary},
	Id, IriIndex, TId,
};

impl<M: Clone + Ord + Merge> Document<M>
	for grdf::meta::BTreeDataset<Id, Id, StrippedObject, GraphLabel, M>
{
	type LocalContext = ();
	type Error = Error<M>;

	fn declare(
		&self,
		_: &mut (),
		context: &mut Context<M>,
		_vocabulary: &mut TldrVocabulary,
		_generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<(), Error<M>> {
		for Quad(id, _, ty, _) in self.pattern_matching(Quad(
			None,
			Some(&Id::Iri(IriIndex::Iri(Term::Rdf(vocab::Rdf::Type)))),
			None,
			None,
		)) {
			let Meta(_, metadata) = &ty.metadata;
			let ty = ty.value.as_id().copied().expect("invalid type");
			context.declare_with(*id, ty, metadata.clone());
		}

		Ok(())
	}

	fn define(
		self,
		_: &mut (),
		context: &mut Context<M>,
		vocabulary: &mut TldrVocabulary,
		_generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<(), Error<M>> {
		fn no_sub_prop(_: TId<UnknownProperty>, _: TId<UnknownProperty>) -> Option<Ordering> {
			unreachable!()
		}

		// Step 2: find out the properties of each node.
		for Meta(
			rdf_types::Quad(
				Meta(subject, subject_meta),
				predicate,
				Meta(object, value_meta),
				_graph,
			),
			_metadata,
		) in self.into_meta_quads()
		{
			if *predicate.value() != Id::Iri(IriIndex::Iri(Term::Rdf(vocab::Rdf::Type))) {
				let node = context
					.require_mut(subject)
					.map_err(|e| Meta(e.into(), subject_meta))?;
				let value = treeldr::Value::from_rdf(vocabulary, object)
					.map_err(|e| Meta(error::Description::from(e), value_meta.clone()))?;
				node.set(predicate.into_value(), no_sub_prop, Meta(value, value_meta))?
			}
		}

		Ok(())
	}
}

impl<M: Clone + Ord + Merge> Document<M>
	for grdf::meta::BTreeDataset<Id, IriIndex, StrippedObject, GraphLabel, M>
{
	type LocalContext = ();
	type Error = Error<M>;

	fn declare(
		&self,
		_: &mut (),
		context: &mut Context<M>,
		_vocabulary: &mut TldrVocabulary,
		_generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<(), Error<M>> {
		for Quad(id, _, ty, _) in self.pattern_matching(Quad(
			None,
			Some(&IriIndex::Iri(Term::Rdf(vocab::Rdf::Type))),
			None,
			None,
		)) {
			let Meta(_, metadata) = &ty.metadata;
			let ty = ty.value.as_id().copied().expect("invalid type");
			context.declare_with(*id, ty, metadata.clone());
		}

		Ok(())
	}

	fn define(
		self,
		_: &mut (),
		context: &mut Context<M>,
		vocabulary: &mut TldrVocabulary,
		_generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<(), Error<M>> {
		fn no_sub_prop(_: TId<UnknownProperty>, _: TId<UnknownProperty>) -> Option<Ordering> {
			unreachable!()
		}

		// Step 2: find out the properties of each node.
		for Meta(
			rdf_types::Quad(
				Meta(subject, subject_meta),
				predicate,
				Meta(object, value_meta),
				_graph,
			),
			_metadata,
		) in self.into_meta_quads()
		{
			if *predicate.value() != IriIndex::Iri(Term::Rdf(vocab::Rdf::Type)) {
				let node = context
					.require_mut(subject)
					.map_err(|e| Meta(e.into(), subject_meta))?;
				let value = treeldr::Value::from_rdf(vocabulary, object)
					.map_err(|e| Meta(error::Description::from(e), value_meta.clone()))?;
				node.set(
					Id::Iri(predicate.into_value()),
					no_sub_prop,
					Meta(value, value_meta),
				)?
			}
		}

		Ok(())
	}
}
