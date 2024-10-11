use std::hash::Hash;

use crate::{value::TypedValue, Type, TypeRef, Value};

use super::TypedPattern;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedListPattern<R> {
	/// List of prefix patterns, front first.
	pub prefix: Vec<TypedPattern<R>>,

	/// List of suffix patterns, back first.
	pub suffix: Option<Vec<TypedPattern<R>>>
}

impl<R> TypedListPattern<R> {
	pub fn minimum_len(&self) -> usize {
		self.prefix.len() + self.suffix.as_ref().map(|s| s.len()).unwrap_or_default()
	}

	pub fn matches(&self, value: &[TypedValue<R>]) -> bool where R: Ord {
		if value.len() < self.minimum_len() {
			return false
		}

		// Prefix.
		let mut head = value.iter();
		for pattern in &self.prefix {
			if !pattern.matches(head.next().unwrap()) {
				return false
			}
		}

		// Suffix.
		if let Some(suffix) = &self.suffix {
			let mut tail = value.iter().rev();
			for pattern in suffix {
				if !pattern.matches(tail.next().unwrap()) {
					return false
				}
			}
		}

		true
	}

	pub fn instantiate(
		&self,
		ty_ref: &TypeRef<R>,
		mut list: Vec<Value<R>>
	) -> Result<Vec<TypedValue<R>>, Vec<Value<R>>> where R: Clone + Ord + Hash {
		let Type::List(ty) = ty_ref.as_ref() else {
			return Err(list)
		};

		if list.len() < self.minimum_len() {
			return Err(list)
		}

		let len = list.len();
		let mut result = Vec::with_capacity(len);

		// Suffix.
		let suffix = match &self.suffix {
			Some(suffix_pattern) => {
				let mut suffix = Vec::with_capacity(suffix_pattern.len());

				for pattern in suffix_pattern {
					let item = list.pop().unwrap();

					match pattern.instantiate(item) {
						Ok(typed_pattern) => {
							suffix.push(typed_pattern);
						}
						Err(item) => {
							list.push(item);
							list.extend(suffix.into_iter().rev().map(TypedValue::into_untyped));
							return Err(list)
						}
					}
				}
				
				suffix
			}
			None => Vec::new()
		};

		// Prefix.
		let mut i = 0;
		let mut items = list.into_iter();
		for pattern in &self.prefix {
			let item = items.next().unwrap();
			match pattern.instantiate(item) {
				Ok(typed_item) => {
					result.push(typed_item);
					i += 1;
				}
				Err(item) => {
					let mut list = Vec::with_capacity(len);
					list.extend(result.into_iter().map(TypedValue::into_untyped));
					list.push(item);
					list.extend(suffix.into_iter().rev().map(TypedValue::into_untyped));
					return Err(list)
				}
			}
		}

		// Infix.
		while let Some(item) = items.next() {
			let item_ty = ty.item_type(i).unwrap();
			match item.into_typed(item_ty) {
				Ok(typed_item) => {
					result.push(typed_item);
					i += 1;
				}
				Err(item) => {
					let mut list = Vec::with_capacity(len);
					list.extend(result.into_iter().map(TypedValue::into_untyped));
					list.push(item);
					list.extend(suffix.into_iter().rev().map(TypedValue::into_untyped));
					return Err(list)
				}
			}
		}

		// Add suffix.
		result.extend(suffix.into_iter().rev());
		Ok(result)
	}
}