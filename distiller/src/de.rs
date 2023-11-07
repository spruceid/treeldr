use rdf_types::Interpretation;
use treeldr::{layout::LayoutType, Context, Layout, Ref};

use crate::Value;

pub enum Error {
	IncompatibleLayout,
	AbstractLayout,
}

/// Deserialize the given `value` according to the provided `layout`, returning
/// the deserialized RDF dataset.
pub fn dehydrate<V, I: Interpretation>(
	vocabulary: &V,
	interpretation: &I,
	context: &Context<I::Resource>,
	value: &Value,
	layout_ref: &Ref<I::Resource, LayoutType>,
) -> Result<(grdf::BTreeDataset<I::Resource>, Vec<I::Resource>), Error> {
	match context.get(layout_ref).unwrap() {
		Layout::Never => Err(Error::IncompatibleLayout),
		Layout::Literal(_) => {
			todo!()
		}
		Layout::Sum(_) => {
			todo!()
		}
		Layout::Product(_) => {
			todo!()
		}
		Layout::List(_) => {
			todo!()
		}
		Layout::Always => Err(Error::AbstractLayout),
	}
}
