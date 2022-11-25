use locspan::Meta;

use crate::{Model, Multiple, TId, Type};

#[derive(Debug)]
pub struct Union<M> {
	options: Multiple<TId<Type>, M>,
}

impl<M> Union<M> {
	pub fn new(options: Multiple<TId<Type>, M>) -> Self {
		// let mut properties = Properties::none();
		// for &ty_ref in options.keys() {
		// 	if let Some(ty_properties) = get(ty_ref).properties() {
		// 		properties.unite_with(ty_properties);
		// 	}
		// }

		// Self {
		// 	options,
		// 	properties,
		// }
		Self { options }
	}

	pub fn options(&self) -> &Multiple<TId<Type>, M> {
		&self.options
	}

	pub fn is_datatype(&self, model: &Model<M>) -> bool {
		self.options
			.iter()
			.all(|Meta(ty_ref, _)| model.get(*ty_ref).unwrap().as_type().is_datatype(model))
	}
}
