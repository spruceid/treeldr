use std::{collections::BTreeMap, hash::Hash};

use crate::{value::{TypedMap, TypedValue}, Type, TypeRef, Value};

use super::TypedPattern;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedMapPattern<R> {
	pub entries: BTreeMap<TypedPattern<R>, TypedPattern<R>>,
	pub ellipsis: bool
}

impl<R> TypedMapPattern<R> {
	pub fn minimum_len(&self) -> usize {
		self.entries.len()
	}

	pub fn maximum_len(&self) -> Option<usize> {
		if self.ellipsis {
			None
		} else {
			Some(self.entries.len())
		}
	}
	
	pub fn matches(&self, map: &TypedMap<R, TypedValue<R>>) -> bool where R: Ord {
		if map.len() < self.minimum_len() {
			return false
		}

		if !self.ellipsis && map.len() > self.minimum_len() {
			return false
		}

		for (key_pattern, value_pattern) in &self.entries {
			if map.pattern_matching(key_pattern).all(|(_, value)| !value_pattern.matches(value)) {
				return false
			}
		}

		true
	}

	pub fn instantiate(
		&self,
		ty_ref: &TypeRef<R>,
		map: BTreeMap<Value<R>, Value<R>>
	) -> Result<TypedMap<R, TypedValue<R>>, BTreeMap<Value<R>, Value<R>>> where R: Clone + Ord + Hash {
		let Type::Struct(ty) = ty_ref.as_ref() else {
			return Err(map)
		};

		if map.len() < self.minimum_len() || self.maximum_len().is_some_and(|m| map.len() > m) {
			return Err(map)
		}

		let mut result = TypedMap::new();
		let mut entries: Vec<_> = map.into_iter().collect();
		let mut fields: Vec<_> = ty.fields.iter().map(|(key, value)| (key, value)).collect();

		// Pattern entries.
		for (key_pattern, value_pattern) in &self.entries {
			let _ = remove_first_match(&mut fields, |(key, _)| key_pattern.is_sub_pattern_of(key));

			let typed_entry = remove_first_map(&mut entries, |(key, value)| {
				match key_pattern.instantiate(key) {
					Ok(typed_key) => {
						match value_pattern.instantiate(value) {
							Ok(typed_value) => Ok((typed_key, typed_value)),
							Err(value) => Err((typed_key.into_untyped(), value))
						}
					}
					Err(key) => Err((key, value))
				}
			});

			match typed_entry {
				Some((typed_key, typed_value)) => {
					result.insert(typed_key, typed_value);
				}
				None => {
					let mut map = BTreeMap::new();
					map.extend(entries);
					map.extend(result.into_iter().map(|(k, v)| (k.into_untyped(), v.into_untyped())));
					return Err(map)
				}
			}
		}

		// Remaining struct entries.
		for (key_pattern, field) in fields {
			let typed_entry = remove_first_map(&mut entries, |(key, value)| {
				match key_pattern.instantiate(key) {
					Ok(typed_key) => {
						match value.into_typed(&field.type_) {
							Ok(typed_value) => Ok((typed_key, typed_value)),
							Err(value) => Err((typed_key.into_untyped(), value))
						}
					}
					Err(key) => Err((key, value))
				}
			});

			match typed_entry {
				Some((typed_key, typed_value)) => {
					result.insert(typed_key, typed_value);
				}
				None => {
					if field.required {
						let mut map = BTreeMap::new();
						map.extend(entries);
						map.extend(result.into_iter().map(|(k, v)| (k.into_untyped(), v.into_untyped())));
						return Err(map)
					}
				}
			}
		}
		
		// Unexpected entries.
		if let Some(_) = entries.pop() {
			let mut map = BTreeMap::new();
			map.extend(entries);
			map.extend(result.into_iter().map(|(k, v)| (k.into_untyped(), v.into_untyped())));
			return Err(map)
		}

		Ok(result)
	}
}

fn remove_first_match<T>(list: &mut Vec<T>, mut f: impl FnMut(&T) -> bool) -> Option<T> {
	let len = list.len();
	
	for i in 0..len {
		if f(&list[i]) {
			return Some(list.swap_remove(i))
		}
	}

	None
}

fn remove_first_map<T, U>(list: &mut Vec<T>, mut f: impl FnMut(T) -> Result<U, T>) -> Option<U> {
	let len = list.len();
	
	for _ in 0..len {
		let item = list.swap_remove(0);
		match f(item) {
			Ok(u) => return Some(u),
			Err(t) => list.push(t)
		}
	}

	None
}