use crate::Package;

use super::{Type, TypeRef};

pub struct TypeAllocator<'a, P, R> {
	package: &'a mut Package<P, R>,
	reserved_types: Vec<Option<Type<P, R>>>,
}

impl<'a, P, R> TypeAllocator<'a, P, R> {
	pub fn new(package: &'a mut Package<P, R>) -> Self {
		Self {
			package,
			reserved_types: Vec::new(),
		}
	}

	pub fn reserve_type(&mut self) -> TypeRef<P>
	where
		P: Copy,
	{
		let ty_ref = TypeRef::new(
			self.package.id,
			(self.package.types.len() + self.reserved_types.len()) as u32,
		);
		self.reserved_types.push(None);
		ty_ref
	}

	pub fn define_type(&mut self, r: TypeRef<P>, value: Type<P, R>) {
		let i = r.index as usize - self.package.types.len();
		self.reserved_types[i] = Some(value);
	}
}

impl<'a, P, R> Drop for TypeAllocator<'a, P, R> {
	fn drop(&mut self) {
		for t in self.reserved_types.drain(..) {
			match t {
				Some(t) => {
					self.package.types.insert(t);
				}
				None => return,
			}
		}
	}
}
