use crate::{Context, Document, Error};
use grdf::{Dataset, Quad};
use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{
	metadata::Merge,
	ty::data::RegExp,
	value,
	vocab::{self, GraphLabel, Object, Term},
	BlankIdIndex, Id, IriIndex, Name,
};

pub(crate) fn expect_id<M>(
	Meta(value, loc): Meta<vocab::Object<M>, M>,
) -> Result<Meta<Id, M>, Error<M>> {
	match value {
		vocab::Object::Literal(_) => panic!("expected IRI or blank node label"),
		vocab::Object::Blank(id) => Ok(Meta(Id::Blank(id), loc)),
		vocab::Object::Iri(id) => Ok(Meta(Id::Iri(id), loc)),
	}
}

pub(crate) fn expect_type<M>(
	value: Meta<vocab::Object<M>, M>,
) -> Result<Meta<crate::Type, M>, Error<M>> {
	Ok(expect_id(value)?.map(Into::into))
}

pub(crate) fn expect_string<M>(
	Meta(value, _): Meta<vocab::Object<M>, M>,
) -> Result<Meta<String, M>, Error<M>> {
	match value {
		vocab::Object::Literal(l) => Ok(l.into_string_literal().map(|s| s.to_string())),
		_ => panic!("expected string literal"),
	}
}

pub(crate) fn expect_non_negative_integer<M>(
	Meta(value, _): Meta<vocab::Object<M>, M>,
) -> Result<Meta<u64, M>, Error<M>> {
	match value {
		vocab::Object::Literal(vocab::Literal::TypedString(Meta(s, meta), _)) => {
			match s.as_str().parse::<u64>() {
				Ok(u) => Ok(Meta(u, meta)),
				Err(_) => panic!("expected non negative integer"),
			}
		}
		_ => panic!("expected non negative integer"),
	}
}

pub(crate) fn expect_numeric<M>(
	Meta(value, _): Meta<vocab::Object<M>, M>,
) -> Result<Meta<value::Numeric, M>, Error<M>> {
	match value {
		vocab::Object::Literal(_lit) => {
			todo!("XSD numeric")
		}
		_ => panic!("expected numeric value"),
	}
}

pub(crate) fn expect_integer<M>(
	Meta(value, _): Meta<vocab::Object<M>, M>,
) -> Result<Meta<value::Integer, M>, Error<M>> {
	match value {
		vocab::Object::Literal(vocab::Literal::TypedString(Meta(_s, _meta), _ty)) => {
			todo!("XSD integer")
		}
		_ => panic!("expected integer value"),
	}
}

pub(crate) fn expect_regexp<M>(
	Meta(value, _): Meta<vocab::Object<M>, M>,
) -> Result<Meta<RegExp, M>, Error<M>> {
	match value {
		vocab::Object::Literal(vocab::Literal::TypedString(Meta(_s, _meta), _ty)) => {
			todo!("regexp")
		}
		_ => panic!("expected regular expression"),
	}
}

pub(crate) fn expect_schema_boolean<M>(
	Meta(value, meta): Meta<vocab::Object<M>, M>,
) -> Result<Meta<bool, M>, Error<M>> {
	match value {
		vocab::Object::Iri(IriIndex::Iri(vocab::Term::Schema(vocab::Schema::True))) => {
			Ok(Meta(true, meta))
		}
		vocab::Object::Iri(IriIndex::Iri(vocab::Term::Schema(vocab::Schema::False))) => {
			Ok(Meta(false, meta))
		}
		_ => panic!("expected boolean"),
	}
}

pub(crate) fn expect_name<M>(
	Meta(value, _): Meta<vocab::Object<M>, M>,
) -> Result<Meta<Name, M>, Error<M>> {
	match value {
		vocab::Object::Literal(vocab::Literal::String(Meta(s, meta))) => match Name::new(s) {
			Ok(name) => Ok(Meta(name, meta)),
			Err(_) => panic!("invalid name"),
		},
		_ => panic!("expected name"),
	}
}

impl<M: Clone + Ord + Merge> Document<M>
	for grdf::meta::BTreeDataset<Id, Id, Object<M>, GraphLabel, M>
{
	type LocalContext = ();
	type Error = Error<M>;

	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_: &mut (),
		context: &mut Context<M>,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		for Quad(id, _, ty, _) in self.pattern_matching(Quad(
			None,
			Some(&Id::Iri(IriIndex::Iri(Term::Rdf(vocab::Rdf::Type)))),
			None,
			None,
		)) {
			let Meta(_, metadata) = &ty.metadata;

			let ty = match &ty.value {
				rdf_types::Term::Iri(i) => Id::Iri(*i),
				rdf_types::Term::Blank(b) => Id::Blank(*b),
				_ => panic!("invalid type"),
			};

			context.declare_with(*id, ty, metadata.clone());
		}

		Ok(())
	}

	fn define<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		_: &mut (),
		context: &mut Context<M>,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		// Step 2: find out the properties of each node.
		for Meta(
			rdf_types::Quad(Meta(subject, subject_meta), predicate, object, _graph),
			_metadata,
		) in self.into_meta_quads()
		{
			if *predicate.value() != Id::Iri(IriIndex::Iri(Term::Rdf(vocab::Rdf::Type))) {
				let node = context
					.require_mut(subject)
					.map_err(|e| Meta(e.into(), subject_meta))?;
				node.set(predicate.into_value(), object)?
			}
		}

		Ok(())
	}
}
