pub mod list;
pub mod literal;
pub mod product;
pub mod sum;

use educe::Educe;
pub use list::{
	ListLayout, ListLayoutType, OrderedListLayout, SizedListLayout, UnorderedListLayout,
};
pub use literal::{
	BooleanLayout, BooleanLayoutType, ByteStringLayout, ByteStringLayoutType, DataLayout,
	DataLayoutType, IdLayout, IdLayoutType, LiteralLayout, LiteralLayoutType, NumberLayout,
	NumberLayoutType, TextStringLayout, TextStringLayoutType, UnitLayout, UnitLayoutType,
};
pub use product::{ProductLayout, ProductLayoutType};
use rdf_types::Term;
use static_assertions::assert_impl_all;
use std::{collections::BTreeMap, hash::Hash};
pub use sum::{SumLayout, SumLayoutType};

use crate::{DerefResource, Layouts, Ref};

/// Layout resource type.
///
/// This is a type marker used in combination with the [`Ref`] type
/// (as `Ref<LayoutType>`) to identify RDF resources representing (arbitrary)
/// TreeLDR layouts.
///
/// The [`Layouts`] type stores the compiled definitions of layouts and
/// because it implements [`DerefResource<LayoutType, R>`] it can be used to
/// dereference a `Ref<LayoutType>` into a [`Layout`].
pub struct LayoutType;

impl<R: Ord> DerefResource<LayoutType, R> for Layouts<R> {
	type Target<'c> = &'c Layout<R> where R: 'c;

	fn deref_resource<'c>(&'c self, r: &crate::Ref<LayoutType, R>) -> Option<Self::Target<'c>> {
		self.layout(r.id())
	}
}

/// Layout definition.
///
/// Represents a compiled layout, defining a bidirectional mapping between
/// tree [`Value`](crate::Value) and RDF [`Dataset`](grdf::Dataset).
///
/// We say that a layout "matches" a tree value if it can be used to deserialize
/// it into an RDF dataset. Similarly we say that a layout "matches" an RDF
/// dataset if it can be used to serialize it into a tree value.
/// Serialization and deserialization is performed using the [`hydrate`] and
/// [`dehydrate`] functions respectively.
///
/// [`hydrate`]: crate::distill::hydrate
/// [`dehydrate`]: crate::distill::dehydrate
///
/// A layout accepts a number of inputs representing the subjects of the
/// RDF dataset (dataset given as input of the `hydrate` function or returned by
/// the `dehydrate` function). With the exception of the top and bottom layouts,
/// each layout has a fixed number of inputs. The top and bottom layouts will
/// accept any number of inputs. The input count can be found using the
/// [`Self::input_count()`] method.
#[derive(Debug, Clone, Educe, serde::Serialize, serde::Deserialize)]
#[educe(
	PartialEq(bound = "R: Ord"),
	Eq(bound = "R: Ord"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: Ord + Hash")
)]
#[serde(bound(deserialize = "R: Clone + Ord + serde::Deserialize<'de>"))]
pub enum Layout<R> {
	/// Bottom layout.
	///
	/// This layout does not match any tree value or RDF dataset.
	/// Using this layout will always cause failure upon serialization or
	/// deserialization.
	///
	/// It can be produced as a result of an empty layout intersection.
	///
	/// This layout accepts any number of inputs.
	Never,

	/// Literal layout.
	///
	/// This layout matches literal values (booleans, numbers, strings, etc.).
	/// The literal value can either represents an RDF literal (data layout),
	/// or an IRI/blank identifier (identifier layout).
	Literal(LiteralLayout<R>),

	/// Product layout.
	///
	/// Matches records (sometime called objects or maps).
	Product(ProductLayout<R>),

	/// List layout.
	///
	/// Matches lists of values, ordered or not.
	List(ListLayout<R>),

	/// Sum layout.
	///
	/// Matches any tree value or RDF dataset matched by exactly one member of
	/// the sum.
	Sum(SumLayout<R>),

	/// Top layout.
	///
	/// Matches any tree value and RDF dataset.
	/// Using this layout to serialize or deserialize will always succeed.
	///
	/// This layout accepts any number of inputs.
	Always,
}

assert_impl_all!(Layout<Term>: Send, Sync);

impl<R: Ord> PartialOrd for Layout<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<R> Layout<R> {
	/// Visits all the layout references this layout definition uses, calling
	/// the `f` function for each one of them.
	///
	/// The same dependency may be visited multiple times.
	pub fn visit_dependencies<'a>(&'a self, f: impl FnMut(&'a Ref<LayoutType, R>)) {
		match self {
			Self::Never => (),
			Self::Literal(_) => (),
			Self::Product(p) => p.visit_dependencies(f),
			Self::List(l) => l.visit_dependencies(f),
			Self::Sum(s) => s.visit_dependencies(f),
			Self::Always => (),
		}
	}

	/// Returns the list of all the layout references this layout definition
	/// uses.
	///
	/// The resulting list is sorted and with no duplicates.
	pub fn dependencies(&self) -> Vec<&Ref<LayoutType, R>>
	where
		R: Ord,
	{
		let mut result = Vec::new();

		self.visit_dependencies(|r| {
			result.push(r);
		});

		result.sort_unstable();
		result.dedup();
		result
	}

	/// Returns the number of inputs this layout requires.
	///
	/// For the top and bottom layouts (`Always` and `Never`), this function
	/// returns `None` as any number of input may be given for those layouts.
	pub fn input_count(&self) -> Option<u32> {
		match self {
			Self::Never => None,
			Self::Literal(_) => Some(1),
			Self::Product(p) => Some(p.input),
			Self::List(l) => Some(l.input_count()),
			Self::Sum(s) => Some(s.input),
			Self::Always => None,
		}
	}

	pub fn extra_properties<'a>(&'a self) -> &'a BTreeMap<R, R> {
		match self {
			Self::Never => <Self as NoExtraProperties<'a, R>>::NO_EXTRA_PROPERTIES,
			Self::Literal(l) => l.extra_properties(),
			Self::Product(p) => &p.extra_properties,
			Self::List(l) => l.extra_properties(),
			Self::Sum(s) => &s.extra_properties,
			Self::Always => <Self as NoExtraProperties<'a, R>>::NO_EXTRA_PROPERTIES,
		}
	}
}

trait NoExtraProperties<'a, R: 'a> {
	const NO_EXTRA_PROPERTIES: &'a BTreeMap<R, R> = &BTreeMap::new();
}

impl<'a, R: 'a> NoExtraProperties<'a, R> for Layout<R> {}
