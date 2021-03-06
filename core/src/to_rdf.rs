use crate::{layout, prop, ty, vocab, Documentation, Id, Model, Ref};
use rdf_types::{Literal, Object, Quad};
use vocab::{StrippedObject, StrippedQuad, Term};

pub trait Generator {
	fn next(&mut self) -> Id;
}

impl Generator for crate::Vocabulary {
	fn next(&mut self) -> Id {
		Id::Blank(self.new_blank_label())
	}
}

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
	!matches!(id, Id::Blank(_) | Id::Iri(Term::Unknown(_)))
}

impl<F> Model<F> {
	pub fn to_rdf(&self, generator: &mut impl Generator, quads: &mut Vec<StrippedQuad>) {
		self.to_rdf_with(generator, quads, Options::default())
	}

	pub fn to_rdf_with(
		&self,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
		options: Options,
	) {
		for (id, node) in self.nodes() {
			if !options.ignore_standard_vocabulary || !is_standard_vocabulary(id) {
				if let Some(ty_ref) = node.as_type() {
					let ty = self.types().get(ty_ref).unwrap();
					ty.to_rdf(self, generator, quads)
				}

				if let Some(prop_ref) = node.as_property() {
					let prop = self.properties().get(prop_ref).unwrap();
					prop.to_rdf(self, quads)
				}

				if let Some(layout_ref) = node.as_layout() {
					let layout = self.layouts().get(layout_ref).unwrap();
					layout.to_rdf(self, generator, quads)
				}

				if let Some(label) = node.label() {
					quads.push(Quad(
						id,
						Term::Rdfs(vocab::Rdfs::Label),
						Object::Literal(Literal::String(label.to_string().into())),
						None,
					))
				}

				node.documentation().to_rdf(id, quads)
			}
		}
	}
}

impl Documentation {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		for block in self.blocks() {
			quads.push(Quad(
				id,
				Term::Rdfs(vocab::Rdfs::Comment),
				Object::Literal(Literal::String(block.as_str().to_string().into())),
				None,
			))
		}
	}
}

fn to_rdf_list<I: IntoIterator<Item = StrippedObject>>(
	generator: &mut impl Generator,
	quads: &mut Vec<StrippedQuad>,
	iter: I,
) -> Id
where
	I::IntoIter: DoubleEndedIterator,
{
	let mut head = Id::Iri(Term::Rdf(vocab::Rdf::Nil));

	for item in iter.into_iter().rev() {
		let id = generator.next();
		quads.push(Quad(
			id,
			Term::Rdf(vocab::Rdf::Type),
			Object::Iri(Term::Rdf(vocab::Rdf::List)),
			None,
		));
		quads.push(Quad(id, Term::Rdf(vocab::Rdf::First), item, None));
		quads.push(Quad(
			id,
			Term::Rdf(vocab::Rdf::Rest),
			head.into_term(),
			None,
		));
		head = id;
	}

	head
}

impl<F> ty::Definition<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) {
		let class = if self.is_datatype(model) {
			vocab::Rdfs::Datatype
		} else {
			vocab::Rdfs::Class
		};

		quads.push(Quad(
			self.id(),
			Term::Rdf(vocab::Rdf::Type),
			Object::Iri(Term::Rdfs(class)),
			None,
		));

		match self.description() {
			ty::Description::Empty => (),
			ty::Description::Data(d) => d.to_rdf(self.id(), generator, quads),
			ty::Description::Normal(_) => (),
			ty::Description::Union(u) => u.to_rdf(model, self.id(), generator, quads),
			ty::Description::Intersection(i) => i.to_rdf(model, self.id(), generator, quads),
			ty::Description::Restriction(r) => r.to_rdf(model, self.id(), generator, quads),
		}
	}
}

impl ty::DataType {
	pub fn to_rdf(&self, id: Id, generator: &mut impl Generator, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::Primitive(_) => (),
			Self::Derived(d) => {
				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::OnDatatype),
					d.base().into_term(),
					None,
				));

				let restrictions_id = d.restrictions().to_rdf(generator, quads);
				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::WithRestrictions),
					restrictions_id.into_term(),
					None,
				));
			}
		}
	}
}

impl<'a> ty::data::Restrictions<'a> {
	pub fn to_rdf(self, generator: &mut impl Generator, quads: &mut Vec<StrippedQuad>) -> Id {
		let restrictions: Vec<_> = self
			.map(|restriction| {
				let id = generator.next();
				restriction.to_rdf(id, quads);
				id.into_term()
			})
			.collect();

		to_rdf_list(generator, quads, restrictions)
	}
}

impl<'a> ty::data::Restriction<'a> {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::Real(r) => r.to_rdf(id, quads),
			Self::Float(r) => r.to_rdf(id, quads),
			Self::Double(r) => r.to_rdf(id, quads),
			Self::String(r) => r.to_rdf(id, quads),
		}
	}
}

impl<'a> ty::data::restriction::real::Restriction<'a> {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		use ty::data::restriction::real::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinInclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinExclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxInclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxExclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
		}
	}
}

impl ty::data::restriction::float::Restriction {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		use ty::data::restriction::float::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinInclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinExclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxInclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxExclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
		}
	}
}

impl ty::data::restriction::double::Restriction {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		use ty::data::restriction::double::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinInclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinExclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxInclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxExclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
		}
	}
}

impl<'a> ty::data::restriction::string::Restriction<'a> {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::MinLength(min) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinLength),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
			Self::MaxLength(max) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxLength),
					Object::Literal(Literal::TypedString(
						max.to_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
			Self::Pattern(regexp) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::Pattern),
					Object::Literal(Literal::String(regexp.to_string().into())),
					None,
				));
			}
		}
	}
}

impl<F> ty::Union<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		id: Id,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) {
		let list_id = to_rdf_list(
			generator,
			quads,
			self.options()
				.map(|ty_ref| model.types().get(ty_ref).unwrap().id().into_term()),
		);

		quads.push(Quad(
			id,
			Term::Owl(vocab::Owl::UnionOf),
			list_id.into_term(),
			None,
		))
	}
}

impl<F> ty::Intersection<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		id: Id,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) {
		let list_id = to_rdf_list(
			generator,
			quads,
			self.types()
				.map(|ty_ref| model.types().get(ty_ref).unwrap().id().into_term()),
		);

		quads.push(Quad(
			id,
			Term::Owl(vocab::Owl::IntersectionOf),
			list_id.into_term(),
			None,
		))
	}
}

impl<F> ty::Restriction<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		id: Id,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) {
		match self.restrictions().len() {
			0 | 1 => {
				self.restrictions()
					.iter()
					.next()
					.unwrap()
					.to_rdf(model, id, self.property(), quads)
			}
			_ => {
				let restrictions: Vec<_> = self
					.restrictions()
					.iter()
					.map(|restriction| {
						let id = generator.next();
						restriction.to_rdf(model, id, self.property(), quads);
						id.into_term()
					})
					.collect();

				let list_id = to_rdf_list(generator, quads, restrictions);

				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::IntersectionOf),
					list_id.into_term(),
					None,
				))
			}
		}
	}
}

impl<F> prop::Restriction<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		id: Id,
		prop_ref: Ref<prop::Definition<F>>,
		quads: &mut Vec<StrippedQuad>,
	) {
		quads.push(Quad(
			id,
			Term::Rdf(vocab::Rdf::Type),
			Object::Iri(Term::Owl(vocab::Owl::Restriction)),
			None,
		));

		quads.push(Quad(
			id,
			Term::Owl(vocab::Owl::OnProperty),
			model.properties().get(prop_ref).unwrap().id().into_term(),
			None,
		));

		match self {
			Self::Range(r) => r.to_rdf(model, id, quads),
			Self::Cardinality(c) => c.to_rdf(id, quads),
		}
	}
}

impl<F> prop::restriction::Range<F> {
	pub fn to_rdf(&self, model: &Model<F>, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::Any(ty_ref) => {
				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::SomeValuesFrom),
					model.types().get(*ty_ref).unwrap().id().into_term(),
					None,
				));
			}
			Self::All(ty_ref) => {
				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::AllValuesFrom),
					model.types().get(*ty_ref).unwrap().id().into_term(),
					None,
				));
			}
		}
	}
}

impl prop::restriction::Cardinality {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::AtLeast(min) => {
				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::MinCardinality),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						Term::Xsd(vocab::Xsd::PositiveInteger),
					)),
					None,
				));
			}
			Self::AtMost(max) => {
				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::MaxCardinality),
					Object::Literal(Literal::TypedString(
						max.to_string().into(),
						Term::Xsd(vocab::Xsd::PositiveInteger),
					)),
					None,
				));
			}
			Self::Exactly(m) => {
				quads.push(Quad(
					id,
					Term::Owl(vocab::Owl::Cardinality),
					Object::Literal(Literal::TypedString(
						m.to_string().into(),
						Term::Xsd(vocab::Xsd::PositiveInteger),
					)),
					None,
				));
			}
		}
	}
}

impl<F> prop::Definition<F> {
	pub fn to_rdf(&self, model: &Model<F>, quads: &mut Vec<StrippedQuad>) {
		quads.push(Quad(
			self.id(),
			Term::Rdf(vocab::Rdf::Type),
			Object::Iri(Term::Rdf(vocab::Rdf::Property)),
			None,
		));

		for ty_ref in self.domain() {
			quads.push(Quad(
				self.id(),
				Term::Rdfs(vocab::Rdfs::Domain),
				model.types().get(ty_ref).unwrap().id().into_term(),
				None,
			))
		}

		quads.push(Quad(
			self.id(),
			Term::Rdfs(vocab::Rdfs::Range),
			model
				.types()
				.get(*self.range().inner())
				.unwrap()
				.id()
				.into_term(),
			None,
		));

		if self.is_required() {
			quads.push(Quad(
				self.id(),
				Term::Schema(vocab::Schema::ValueRequired),
				Object::Iri(Term::Schema(vocab::Schema::True)),
				None,
			));
		}

		if !self.is_functional() {
			quads.push(Quad(
				self.id(),
				Term::Schema(vocab::Schema::MultipleValues),
				Object::Iri(Term::Schema(vocab::Schema::True)),
				None,
			));
		}
	}
}

impl<F> layout::Definition<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) {
		quads.push(Quad(
			self.id(),
			Term::Rdf(vocab::Rdf::Type),
			Object::Iri(Term::TreeLdr(vocab::TreeLdr::Layout)),
			None,
		));

		if let Some(ty_ref) = self.ty() {
			quads.push(Quad(
				self.id(),
				Term::TreeLdr(vocab::TreeLdr::LayoutFor),
				model.types().get(ty_ref).unwrap().id().into_term(),
				None,
			));
		}

		if let Some(name) = self.name() {
			quads.push(Quad(
				self.id(),
				Term::TreeLdr(vocab::TreeLdr::Name),
				Object::Literal(Literal::String(name.as_str().to_string().into())),
				None,
			));
		}

		match self.description() {
			layout::Description::Never(_) => (),
			layout::Description::Primitive(n, _) => n.to_rdf(self.id(), generator, quads),
			layout::Description::Struct(s) => s.to_rdf(model, self.id(), generator, quads),
			layout::Description::Enum(e) => e.to_rdf(model, self.id(), generator, quads),
			layout::Description::Required(r) => r.to_rdf(model, self.id(), quads),
			layout::Description::Option(o) => o.to_rdf(model, self.id(), quads),
			layout::Description::Array(a) => a.to_rdf(model, self.id(), quads),
			layout::Description::Set(s) => s.to_rdf(model, self.id(), quads),
			layout::Description::Reference(r) => {
				quads.push(Quad(
					self.id(),
					Term::TreeLdr(vocab::TreeLdr::Reference),
					model.layouts().get(r.id_layout()).unwrap().id().into_term(),
					None,
				));
			}
			layout::Description::Alias(_, alias_ref) => {
				quads.push(Quad(
					self.id(),
					Term::TreeLdr(vocab::TreeLdr::Alias),
					model.layouts().get(*alias_ref).unwrap().id().into_term(),
					None,
				));
			}
		}
	}
}

impl<F> layout::Required<F> {
	pub fn to_rdf(&self, model: &Model<F>, id: Id, quads: &mut Vec<StrippedQuad>) {
		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Required),
			model
				.layouts()
				.get(self.item_layout())
				.unwrap()
				.id()
				.into_term(),
			None,
		))
	}
}

impl<F> layout::Optional<F> {
	pub fn to_rdf(&self, model: &Model<F>, id: Id, quads: &mut Vec<StrippedQuad>) {
		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Option),
			model
				.layouts()
				.get(self.item_layout())
				.unwrap()
				.id()
				.into_term(),
			None,
		))
	}
}

impl<F> layout::Array<F> {
	pub fn to_rdf(&self, model: &Model<F>, id: Id, quads: &mut Vec<StrippedQuad>) {
		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Array),
			model
				.layouts()
				.get(self.item_layout())
				.unwrap()
				.id()
				.into_term(),
			None,
		));

		if let Some(semantics) = self.semantics() {
			if let Some(first_prop) = semantics.first() {
				quads.push(Quad(
					id,
					Term::TreeLdr(vocab::TreeLdr::ArrayListFirst),
					model.properties().get(first_prop).unwrap().id().into_term(),
					None,
				));
			}

			if let Some(rest_prop) = semantics.rest() {
				quads.push(Quad(
					id,
					Term::TreeLdr(vocab::TreeLdr::ArrayListRest),
					model.properties().get(rest_prop).unwrap().id().into_term(),
					None,
				));
			}

			if let Some(nil_value) = semantics.nil() {
				quads.push(Quad(
					id,
					Term::TreeLdr(vocab::TreeLdr::ArrayListNil),
					nil_value.into_term(),
					None,
				));
			}
		}
	}
}

impl<F> layout::Set<F> {
	pub fn to_rdf(&self, model: &Model<F>, id: Id, quads: &mut Vec<StrippedQuad>) {
		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Set),
			model
				.layouts()
				.get(self.item_layout())
				.unwrap()
				.id()
				.into_term(),
			None,
		))
	}
}

impl layout::RestrictedPrimitive {
	pub fn to_rdf(&self, id: Id, generator: &mut impl Generator, quads: &mut Vec<StrippedQuad>) {
		if self.is_restricted() {
			quads.push(Quad(
				id,
				Term::TreeLdr(vocab::TreeLdr::DerivedFrom),
				self.primitive().id().into_term(),
				None,
			));

			let restrictions_id = self.restrictions().to_rdf(generator, quads);
			quads.push(Quad(
				id,
				Term::TreeLdr(vocab::TreeLdr::WithRestrictions),
				restrictions_id.into_term(),
				None,
			));
		} else {
			match id {
				Id::Iri(Term::TreeLdr(vocab::TreeLdr::Primitive(_))) => (),
				_ => {
					quads.push(Quad(
						id,
						Term::TreeLdr(vocab::TreeLdr::Alias),
						self.primitive().id().into_term(),
						None,
					));
				}
			}
		}
	}
}

impl<'a> layout::primitive::Restrictions<'a> {
	pub fn to_rdf(self, generator: &mut impl Generator, quads: &mut Vec<StrippedQuad>) -> Id {
		let restrictions: Vec<_> = self
			.map(|restriction| {
				let id = generator.next();
				restriction.to_rdf(id, quads);
				id.into_term()
			})
			.collect();

		to_rdf_list(generator, quads, restrictions)
	}
}

impl<'a> layout::primitive::Restriction<'a> {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::Integer(r) => r.to_rdf(id, quads),
			Self::UnsignedInteger(r) => r.to_rdf(id, quads),
			Self::Float(r) => r.to_rdf(id, quads),
			Self::Double(r) => r.to_rdf(id, quads),
			Self::String(r) => r.to_rdf(id, quads),
		}
	}
}

impl layout::primitive::restricted::integer::Restriction {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::MinInclusive(min) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinInclusive),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
			Self::MaxInclusive(min) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxInclusive),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
		}
	}
}

impl layout::primitive::restricted::unsigned::Restriction {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::MinInclusive(min) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinInclusive),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
			Self::MaxInclusive(min) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxInclusive),
					Object::Literal(Literal::TypedString(
						min.to_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
		}
	}
}

impl layout::primitive::restricted::float::Restriction {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		use layout::primitive::restricted::float::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinInclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinExclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxInclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxExclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
		}
	}
}

impl layout::primitive::restricted::double::Restriction {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		use layout::primitive::restricted::double::{Max, Min};
		match self {
			Self::Min(Min::Included(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinInclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Min(Min::Excluded(min)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinExclusive),
					Object::Literal(min.literal()),
					None,
				));
			}
			Self::Max(Max::Included(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxInclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
			Self::Max(Max::Excluded(max)) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxExclusive),
					Object::Literal(max.literal()),
					None,
				));
			}
		}
	}
}

impl<'a> layout::primitive::restricted::string::Restriction<'a> {
	pub fn to_rdf(&self, id: Id, quads: &mut Vec<StrippedQuad>) {
		match self {
			Self::MinLength(min) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MinLength),
					Object::Literal(Literal::TypedString(
						xsd_types::IntegerBuf::from(*min).into_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
			Self::MaxLength(max) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::MaxLength),
					Object::Literal(Literal::TypedString(
						xsd_types::IntegerBuf::from(*max).into_string().into(),
						Term::Xsd(vocab::Xsd::Integer),
					)),
					None,
				));
			}
			Self::Pattern(regexp) => {
				quads.push(Quad(
					id,
					Term::Xsd(vocab::Xsd::Pattern),
					Object::Literal(Literal::String(regexp.to_string().into())),
					None,
				));
			}
		}
	}
}

impl<F> layout::Struct<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		id: Id,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) {
		let mut fields = Vec::with_capacity(self.fields().len());
		for field in self.fields() {
			fields.push(field.to_rdf(model, generator, quads).into_term());
		}

		let fields_list = to_rdf_list(generator, quads, fields);

		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Fields),
			fields_list.into_term(),
			None,
		))
	}
}

impl<F> layout::Field<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) -> Id {
		let id = generator.next();

		quads.push(Quad(
			id,
			Term::Rdf(vocab::Rdf::Type),
			Object::Iri(Term::TreeLdr(vocab::TreeLdr::Field)),
			None,
		));

		if let Some(prop_ref) = self.property() {
			quads.push(Quad(
				id,
				Term::TreeLdr(vocab::TreeLdr::FieldFor),
				model.properties().get(prop_ref).unwrap().id().into_term(),
				None,
			))
		}

		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Format),
			model.layouts().get(self.layout()).unwrap().id().into_term(),
			None,
		));

		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Name),
			Object::Literal(Literal::String(self.name().to_string().into())),
			None,
		));

		if let Some(label) = self.label() {
			quads.push(Quad(
				id,
				Term::Rdfs(vocab::Rdfs::Label),
				Object::Literal(Literal::String(label.to_string().into())),
				None,
			));
		}

		self.documentation().to_rdf(id, quads);

		id
	}
}

impl<F> layout::Enum<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		id: Id,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) {
		let mut variants = Vec::with_capacity(self.variants().len());
		for variant in self.variants() {
			variants.push(variant.to_rdf(model, generator, quads).into_term());
		}

		let variants_list = to_rdf_list(generator, quads, variants);

		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Enumeration),
			variants_list.into_term(),
			None,
		))
	}
}

impl<F> layout::Variant<F> {
	pub fn to_rdf(
		&self,
		model: &Model<F>,
		generator: &mut impl Generator,
		quads: &mut Vec<StrippedQuad>,
	) -> Id {
		let id = generator.next();

		quads.push(Quad(
			id,
			Term::Rdf(vocab::Rdf::Type),
			Object::Iri(Term::TreeLdr(vocab::TreeLdr::Variant)),
			None,
		));

		if let Some(layout) = self.layout() {
			quads.push(Quad(
				id,
				Term::TreeLdr(vocab::TreeLdr::Format),
				model.layouts().get(layout).unwrap().id().into_term(),
				None,
			))
		}

		quads.push(Quad(
			id,
			Term::TreeLdr(vocab::TreeLdr::Name),
			Object::Literal(Literal::String(self.name().to_string().into())),
			None,
		));

		if let Some(label) = self.label() {
			quads.push(Quad(
				id,
				Term::Rdfs(vocab::Rdfs::Label),
				Object::Literal(Literal::String(label.to_string().into())),
				None,
			));
		}

		self.documentation().to_rdf(id, quads);

		id
	}
}
