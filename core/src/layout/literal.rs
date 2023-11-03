use inferdf::Id;

use crate::Ref;

/// Literal layout.
pub struct LiteralLayout {
	/// Identifier.
	pub id: Id,

	pub derived_from: Option<Ref<LiteralLayout>>
}