use iref::Iri;
use rdf_types::{
	dataset::MultiPatternMatchingDataset,
	interpretation::{ConstGenerativeInterpretation, ReverseInterpretation},
	pattern::CanonicalQuadPattern,
	Dataset, Interpretation, LiteralRef, Quad,
};
use std::hash::Hash;

use super::{
	class::ClassResources, root::RootContext, ContextStack, IrisOf, LiteralsOf, QuadPatternMatching,
};

pub enum ParentContext<'a, I: ?Sized, D: ?Sized + Dataset> {
	Root(RootContext<'a, I, D>),
	Sub(&'a ContextStack<'a, I, D>),
}

impl<'a, I, D> ParentContext<'a, I, D>
where
	I: ?Sized,
	D: ?Sized + Dataset,
{
	pub fn iri_interpretation(&self, iri: &Iri) -> Option<I::Resource>
	where
		I: Interpretation<Resource = D::Resource>,
		D::Resource: Clone,
	{
		match self {
			Self::Root(r) => r.interpretation.iri(iri),
			Self::Sub(s) => s.iri_interpretation(iri),
		}
	}

	pub fn literal_interpretation(&self, literal: LiteralRef) -> Option<I::Resource>
	where
		I: Interpretation<Resource = D::Resource>,
		D::Resource: Clone,
	{
		match self {
			Self::Root(r) => r.interpretation.literal(literal),
			Self::Sub(s) => s.literal_interpretation(literal),
		}
	}

	pub fn iris_of<'b>(&'b self, r: &'b I::Resource) -> IrisOf<'b, I>
	where
		I: ReverseInterpretation<Resource = D::Resource>,
		D::Resource: Eq + Hash,
	{
		match self {
			Self::Root(c) => IrisOf::Root(c.interpretation.iris_of(r)),
			Self::Sub(c) => c.iris_of(r),
		}
	}

	pub fn literals_of<'b>(&'b self, r: &'b I::Resource) -> LiteralsOf<'b, I>
	where
		I: ReverseInterpretation<Resource = D::Resource>,
		D::Resource: Eq + Hash,
	{
		match self {
			Self::Root(c) => LiteralsOf::Root(c.interpretation.literals_of(r)),
			Self::Sub(c) => c.literals_of(r),
		}
	}

	pub fn canonical_resource<'r>(&'r self, r: &'r D::Resource) -> &'r D::Resource
	where
		D::Resource: Eq + Hash,
	{
		match self {
			Self::Root(_) => r,
			Self::Sub(s) => s.canonical_resource(r),
		}
	}

	pub fn canonical_resource_class<'r>(
		&'r self,
		r: &'r D::Resource,
	) -> ClassResources<'r, D::Resource>
	where
		D::Resource: Clone + Eq + Hash,
	{
		match self {
			Self::Root(_) => ClassResources {
				local_resources: &[],
				parent_resources: std::slice::from_ref(r),
			},
			Self::Sub(s) => s.canonical_resource_class(r),
		}
	}

	pub fn new_resource(&self) -> I::Resource
	where
		I: ConstGenerativeInterpretation,
	{
		match self {
			Self::Root(r) => r.new_resource(),
			Self::Sub(s) => s.new_non_canonical_resource(),
		}
	}

	pub fn multi_pattern_matching<'p, P>(
		&self,
		pattern: CanonicalQuadPattern<P>,
	) -> ParentQuadPatternMatching<'_, 'p, D>
	where
		P: IntoIterator<Item = &'p D::Resource>,
		D: MultiPatternMatchingDataset,
		D::Resource: 'p + Ord,
	{
		match self {
			Self::Root(r) => {
				ParentQuadPatternMatching::Root(r.dataset.quad_multi_pattern_matching(pattern))
			}
			Self::Sub(f) => {
				ParentQuadPatternMatching::Frame(Box::new(f.multi_pattern_matching(pattern)))
			}
		}
	}
}

pub enum ParentQuadPatternMatching<'a, 'p, D>
where
	D: 'a + ?Sized + MultiPatternMatchingDataset,
	D::Resource: 'p,
{
	Root(D::QuadMultiPatternMatching<'a, 'p>),
	Frame(Box<QuadPatternMatching<'a, 'p, D>>),
}

impl<'a, 'p, D> Iterator for ParentQuadPatternMatching<'a, 'p, D>
where
	D: 'a + ?Sized + MultiPatternMatchingDataset,
	D::Resource: 'p,
{
	type Item = Quad<&'a D::Resource>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Root(r) => r.next(),
			Self::Frame(f) => f.next(),
		}
	}
}
