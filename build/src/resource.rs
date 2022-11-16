use locspan::Meta;
use treeldr::{Name, Id, Type};

use crate::{Multiple, Single, multiple, single};

#[derive(Clone)]
pub struct Data<M> {
	pub id: Id,
	pub metadata: M,
	pub type_: Multiple<Type, M>,
	pub label: Multiple<String, M>,
	pub comment: Multiple<String, M>,
	pub name: Single<Name, M>,
	pub format: Single<Id, M>
}

impl<M> Data<M> {
	pub fn new(id: Id, metadata: M) -> Self {
		Self {
			id,
			metadata,
			type_: Multiple::default(),
			label: Multiple::default(),
			comment: Multiple::default(),
			name: Single::default(),
			format: Single::default()
		}
	}
}

pub enum BindingRef<'a, M> {
	Type(Meta<Id, &'a M>),
	Label(Meta<&'a str, &'a M>),
	Comment(Meta<&'a str, &'a M>),
	Name(Meta<&'a Name, &'a M>),
	Format(Meta<Id, &'a M>)
}

pub struct Bindings<'a, M> {
	type_: multiple::Iter<'a, Id, M>,
	label: multiple::Iter<'a, String, M>,
	comment: multiple::Iter<'a, String, M>,
	name: single::Iter<'a, Name, M>,
	format: single::Iter<'a, Id, M>
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
							.or_else(|| {
								self.name
									.next()
									.map(BindingRef::Name)
									.or_else(|| {
										self.format
											.next()
											.map(Meta::into_cloned_value)
											.map(BindingRef::Format)
									})
							})
					})
			})
	}
}