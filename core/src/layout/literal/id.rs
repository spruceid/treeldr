use crate::Ref;

use super::LiteralLayout;

pub struct IdLayout<R> {
	pub literal: Ref<R, LiteralLayout<R>>,
}
