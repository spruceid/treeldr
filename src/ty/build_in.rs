use super::{LiteralType, TypeDesc};

macro_rules! built_in_types {
	{ $($field:ident : $variant:ident),* } => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub enum BuiltInType {
			$($variant),*
		}

		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct BuiltInTypeMap<T> {
			$(pub $field: T),*
		}

		impl<T> BuiltInTypeMap<T> {
			pub fn new(mut f: impl FnMut(BuiltInType) -> T) -> Self {
				Self {
					$(
						$field: f(BuiltInType::$variant),
					)*
				}
			}

			pub fn get(&self, ty: BuiltInType) -> &T {
				match ty {
					$(
						BuiltInType::$variant => &self.$field,
					)*
				}
			}

			pub fn get_mut(&mut self, ty: BuiltInType) -> &mut T {
				match ty {
					$(
						BuiltInType::$variant => &mut self.$field,
					)*
				}
			}

			pub fn replace(&mut self, ty: BuiltInType, value: T) -> T {
				match ty {
					$(
						BuiltInType::$variant => std::mem::replace(&mut self.$field, value),
					)*
				}
			}

			pub fn iter(&self) -> BuiltInTypeMapIter<T> {
				BuiltInTypeMapIter {
					$(
						$field: Some(&self.$field),
					)*
				}
			}
		}

		impl<'a, T> IntoIterator for &'a BuiltInTypeMap<T> {
			type Item = (BuiltInType, &'a T);
			type IntoIter = BuiltInTypeMapIter<'a, T>;

			fn into_iter(self) -> Self::IntoIter {
				self.iter()
			}
		}

		impl<T> IntoIterator for BuiltInTypeMap<T> {
			type Item = (BuiltInType, T);
			type IntoIter = BuiltInTypeMapIntoIter<T>;

			fn into_iter(self) -> Self::IntoIter {
				BuiltInTypeMapIntoIter {
					$(
						$field: Some(self.$field),
					)*
				}
			}
		}

		pub struct BuiltInTypeMapIter<'a, T> {
			$($field: Option<&'a T>),*
		}

		impl<'a, T> Iterator for BuiltInTypeMapIter<'a, T> {
			type Item = (BuiltInType, &'a T);

			fn next(&mut self) -> Option<Self::Item> {
				$(
					if let Some(item) = self.$field.take() {
						return Some((BuiltInType::$variant, item))
					}
				)*

				None
			}
		}

		pub struct BuiltInTypeMapIntoIter<T> {
			$($field: Option<T>),*
		}

		impl<T> Iterator for BuiltInTypeMapIntoIter<T> {
			type Item = (BuiltInType, T);

			fn next(&mut self) -> Option<Self::Item> {
				$(
					if let Some(item) = self.$field.take() {
						return Some((BuiltInType::$variant, item))
					}
				)*

				None
			}
		}
	};
}

built_in_types! {
	any: Any,
	resource: Resource,
	unit: Unit,
	boolean: Boolean,
	number: Number,
	int: Int,
	uint: Uint,
	byte_string: ByteString,
	text_string: TextString
}

impl BuiltInType {
	pub fn name(&self) -> &'static str {
		match self {
			Self::Any => "any",
			Self::Resource => "resource",
			Self::Unit => "unit",
			Self::Boolean => "bool",
			Self::Number => "number",
			Self::Int => "int",
			Self::Uint => "uint",
			Self::TextString => "string",
			Self::ByteString => "bytes",
		}
	}

	pub fn into_description<P, R>(self) -> TypeDesc<P, R> {
		match self {
			Self::Any => TypeDesc::Any,
			Self::Resource => TypeDesc::resource(),
			Self::Unit => TypeDesc::Literal(LiteralType::Unit),
			Self::Boolean => TypeDesc::Literal(LiteralType::boolean()),
			Self::Number => TypeDesc::Literal(LiteralType::number()),
			Self::Int => TypeDesc::Literal(LiteralType::int()),
			Self::Uint => TypeDesc::Literal(LiteralType::uint()),
			Self::TextString => TypeDesc::Literal(LiteralType::text_string()),
			Self::ByteString => TypeDesc::Literal(LiteralType::byte_string()),
		}
	}
}
