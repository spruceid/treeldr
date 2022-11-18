use locspan::Meta;
use treeldr::{Id, Type};

use crate::{Multiple, multiple};

#[derive(Debug, Clone)]
pub struct AnonymousData<M> {
	pub type_: Multiple<Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

#[derive(Debug, Clone)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: Multiple<Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			id,
			metadata,
			type_: Multiple::default(),
			label: Multiple::default(),
			comment: Multiple::default()
		}
	}

	pub fn clone_anonymous(&self) -> AnonymousData<M> where M: Clone {
		AnonymousData {
			type_: self.type_.clone(),
			label: self.label.clone(),
			comment: self.comment.clone()
		}
	}
}

pub enum BindingRef<'a, M> {
	Type(Meta<Id, &'a M>),
	Label(Meta<&'a str, &'a M>),
	Comment(Meta<&'a str, &'a M>)
}

pub struct Bindings<'a, M> {
	type_: multiple::Iter<'a, Id, M>,
	label: multiple::Iter<'a, String, M>,
	comment: multiple::Iter<'a, String, M>
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.next()
			.map(Meta::into_cloned_value)
			.map(BindingRef::Type)
			.or_else(|| {
				self.label
					.next()
					.map(|v| v.map(String::as_str))
					.map(BindingRef::Label)
					.or_else(|| {
						self.comment
							.next()
							.map(|v| v.map(String::as_str))
							.map(BindingRef::Comment)
					})
			})
	}
}