use std::collections::BTreeSet;

use crate::{
	layout,
	node::BindingValueRef,
	ty,
	value::AsRdfLiteral,
	vocab::{self, StrippedObject, StrippedQuad, Term},
	BlankIdIndex, Id, IriIndex, MutableModel,
};
use locspan::Meta;
use rdf_types::{Generator, Literal, Object, Quad, Vocabulary};

#[derive(Debug, Clone, Copy)]
pub struct Options {
	/// Ignore standard definitions.
	///
	/// Defaults to `true`.
	pub ignore_standard_vocabulary: bool,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			ignore_standard_vocabulary: true,
		}
	}
}

fn is_standard_vocabulary(id: Id) -> bool {
	matches!(id, Id::Iri(IriIndex::Iri(_)))
}

pub trait ToRdf {
	type Target;

	fn to_rdf<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
	) -> Self::Target {
		self.to_rdf_with(vocabulary, generator, quads, Options::default())
	}

	fn to_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> Self::Target;
}

pub trait IntoRdf: Sized {
	type Target;

	fn into_rdf<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
	) -> Self::Target {
		self.into_rdf_with(vocabulary, generator, quads, Options::default())
	}

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> Self::Target;
}

pub trait MapIntoRdf: Sized + DoubleEndedIterator {
	type Target;

	fn map_into_rdf<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, G: Generator<V>>(
		self,
		vocabulary: &mut V,
		generator: &mut G,
		quads: &mut Vec<StrippedQuad>,
		f: impl Fn(&mut V, &mut G, &mut Vec<StrippedQuad>, Options, Self::Item) -> StrippedObject,
	) -> Self::Target {
		self.map_into_rdf_with(vocabulary, generator, quads, Options::default(), f)
	}

	fn map_into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, G: Generator<V>>(
		self,
		vocabulary: &mut V,
		generator: &mut G,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
		f: impl Fn(&mut V, &mut G, &mut Vec<StrippedQuad>, Options, Self::Item) -> StrippedObject,
	) -> Self::Target;
}

impl<M> ToRdf for MutableModel<M> {
	type Target = ();

	fn to_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) {
		let mut used = BTreeSet::new();
		let mut stack = Vec::new();

		for (id, _) in self.nodes() {
			if !id.is_blank()
				&& (!options.ignore_standard_vocabulary || !is_standard_vocabulary(id))
			{
				stack.push(id)
			}
		}

		while let Some(id) = stack.pop() {
			if used.insert(id) {
				let node = self.get_resource(id).unwrap();
				for Meta(b, _) in node.bindings() {
					for id in b.value().ids() {
						if !options.ignore_standard_vocabulary || !is_standard_vocabulary(id) {
							stack.push(id)
						}
					}
				}
			}
		}

		for id in used {
			let node = self.get_resource(id).unwrap();
			for Meta(b, _) in node.bindings() {
				let property = *b.property().id().as_iri().unwrap();
				let object = b
					.value()
					.into_rdf_with(vocabulary, generator, quads, options);

				quads.push(Quad(id, property, object, None))
			}
		}
	}
}

impl<'a, M> IntoRdf for BindingValueRef<'a, M> {
	type Target = StrippedObject;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> StrippedObject {
		match self {
			Self::SchemaBoolean(b) => {
				let term = if b {
					Term::Schema(crate::vocab::Schema::True)
				} else {
					Term::Schema(crate::vocab::Schema::False)
				};

				Object::Iri(IriIndex::Iri(term))
			}
			Self::NonNegativeInteger(u) => Object::Literal(u.as_rdf_literal()),
			Self::String(s) => Object::Literal(Literal::String(s.to_string().into())),
			Self::Name(n) => Object::Literal(Literal::String(n.to_string().into())),
			Self::Id(id) => id.into_term(),
			Self::Types(types) => types
				.iter()
				.map(|ty| ty.id().into_term())
				.into_rdf(vocabulary, generator, quads)
				.into_term(),
			Self::DataType(ty) => ty.id().into_term(),
			Self::DatatypeRestrictions(r) => r
				.iter()
				.map_into_rdf_with(
					vocabulary,
					generator,
					quads,
					options,
					|vocabulary, generator, quads, options, r| {
						r.into_rdf_with(vocabulary, generator, quads, options)
							.into_term()
					},
				)
				.into_term(),
			Self::Property(p) => p.id().into_term(),
			Self::Layouts(layouts) => layouts
				.iter()
				.map(|l| l.id().into_term())
				.into_rdf(vocabulary, generator, quads)
				.into_term(),
			Self::LayoutRestrictions(r) => r
				.iter()
				.map_into_rdf_with(
					vocabulary,
					generator,
					quads,
					options,
					|vocabulary, generator, quads, options, r| {
						r.into_rdf_with(vocabulary, generator, quads, options)
							.into_term()
					},
				)
				.into_term(),
			Self::Fields(fields) => fields
				.iter()
				.map(|f| f.id().into_term())
				.into_rdf_with(vocabulary, generator, quads, options)
				.into_term(),
			Self::Variants(variants) => variants
				.iter()
				.map(|v| v.id().into_term())
				.into_rdf_with(vocabulary, generator, quads, options)
				.into_term(),
		}
	}
}

impl<I> IntoRdf for I
where
	I: DoubleEndedIterator<Item = StrippedObject>,
{
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> Self::Target {
		self.map_into_rdf_with(vocabulary, generator, quads, options, |_, _, _, _, t| t)
	}
}

impl<I> MapIntoRdf for I
where
	I: DoubleEndedIterator,
{
	type Target = Id;

	fn map_into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, G: Generator<V>>(
		self,
		vocabulary: &mut V,
		generator: &mut G,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
		f: impl Fn(&mut V, &mut G, &mut Vec<StrippedQuad>, Options, Self::Item) -> StrippedObject,
	) -> Self::Target {
		let mut head = Id::Iri(IriIndex::Iri(Term::Rdf(vocab::Rdf::Nil)));

		for item in self.rev() {
			let id = generator.next(vocabulary);
			quads.push(Quad(
				id,
				IriIndex::Iri(Term::Rdf(vocab::Rdf::Type)),
				StrippedObject::Iri(IriIndex::Iri(Term::Rdf(vocab::Rdf::List))),
				None,
			));

			let object = f(vocabulary, generator, quads, options, item);

			quads.push(Quad(
				id,
				IriIndex::Iri(Term::Rdf(vocab::Rdf::First)),
				object,
				None,
			));
			quads.push(Quad(
				id,
				IriIndex::Iri(Term::Rdf(vocab::Rdf::Rest)),
				head.into_term(),
				None,
			));
			head = id;
		}

		head
	}
}

impl<'a> IntoRdf for ty::data::Restriction<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> Self::Target {
		match self {
			Self::Real(r) => r.into_rdf_with(vocabulary, generator, quads, options),
			Self::Float(r) => r.into_rdf_with(vocabulary, generator, quads, options),
			Self::Double(r) => r.into_rdf_with(vocabulary, generator, quads, options),
			Self::String(r) => r.into_rdf_with(vocabulary, generator, quads, options),
		}
	}
}

impl<'a> IntoRdf for ty::data::restriction::real::Restriction<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		use ty::data::restriction::real::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MinInclusive)),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MinExclusive)),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MaxInclusive)),
					Object::Literal(max.literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MaxExclusive)),
					Object::Literal(max.literal()),
					None,
				));
			}
		}

		id
	}
}

impl IntoRdf for ty::data::restriction::float::Restriction {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		use ty::data::restriction::float::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MinInclusive)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MinExclusive)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MaxInclusive)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MaxExclusive)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
		}

		id
	}
}

impl IntoRdf for ty::data::restriction::double::Restriction {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		use ty::data::restriction::double::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MinInclusive)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MinExclusive)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MaxInclusive)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MaxExclusive)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
		}

		id
	}
}

impl<'a> IntoRdf for ty::data::restriction::string::Restriction<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		match self {
			Self::MinLength(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MinLength)),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
			Self::MaxLength(max) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::MaxLength)),
					Object::Literal(Literal::TypedString(
						max.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
			Self::Pattern(regexp) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::Xsd(vocab::Xsd::Pattern)),
					Object::Literal(Literal::String(regexp.to_string().into())),
					None,
				));
			}
		}

		id
	}
}

impl<'a> IntoRdf for layout::restriction::RestrictionRef<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> Self::Target {
		match self {
			Self::Primitive(p) => p.into_rdf_with(vocabulary, generator, quads, options),
			Self::Container(c) => c.into_rdf_with(vocabulary, generator, quads, options),
		}
	}
}

impl<'a> IntoRdf for layout::primitive::RestrictionRef<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> Self::Target {
		match self {
			Self::Integer(r) => r.into_rdf_with(vocabulary, generator, quads, options),
			Self::UnsignedInteger(r) => r.into_rdf_with(vocabulary, generator, quads, options),
			Self::Float(r) => r.into_rdf_with(vocabulary, generator, quads, options),
			Self::Double(r) => r.into_rdf_with(vocabulary, generator, quads, options),
			Self::String(r) => r.into_rdf_with(vocabulary, generator, quads, options),
		}
	}
}

impl<'a> IntoRdf for layout::restriction::ContainerRestrictionRef<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) -> Self::Target {
		match self {
			Self::Cardinal(c) => c.into_rdf_with(vocabulary, generator, quads, options),
		}
	}
}

impl<'a> IntoRdf for layout::restriction::cardinal::RestrictionRef<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		match self {
			Self::Min(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::MinCardinality)),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
			Self::Max(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::MaxCardinality)),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
		}

		id
	}
}

impl<'a> IntoRdf for layout::primitive::restriction::integer::RestrictionRef<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		match self {
			Self::MinInclusive(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMinimum)),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
			Self::MaxInclusive(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMaximum)),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
		}

		id
	}
}

impl<'a> IntoRdf for layout::primitive::restriction::unsigned::RestrictionRef<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		match self {
			Self::MinInclusive(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMinimum)),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
			Self::MaxInclusive(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMaximum)),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						IriIndex::Iri(Term::Xsd(vocab::Xsd::Integer)),
					)),
					None,
				));
			}
		}

		id
	}
}

impl IntoRdf for layout::primitive::restriction::float::Restriction {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		use layout::primitive::restriction::float::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMinimum)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::ExclusiveMinimum)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMaximum)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::ExclusiveMaximum)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
		}

		id
	}
}

impl IntoRdf for layout::primitive::restriction::double::Restriction {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		use layout::primitive::restriction::double::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMinimum)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::ExclusiveMinimum)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::InclusiveMaximum)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::ExclusiveMaximum)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
		}

		id
	}
}

impl<'a> IntoRdf for layout::primitive::restriction::string::RestrictionRef<'a> {
	type Target = Id;

	fn into_rdf_with<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		quads: &mut Vec<StrippedQuad>,
		_options: Options,
	) -> Self::Target {
		let id = generator.next(vocabulary);

		match self {
			Self::MinLength(min) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::MinLength)),
					Object::Literal(min.as_rdf_literal()),
					None,
				));
			}
			Self::MaxLength(max) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::MaxLength)),
					Object::Literal(max.as_rdf_literal()),
					None,
				));
			}
			Self::Pattern(regexp) => {
				quads.push(Quad(
					id,
					IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::Pattern)),
					Object::Literal(Literal::String(regexp.to_string().into())),
					None,
				));
			}
		}

		id
	}
}
