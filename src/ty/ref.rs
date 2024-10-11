use core::fmt;
use std::{borrow::Borrow, cmp::Ordering, hash::Hash, ops::Deref, sync::{Arc, OnceLock, Weak}};

use dashmap::DashMap;
use educe::Educe;

use super::Type;

#[derive(Debug, Educe)]
#[educe(Clone)]
pub struct WeakTypeRef<R>(Weak<TypeDefinition<R>>);

#[derive(Debug, Educe)]
#[educe(Clone)]
pub struct TypeRef<R>(Arc<TypeDefinition<R>>);

impl<R> TypeRef<R> {
	pub fn new(ty: Type<R>) -> Self {
		Self(Arc::new(TypeDefinition::new(ty)))
	}

	/// Create a new undefined type reference.
	/// 
	/// Trying to access the type definition before calling [`TypeRef::define`]
	/// will cause a panic.
	pub fn new_undefined() -> Self {
		Self(Arc::new(TypeDefinition::new_undefined()))
	}

	pub fn declare(&self, ty: Type<R>) {
		self.0.value.set(ty).ok().expect("type is already defined");
	}

	pub fn downgrade(&self) -> WeakTypeRef<R> {
		WeakTypeRef(Arc::downgrade(&self.0))
	}
}

impl<R: Ord + 'static> TypeRef<R> {
	pub fn is_subtype_of(&self, other: &Self) -> bool {
		self.subtype_cmp(other).is_some_and(Ordering::is_le)
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match self.0.subtype_cmp.get(other) {
			Some(cmp) => *cmp,
			None => {
				self.0.subtype_cmp.insert(other, Some(Ordering::Equal));
				let cmp = self.as_ref().subtype_cmp(other.as_ref());
				self.0.subtype_cmp.insert(other, cmp);
				other.0.subtype_cmp.insert(self, cmp.map(Ordering::reverse));
				cmp
			}
		}
	}
}

impl<R> PartialEq for TypeRef<R> {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<R> Eq for TypeRef<R> {}

impl<R> Hash for TypeRef<R> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		Arc::as_ptr(&self.0).hash(state);
	}
}

impl<R: 'static + PartialOrd> PartialOrd for TypeRef<R> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match self.0.cmp.get(other) {
			Some(cmp) => *cmp,
			None => {
				self.0.cmp.insert(other, Some(Ordering::Equal));
				let cmp = self.as_ref().partial_cmp(other.as_ref());
				self.0.cmp.insert(other, cmp);
				other.0.cmp.insert(self, cmp.map(Ordering::reverse));
				cmp
			}
		}
	}
}

impl<R: 'static + Ord> Ord for TypeRef<R> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap()
	}
}

impl<R> Borrow<Type<R>> for TypeRef<R> {
	fn borrow(&self) -> &Type<R> {
		self
	}
}

impl<R> AsRef<Type<R>> for TypeRef<R> {
	fn as_ref(&self) -> &Type<R> {
		self
	}
}

impl<R> Deref for TypeRef<R> {
	type Target = Type<R>;

	fn deref(&self) -> &Self::Target {
		self.0.value.get().expect("trying to access a non-defined type")
	}
}

struct TypeDefinition<R> {
	value: OnceLock<Type<R>>,
	cmp: WeakTypeMap<R, Option<Ordering>>,
	subtype_cmp: WeakTypeMap<R, Option<Ordering>>,
}

impl<R> TypeDefinition<R> {
	fn new(ty: Type<R>) -> Self {
		let value = OnceLock::new();
		let _ = value.set(ty);

		Self {
			value,
			cmp: WeakTypeMap::default(),
			subtype_cmp: WeakTypeMap::default()
		}
	}

	fn new_undefined() -> Self {
		Self {
			value: OnceLock::new(),
			cmp: WeakTypeMap::default(),
			subtype_cmp: WeakTypeMap::default()
		}
	}
}

impl<R: fmt::Debug> fmt::Debug for TypeDefinition<R> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.fmt(f)
	}
}

#[derive(Educe)]
#[educe(Default)]
struct WeakTypeMap<R, V>(DashMap<*const TypeDefinition<R>, V>);

impl<R, V> WeakTypeMap<R, V> {
	fn insert(&self, key: &TypeRef<R>, value: V) {
		self.0.insert(Weak::into_raw(key.downgrade().0), value);
	}

	fn get(&self, key: &TypeRef<R>) -> Option<dashmap::mapref::one::Ref<*const TypeDefinition<R>, V>> {
		self.0.get(&Arc::as_ptr(&key.0))
	}
}

impl<R, V> Drop for WeakTypeMap<R, V> {
	fn drop(&mut self) {
		for (ptr, _) in std::mem::take(&mut self.0) {
			unsafe {
				// SAFETY: `ptr` we created using `Weak::into_ptr`.
				let _ = Weak::from_raw(ptr);
			}
		}
	}
}