use crate::Ref;

use super::LiteralLayout;

pub struct IdLayout<R> {
	pub data: Ref<R, LiteralLayout<R>>,
}
