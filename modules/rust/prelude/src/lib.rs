pub use chrono;
pub use contextual;
pub use grdf;
pub use iref;
pub use rdf_types;
pub use static_iref;

pub mod iter;

#[cfg(feature = "json-ld")]
pub use json_ld;

#[cfg(feature = "json-ld")]
pub use locspan;

pub mod ty;

pub mod rdf;

#[cfg(feature = "json-ld")]
pub mod ld;

pub use rdf::{FromRdf, FromRdfError, RdfIterator};

#[cfg(feature = "json-ld")]
pub use crate::ld::{IntoJsonLdObject, IntoJsonLdObjectMeta, IntoJsonLdSyntax};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<I>(pub I);

impl<I> Id<I> {
	pub fn unwrap(self) -> I {
		self.0
	}
}

impl<I> From<I> for Id<I> {
	fn from(value: I) -> Self {
		Id(value)
	}
}

/// Contravariant reference type.
///
/// The referenced value may live shorter than `'a`, and at most `'a`.
pub struct ContravariantReference<'a, T: ?Sized> {
	/// Contravariance marker.
	_contravariance: std::marker::PhantomData<fn(&'a ()) -> ()>,

	/// Value pointer.
	ptr: *const T,
}

impl<'a, T: ?Sized> ContravariantReference<'a, T> {
	pub fn new(value: &'a T) -> Self {
		Self {
			_contravariance: std::marker::PhantomData,
			ptr: value as *const T,
		}
	}

	pub fn get<'b, U>(&self, f: impl FnOnce(&'b T) -> U) -> U
	where
		'a: 'b,
		T: 'b,
	{
		f(unsafe { &*self.ptr })
	}
}

pub trait Provider<I: ?Sized, T: ?Sized> {
	fn get(&self, id: &I) -> Option<&T>;
}

/// Reference type.
pub trait Reference<'a>: 'a + Copy {}

impl<'a> Reference<'a> for std::convert::Infallible {}

impl<'a, T> Reference<'a> for &'a T {}

pub trait Table {
	type Instance<'a>
	where
		Self: 'a;
}

/// Type that can be turned into a custom trait object using the dynamic table
/// `T`.
///
/// # Safety
///
/// - The data pointed by the pointer returned by the `as_trait_object` must live
/// at least as long as the input lifetime. This pointer must be a valid
/// parameter for the returned table in this lifetime.
///
/// - The data pointed by the pointer returned by the `into_trait_object` must
/// be valid to the lifetime `'r` (a parameter of the function). The pointer
/// must be a valid parameter for the returned table for `'r`.
pub unsafe trait AsTraitObject<T: Table> {
	/// Turns the given reference into a custom trait object.
	///
	/// # Safety
	///
	/// The returned pointer must live at least as long as the input lifetime.
	/// The returned pointer must be a valid parameter for the returned table in
	/// this lifetime.
	fn as_trait_object(&self) -> (*const u8, T::Instance<'_>);

	/// Turnes the given value into a custom trait object.
	///
	/// # Safety
	///
	/// The returned pointer must live at least as long as `'r`.
	/// The returned pointer must be a valid parameter for the returned table
	/// in the lifetime `'r`.
	fn into_trait_object<'r>(self) -> (*const u8, T::Instance<'r>)
	where
		Self: Reference<'r>,
	{
		unimplemented!()
	}
}

unsafe impl<T: Table> AsTraitObject<T> for std::convert::Infallible {
	fn as_trait_object(&self) -> (*const u8, T::Instance<'_>) {
		unreachable!()
	}

	fn into_trait_object<'r>(self) -> (*const u8, T::Instance<'r>)
	where
		Self: Reference<'r>,
	{
		unreachable!()
	}
}

unsafe impl<'a, T: Table, R: AsTraitObject<T>> AsTraitObject<T> for &'a R {
	fn as_trait_object(&self) -> (*const u8, T::Instance<'_>) {
		R::as_trait_object(*self)
	}

	fn into_trait_object<'r>(self) -> (*const u8, T::Instance<'r>)
	where
		Self: Reference<'r>,
	{
		R::as_trait_object(self)
	}
}

/// Dynamic iterator value.
///
/// This is used in place of `Box<dyn 'a + Iterator<Item = T>>` when one needs
/// to preserve covariance w.r.t `'a`.
pub struct BoxedDynIterator<'a, T> {
	/// Iterator lifetime.
	_lft: std::marker::PhantomData<&'a ()>,

	/// Pointer to the iterator value on the heap.
	ptr: *mut u8,

	/// Pointer to the `Iterator::next` function.
	next: fn(*mut u8) -> Option<T>,

	/// Drop function.
	///
	/// # Safety
	///
	/// This must be called at most once. The `ptr` must not be used afterward.
	drop: unsafe fn(*mut u8),
}

impl<'a, T> BoxedDynIterator<'a, T> {
	pub fn new<I: 'a + Iterator<Item = T>>(iterator: I) -> Self {
		let boxed = Box::new(iterator);

		Self {
			_lft: std::marker::PhantomData,
			ptr: Box::into_raw(boxed) as *mut u8,
			next: |ptr| {
				let value: &mut I = unsafe { &mut *(ptr as *mut I) };
				value.next()
			},
			drop: |ptr| unsafe {
				let layout = std::alloc::Layout::new::<I>();
				std::ptr::drop_in_place(ptr as *mut I);
				std::alloc::dealloc(ptr, layout);
			},
		}
	}
}

impl<'a, T> Iterator for BoxedDynIterator<'a, T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		(self.next)(self.ptr)
	}
}

impl<'a, T> Drop for BoxedDynIterator<'a, T> {
	fn drop(&mut self) {
		unsafe { (self.drop)(self.ptr) }
	}
}
