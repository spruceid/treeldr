use std::cmp::Ordering;

use educe::Educe;

use super::Literal;

pub trait ValueLike {
	type Resource;
	type List: ListLike<Resource = Self::Resource>;
	type Map: MapLike<Resource = Self::Resource>;

	fn destruct(&self) -> Destruct<Self>;

	fn eq(&self, other: &impl ValueLike<Resource = Self::Resource>) -> bool
	where
		Self::Resource: Eq,
	{
		self.destruct().eq(other.destruct())
	}

	fn cmp(&self, other: &impl ValueLike<Resource = Self::Resource>) -> Ordering
	where
		Self::Resource: Ord,
	{
		self.destruct().cmp(other.destruct())
	}
}

pub trait ListLike {
	type Resource;
	type Value: ValueLike<Resource = Self::Resource>;

	fn len(&self) -> usize;

	fn iter(&self) -> impl Iterator<Item = &Self::Value>;

	fn eq(&self, other: &impl ListLike<Resource = Self::Resource>) -> bool
	where
		Self::Resource: Eq,
	{
		let mut a_iter = self.iter();
		let mut b_iter = other.iter();

		loop {
			match (a_iter.next(), b_iter.next()) {
				(Some(a), Some(b)) => {
					if !a.eq(b) {
						break false;
					}
				}
				(None, None) => break true,
				_ => break false,
			}
		}
	}

	fn cmp(&self, other: &impl ListLike<Resource = Self::Resource>) -> Ordering
	where
		Self::Resource: Ord,
	{
		let mut a_iter = self.iter();
		let mut b_iter = other.iter();

		loop {
			match (a_iter.next(), b_iter.next()) {
				(Some(a), Some(b)) => match a.cmp(b) {
					Ordering::Equal => (),
					ord => break ord,
				},
				(Some(_), None) => break Ordering::Greater,
				(None, Some(_)) => break Ordering::Less,
				(None, None) => break Ordering::Equal,
			}
		}
	}
}

pub trait MapLike {
	type Resource;
	type Value: ValueLike<Resource = Self::Resource>;

	fn len(&self) -> usize;

	fn iter(&self) -> impl Iterator<Item = (&Self::Value, &Self::Value)>;

	fn eq(&self, other: &impl MapLike<Resource = Self::Resource>) -> bool
	where
		Self::Resource: Eq,
	{
		let mut a_iter = self.iter();
		let mut b_iter = other.iter();

		loop {
			match (a_iter.next(), b_iter.next()) {
				(Some((a_key, a_value)), Some((b_key, b_value))) => {
					if !a_key.eq(b_key) || !a_value.eq(b_value) {
						break false;
					}
				}
				(None, None) => break true,
				_ => break false,
			}
		}
	}

	fn cmp(&self, other: &impl MapLike<Resource = Self::Resource>) -> Ordering
	where
		Self::Resource: Ord,
	{
		let mut a_iter = self.iter();
		let mut b_iter = other.iter();

		loop {
			match (a_iter.next(), b_iter.next()) {
				(Some((a_key, a_value)), Some((b_key, b_value))) => match a_key.cmp(b_key) {
					Ordering::Equal => match a_value.cmp(b_value) {
						Ordering::Equal => (),
						ord => break ord,
					},
					ord => break ord,
				},
				(Some(_), None) => break Ordering::Greater,
				(None, Some(_)) => break Ordering::Less,
				(None, None) => break Ordering::Equal,
			}
		}
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum Destruct<'a, T: ?Sized + ValueLike> {
	Resource(&'a T::Resource),
	Literal(&'a Literal),
	List(&'a T::List),
	Map(&'a T::Map),
}

impl<'a, T: ?Sized + ValueLike> Destruct<'a, T>
where
	T::Resource: Eq,
{
	pub fn eq<U: ?Sized + ValueLike<Resource = T::Resource>>(self, other: Destruct<'a, U>) -> bool {
		match (self, other) {
			(Self::Resource(a), Destruct::Resource(b)) => a.eq(b),
			(Self::Literal(a), Destruct::Literal(b)) => a.eq(b),
			(Self::List(a), Destruct::List(b)) => a.eq(b),
			(Self::Map(a), Destruct::Map(b)) => a.eq(b),
			_ => false,
		}
	}
}

impl<'a, T: ?Sized + ValueLike> Destruct<'a, T>
where
	T::Resource: Ord,
{
	pub fn cmp<U: ?Sized + ValueLike<Resource = T::Resource>>(
		self,
		other: Destruct<'a, U>,
	) -> Ordering {
		match (self, other) {
			(Self::Resource(a), Destruct::Resource(b)) => a.cmp(b),
			(Self::Resource(_), _) => Ordering::Less,
			(_, Destruct::Resource(_)) => Ordering::Greater,
			(Self::Literal(a), Destruct::Literal(b)) => a.cmp(b),
			(Self::Literal(_), _) => Ordering::Less,
			(_, Destruct::Literal(_)) => Ordering::Greater,
			(Self::List(a), Destruct::List(b)) => a.cmp(b),
			(Self::List(_), _) => Ordering::Less,
			(_, Destruct::List(_)) => Ordering::Greater,
			(Self::Map(a), Destruct::Map(b)) => a.cmp(b),
		}
	}
}

impl<'a, T: ?Sized + ValueLike, U: ?Sized + ValueLike<Resource = T::Resource>>
	PartialEq<Destruct<'a, U>> for Destruct<'a, T>
where
	T::Resource: Eq,
{
	fn eq(&self, other: &Destruct<'a, U>) -> bool {
		Self::eq(*self, *other)
	}
}

impl<'a, T: ?Sized + ValueLike> Eq for Destruct<'a, T> where T::Resource: Eq {}

impl<'a, T: ?Sized + ValueLike, U: ?Sized + ValueLike<Resource = T::Resource>>
	PartialOrd<Destruct<'a, U>> for Destruct<'a, T>
where
	T::Resource: Ord,
{
	fn partial_cmp(&self, other: &Destruct<'a, U>) -> Option<Ordering> {
		Some(Self::cmp(*self, *other))
	}
}

impl<'a, T: ?Sized + ValueLike> Ord for Destruct<'a, T>
where
	T::Resource: Ord,
{
	fn cmp(&self, other: &Self) -> Ordering {
		Self::cmp(*self, *other)
	}
}
