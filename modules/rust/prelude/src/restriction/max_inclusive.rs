use crate::ty;

pub trait MaxInclusive<T> {
	fn check(&self, value: &T) -> bool;
}

macro_rules! impl_for {
	{ $($ty:ty),* } => {
		$(
			impl MaxInclusive<$ty> for $ty {
				fn check(&self, value: &$ty) -> bool {
					self <= value
				}
			}
		)*
	};
}

impl_for! {
	ty::Integer,
	ty::NonNegativeInteger,
	ty::NonPositiveInteger,
	ty::PositiveInteger,
	ty::NegativeInteger,
	ty::U64,
	ty::U32,
	ty::U16,
	ty::U8,
	ty::I64,
	ty::I32,
	ty::I16,
	ty::I8,
	f64,
	f32
}
