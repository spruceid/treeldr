use std::marker::PhantomData;

use educe::Educe;

/// Typed RDF resource identifier.
///
/// An RDF resource identified by the type parameter `R` can be anything.
/// This is a simple wrapper around an RDF resource identifier (`R`), with the
/// addition of some type information (`T`) giving some hint about the expected
/// type of the resource. The most common type found in this library is
/// [`LayoutType`](crate::LayoutType), meaning that the referenced resource is
/// a TreeLDR layout. However sometimes the type can be more precise (a layout
/// sub-type for instance).
///
/// Adding type information gives more legibility to the code and allows the use
/// of some handy functions taking care of type conversions. For instance,
/// the [`DerefResource`] trait provides a method to dereference a resource,
/// returning the definition matching the provided type information (e.g.
/// dereferencing a `Ref<LayoutType>` gives a `Layout`, dereferencing a
/// `Ref<RecordLayoutType>` gives a `RecordLayout`, etc.).
///
/// The default resource identifier is [`Term`], meaning that the resource is
/// identified by its lexical RDF representation (an IRI, a blank node
/// identifier or a literal value). This default parameter is easy to use but
/// beware of the following:
///   - A resource may have more than one lexical representation. Hence the
///     [`Term`] type is not adequate as a unique resource identifier.
///   - A term is basically a text string, it requires allocation when created
///     and cloned, and comparison is done in linear time (`O(n)`).
/// For these reasons, it is advised to use a more optimized/unique identifier
/// type for resources, using [`Vocabulary`](rdf_types::Vocabulary) to store
/// the actual lexical representations and
/// [`Interpretation`](rdf_types::Interpretation) to map lexical representations
/// to resources.
///
/// [`Term`]: rdf_types::Term
#[derive(Educe)]
#[educe(
	Debug(bound = "R: std::fmt::Debug"),
	Clone(bound = "R: Clone"),
	PartialEq(bound = "R: PartialEq"),
	Eq(bound = "R: Eq"),
	PartialOrd(bound = "R: PartialOrd"),
	Ord(bound = "R: Ord"),
	Hash(bound = "R: std::hash::Hash")
)]
#[repr(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(
	transparent,
	bound(
		serialize = "R: serde::Serialize",
		deserialize = "R: serde::Deserialize<'de>"
	)
)]
pub struct Ref<T, R = rdf_types::Term>(R, PhantomData<T>);

impl<R: Copy, T> Copy for Ref<T, R> {}

impl<T, R> Ref<T, R> {
	/// Creates a new typed resource identifier from the untyped resource
	/// identifier.
	///
	/// The actual type of the resource does not matter here, type checking is
	/// done when the resource is dereferenced.
	pub fn new(id: R) -> Self {
		Self(id, PhantomData)
	}

	/// Creates a new typed resource identifier reference from the untyped
	/// resource identifier reference.
	pub fn new_ref(id: &R) -> &Self {
		unsafe {
			// SAFETY: `Ref` uses `repr(transparent)` over `R`.
			std::mem::transmute(id)
		}
	}

	/// Returns the untyped resource identifier.
	pub fn id(&self) -> &R {
		&self.0
	}

	/// Consumes the typed identifier and returns the untyped resource
	/// identifier.
	pub fn into_id(self) -> R {
		self.0
	}

	/// Changes the type of the identifier.
	///
	/// It is always possible to change the type of a typed resource identifier:
	/// the actual type of the resource does not matter here, type checking is
	/// done when the resource is dereferenced.
	pub fn cast<U>(self) -> Ref<U, R> {
		Ref(self.0, PhantomData)
	}

	/// Returns a copy of this resource identifier with a different type.
	///
	/// It is always possible to change the type of a typed resource identifier:
	/// the actual type of the resource does not matter here, type checking is
	/// done when the resource is dereferenced.
	pub fn casted<U>(&self) -> Ref<U, R>
	where
		R: Clone,
	{
		Ref(self.0.clone(), PhantomData)
	}
}

/// Context able to fetch (dereference) the definition of a resource for the
/// given type `T`.
///
/// Resources are identified by the type parameter `R`.
pub trait DerefResource<T, R>: Sized {
	/// Type of the value returned upon dereferencing the resource identifier.
	type Target<'c>
	where
		Self: 'c,
		R: 'c;

	/// Returns the definition of the `resource` for the type `T`, if any.
	///
	/// This function returns `Some(value)` if the definition is found or `None`
	/// if the resource is unknown or if it has no known definition for the type
	/// `T` (type mismatch).
	fn deref_resource<'c>(&'c self, resource: &Ref<T, R>) -> Option<Self::Target<'c>>;
}
