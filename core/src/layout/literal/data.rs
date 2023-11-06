use crate::Ref;

/// Data layout.
pub struct DataLayout<R> {
	/// Identifier.
	pub id: R,

	pub derived_from: Option<Ref<R, Self>>,
}
