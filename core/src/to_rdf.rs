use crate::{
	Model,
	vocab::LocQuad
};

impl<F> Model<F> {
	pub fn to_rdf(&self, quads: &mut Vec<LocQuad<F>>) {
		todo!()
	}
}