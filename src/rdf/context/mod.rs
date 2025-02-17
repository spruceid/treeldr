use iref::{Iri, IriBuf};
use rdf_types::{
	dataset::MultiPatternMatchingDataset,
	interpretation::{ConstGenerativeInterpretation, ReverseInterpretation},
	pattern::CanonicalQuadPattern,
	CowLiteral, Dataset, Interpretation, Literal, LiteralRef, Quad,
};
use std::{borrow::Cow, hash::Hash};

mod root;
use root::*;

mod delta;
use delta::*;

mod parent;
use parent::*;

mod class;
use class::*;

pub struct ContextStack<'a, I: ?Sized, D: ?Sized + Dataset> {
	parent: ParentContext<'a, I, D>,
	frame: ContextDelta<D::Resource>,
}

impl<'a, I, D> ContextStack<'a, I, D>
where
	I: ?Sized,
	D: ?Sized + Dataset,
{
	pub fn new(interpretation: &'a I, dataset: &'a D) -> Self {
		ContextStack {
			parent: ParentContext::Root(RootContext::new(interpretation, dataset)),
			frame: ContextDelta::new(),
		}
	}

	pub fn push(&self) -> ContextStack<'_, I, D> {
		ContextStack {
			parent: ParentContext::Sub(self),
			frame: ContextDelta::new(),
		}
	}

	pub fn pop(self) -> ContextDelta<D::Resource> {
		self.frame
	}

	fn new_non_canonical_resource(&self) -> I::Resource
	where
		I: ConstGenerativeInterpretation,
	{
		self.parent.new_resource()
	}

	pub fn iri_interpretation(&self, iri: &Iri) -> Option<I::Resource>
	where
		I: Interpretation<Resource = D::Resource>,
		D::Resource: Clone,
	{
		match self.frame.interpretation.iris.get(iri) {
			Some(&i) => Some(
				self.frame
					.interpretation
					.classes
					.get(i)
					.unwrap()
					.canonical_resource
					.clone(),
			),
			None => self.parent.iri_interpretation(iri),
		}
	}

	pub fn literal_interpretation(&self, literal: LiteralRef) -> Option<I::Resource>
	where
		I: Interpretation<Resource = D::Resource>,
		D::Resource: Clone,
	{
		match self.frame.interpretation.literals.get(&literal) {
			Some(&i) => Some(
				self.frame
					.interpretation
					.classes
					.get(i)
					.unwrap()
					.canonical_resource
					.clone(),
			),
			None => self.parent.literal_interpretation(literal),
		}
	}

	pub fn iris_of<'b>(&'b self, r: &'b I::Resource) -> IrisOf<'b, I>
	where
		I: ReverseInterpretation<Resource = D::Resource>,
		D::Resource: Eq + Hash,
	{
		let parent_canonical = self.parent.canonical_resource(r);
		match self
			.frame
			.interpretation
			.parent_mapping
			.get(parent_canonical)
		{
			Some(&i) => IrisOf::Sub(self.frame.interpretation.classes.get(i).unwrap().iris()),
			None => self.parent.iris_of(parent_canonical),
		}
	}

	pub fn literals_of<'b>(&'b self, r: &'b I::Resource) -> LiteralsOf<'b, I>
	where
		I: ReverseInterpretation<Resource = D::Resource>,
		D::Resource: Eq + Hash,
	{
		let parent_canonical = self.parent.canonical_resource(r);
		match self
			.frame
			.interpretation
			.parent_mapping
			.get(parent_canonical)
		{
			Some(&i) => {
				LiteralsOf::Sub(self.frame.interpretation.classes.get(i).unwrap().literals())
			}
			None => self.parent.literals_of(parent_canonical),
		}
	}

	pub fn new_resource(&mut self) -> I::Resource
	where
		I: ConstGenerativeInterpretation<Resource = D::Resource>,
		D::Resource: Clone + Eq + Hash,
	{
		let r = self.new_non_canonical_resource();
		self.frame
			.interpretation
			.insert_resource(Class::new_local(r.clone(), [], []));
		r
	}

	pub fn insert_iri<'b>(&mut self, iri: impl Into<Cow<'b, Iri>>) -> I::Resource
	where
		I: ConstGenerativeInterpretation<Resource = D::Resource>,
		D::Resource: Clone + Eq + Hash,
	{
		let iri = iri.into();
		match self.parent.iri_interpretation(iri.as_ref()) {
			Some(r) => r,
			None => {
				let r = self.new_non_canonical_resource();
				self.frame.interpretation.insert_resource(Class::new_local(
					r.clone(),
					[iri.into_owned()],
					[],
				));
				r
			}
		}
	}

	pub fn insert_literal<'b>(&mut self, literal: impl Into<CowLiteral<'b>>) -> I::Resource
	where
		I: ConstGenerativeInterpretation<Resource = D::Resource>,
		D::Resource: Clone + Eq + Hash,
	{
		let literal = literal.into();
		match self.parent.literal_interpretation(literal.as_ref()) {
			Some(r) => r,
			None => {
				let r = self.new_non_canonical_resource();
				self.frame.interpretation.insert_resource(Class::new_local(
					r.clone(),
					[],
					[literal.into_owned()],
				));
				r
			}
		}
	}

	pub fn insert(&mut self, quad: Quad<D::Resource>)
	where
		D::Resource: Ord,
	{
		self.frame.dataset.insert(quad);
	}

	pub fn canonical_resource<'r>(&'r self, r: &'r D::Resource) -> &'r D::Resource
	where
		D::Resource: Eq + Hash,
	{
		let parent_canonical = self.parent.canonical_resource(r);
		match self
			.frame
			.interpretation
			.parent_mapping
			.get(parent_canonical)
		{
			Some(&i) => {
				&self
					.frame
					.interpretation
					.classes
					.get(i)
					.unwrap()
					.canonical_resource
			}
			None => parent_canonical,
		}
	}

	pub fn canonical_resource_class<'r>(
		&'r self,
		r: &'r D::Resource,
	) -> ClassResources<'r, D::Resource>
	where
		D::Resource: Clone + Eq + Hash,
	{
		match self.frame.interpretation.parent_mapping.get(r) {
			Some(&i) => {
				self.frame
					.interpretation
					.classes
					// TODO: since this function assumes the input resource is
					// canonical, here `i` should directly point to a class.
					// If `DisjointVec` someday provides a method to get the
					// value of a class directly, we could use it here instead
					// of `get` that does class resolution.
					.get(i)
					.unwrap()
					.resources()
			}
			None => self.parent.canonical_resource_class(r),
		}
	}

	pub fn resource_class<'r>(&'r self, r: &'r D::Resource) -> ClassResources<'r, D::Resource>
	where
		D::Resource: Clone + Eq + Hash,
	{
		let parent_canonical = self.parent.canonical_resource(r);
		match self
			.frame
			.interpretation
			.parent_mapping
			.get(parent_canonical)
		{
			Some(&i) => self
				.frame
				.interpretation
				.classes
				.get(i)
				.unwrap()
				.resources(),
			None => self.parent.canonical_resource_class(r),
		}
	}

	fn import_resource(&mut self, r: &D::Resource) -> usize
	where
		I: ReverseInterpretation<Resource = D::Resource>,
		D::Resource: Clone + Eq + Hash,
	{
		let parent_canonical = self.parent.canonical_resource(r);
		match self
			.frame
			.interpretation
			.parent_mapping
			.get(parent_canonical)
		{
			Some(&i) => i,
			None => self
				.frame
				.interpretation
				.insert_resource(Class::from_parent(
					parent_canonical.clone(),
					self.parent
						.canonical_resource_class(parent_canonical)
						.iter()
						.cloned()
						.collect(),
					self.parent.iris_of(parent_canonical).map(Cow::into_owned),
					self.parent
						.literals_of(parent_canonical)
						.map(CowLiteral::into_owned),
				)),
		}
	}

	/// Unify two resources.
	pub fn unify(&mut self, a: &D::Resource, b: &D::Resource)
	where
		I: ReverseInterpretation<Resource = D::Resource>,
		D::Resource: Clone + Ord + Hash,
	{
		let ca = self.canonical_resource(a);
		let cb = self.canonical_resource(b);

		if ca != cb {
			let ia = self.import_resource(a);
			let ib = self.import_resource(b);
			self.frame.interpretation.unify(ia, ib)
		}
	}
}

impl<'a, I: ?Sized + ReverseInterpretation, D: Dataset> ContextStack<'a, I, D>
where
	I::Resource: Eq + Hash,
{
	// fn base_resource_to_deduced<'r>(&self, resource: &'r I::Resource) -> DeducedResourceRef<'r, I::Resource> {
	// 	match self.extended.interpretation.parent_mapping.get(resource).copied() {
	// 		Some(i) => DeducedResourceRef::Deduced(i),
	// 		None => DeducedResourceRef::Base(resource)
	// 	}
	// }

	// fn canonical_base_resource<'r>(&self, resource: &'r I::Resource) -> DeducedResourceRef<'r, I::Resource> {
	// 	match self.extended.interpretation.parent_mapping.get(resource).copied() {
	// 		Some(i) => self.canonical_extension_resource(i),
	// 		None => DeducedResourceRef::Base(resource)
	// 	}
	// }

	// fn canonical_resource<'r>(&self, resource: DeducedResourceRef<'r, I::Resource>) -> DeducedResourceRef<'r, I::Resource> {
	// 	match resource {
	// 		DeducedResourceRef::Base(r) => DeducedResourceRef::Base(r),
	// 		DeducedResourceRef::Deduced(i) => DeducedResourceRef::Deduced(self.extended.interpretation.classes.class_of(i).unwrap_or(i))
	// 	}
	// }

	// fn get_class(&self, r: DeducedResourceRef<I::Resource>) -> Option<ClassRef<I>> {
	// 	match r {
	// 		DeducedResourceRef::Base(r) => {
	// 			Some(ClassRef {
	// 				resources: std::slice::from_ref(r),
	// 				terms: ClassTerms::Singleton(self.base.interpretation.iris_of(r))
	// 			})
	// 		}
	// 		DeducedResourceRef::Deduced(i) => {
	// 			let class = self.frame.interpretation.classes.get(i)?;
	// 			Some(ClassRef {
	// 				resources: &class.resources,
	// 				terms: ClassTerms::Merged(&class.terms)
	// 			})
	// 		}
	// 	}
	// }
}

impl<'a, I, D> ContextStack<'a, I, D>
where
	I: ?Sized,
	D: ?Sized + MultiPatternMatchingDataset,
	D::Resource: Ord,
{
	pub fn pattern_matching<'b>(
		&'b self,
		pattern: CanonicalQuadPattern<&'b D::Resource>,
	) -> QuadPatternMatching<'b, 'b, D>
	where
		D::Resource: Clone + Hash,
	{
		self.multi_pattern_matching(pattern.map(|r| self.resource_class(r)))
	}

	/// Returns an iterator over all the quads of the dataset matching the given
	/// pattern.
	fn multi_pattern_matching<'p, P>(
		&self,
		pattern: CanonicalQuadPattern<P>,
	) -> QuadPatternMatching<'_, 'p, D>
	where
		P: IntoIterator<Item = &'p D::Resource>,
		D::Resource: 'p,
	{
		let (parent_pattern, frame_pattern) = pattern.map2(|resources| {
			let mut parent_resources = Vec::new();
			let mut frame_resources = Vec::new();

			for r in resources {
				parent_resources.push(r);
				frame_resources.push(r);
			}

			(parent_resources, frame_resources)
		});

		QuadPatternMatching {
			parent: self.parent.multi_pattern_matching(parent_pattern),
			frame: self.frame.dataset.multi_pattern_matching(frame_pattern),
		}
	}
}

pub struct QuadPatternMatching<'a, 'p, D>
where
	D: ?Sized + MultiPatternMatchingDataset,
{
	parent: ParentQuadPatternMatching<'a, 'p, D>,
	frame: rdf_types::dataset::indexed_btree_dataset::MultiPatternMatching<'a, D::Resource>,
}

impl<'a, 'p, D> Iterator for QuadPatternMatching<'a, 'p, D>
where
	D: ?Sized + MultiPatternMatchingDataset,
	D::Resource: 'p,
{
	type Item = Quad<&'a D::Resource>;

	fn next(&mut self) -> Option<Self::Item> {
		self.parent.next().or_else(|| self.frame.next())
	}
}

pub enum IrisOf<'a, I: 'a + ?Sized + ReverseInterpretation> {
	Root(I::Iris<'a>),
	Sub(std::iter::Chain<std::slice::Iter<'a, IriBuf>, std::slice::Iter<'a, IriBuf>>),
}

impl<'a, I> Iterator for IrisOf<'a, I>
where
	I: 'a + ?Sized + ReverseInterpretation,
{
	type Item = Cow<'a, Iri>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Root(c) => c.next(),
			Self::Sub(c) => c.next().map(|i| Cow::Borrowed(i.as_iri())),
		}
	}
}

pub enum LiteralsOf<'a, I: 'a + ?Sized + ReverseInterpretation> {
	Root(I::Literals<'a>),
	Sub(std::iter::Chain<std::slice::Iter<'a, Literal>, std::slice::Iter<'a, Literal>>),
}

impl<'a, I> Iterator for LiteralsOf<'a, I>
where
	I: 'a + ?Sized + ReverseInterpretation,
{
	type Item = CowLiteral<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Root(c) => c.next(),
			Self::Sub(c) => c.next().map(Literal::as_cow),
		}
	}
}
