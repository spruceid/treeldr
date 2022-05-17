pub use treeldr::{
	Id,
	ty::data::Primitive
};
use crate::{
	context,
	Error
};

#[derive(Clone)]
pub enum DataType {
	Unknown,
	Primitive(Primitive),
	Derived(Derived)
}

impl Default for DataType {
	fn default() -> Self {
		Self::Unknown
	}
}

impl DataType {
	pub fn build<F>(
		self,
		nodes: &context::allocated::Nodes<F>,
	) -> Result<treeldr::ty::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		// let mut result = treeldr::ty::Normal::new();

		// for (prop_id, prop_causes) in self.properties {
		// 	let prop_ref = nodes.require_property(prop_id, prop_causes.preferred().cloned())?;
		// 	result.insert_property(*prop_ref.inner(), prop_causes)
		// }

		// Ok(treeldr::ty::Description::Normal(result))
		todo!()
	}
}

#[derive(Clone)]
pub struct Derived {
	base: Id,
	restrictions: Vec<Restriction>
}

#[derive(Clone)]
pub enum Restriction {
	// ...
}